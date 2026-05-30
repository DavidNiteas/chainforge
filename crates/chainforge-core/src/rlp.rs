use chainforge_error::ChainforgeError;

/// RLP 编码器
pub struct RlpEncoder {
    buf: Vec<u8>,
}

impl Default for RlpEncoder {
    fn default() -> Self {
        Self::new()
    }
}

impl RlpEncoder {
    pub fn new() -> Self {
        RlpEncoder { buf: Vec::new() }
    }

    pub fn finish(self) -> Vec<u8> {
        self.buf
    }

    pub fn encode_bytes(&mut self, bytes: &[u8]) {
        if bytes.len() == 1 && bytes[0] < 0x80 {
            self.buf.push(bytes[0]);
        } else if bytes.len() <= 55 {
            self.buf.push(0x80 + bytes.len() as u8);
            self.buf.extend_from_slice(bytes);
        } else {
            let len_bytes = encode_length(bytes.len());
            self.buf.push(0xb7 + len_bytes.len() as u8);
            self.buf.extend_from_slice(&len_bytes);
            self.buf.extend_from_slice(bytes);
        }
    }

    pub fn encode_list<F>(&mut self, f: F)
    where
        F: FnOnce(&mut RlpEncoder),
    {
        let mut inner = RlpEncoder::new();
        f(&mut inner);
        let payload = inner.finish();
        if payload.len() <= 55 {
            self.buf.push(0xc0 + payload.len() as u8);
            self.buf.extend_from_slice(&payload);
        } else {
            let len_bytes = encode_length(payload.len());
            self.buf.push(0xf7 + len_bytes.len() as u8);
            self.buf.extend_from_slice(&len_bytes);
            self.buf.extend_from_slice(&payload);
        }
    }

    pub fn encode_u64(&mut self, val: u64) {
        if val == 0 {
            self.encode_bytes(&[]);
        } else {
            let bytes = val.to_be_bytes();
            let start = bytes.iter().position(|&b| b != 0).unwrap_or(7);
            self.encode_bytes(&bytes[start..]);
        }
    }

    pub fn encode_u128(&mut self, val: u128) {
        if val == 0 {
            self.encode_bytes(&[]);
        } else {
            let bytes = val.to_be_bytes();
            let start = bytes.iter().position(|&b| b != 0).unwrap_or(15);
            self.encode_bytes(&bytes[start..]);
        }
    }
}

fn encode_length(len: usize) -> Vec<u8> {
    let mut result = Vec::new();
    let mut n = len;
    while n > 0 {
        result.push((n & 0xff) as u8);
        n >>= 8;
    }
    result.reverse();
    result
}

/// RLP 解码器
pub struct RlpDecoder<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> RlpDecoder<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        RlpDecoder { data, pos: 0 }
    }

    pub fn decode_bytes(&mut self) -> Result<&'a [u8], ChainforgeError> {
        let prefix = self.read_byte()?;
        if prefix < 0x80 {
            self.pos -= 1;
            let result = &self.data[self.pos..self.pos + 1];
            self.pos += 1;
            Ok(result)
        } else if prefix <= 0xb7 {
            let len = (prefix - 0x80) as usize;
            self.ensure(len)?;
            let result = &self.data[self.pos..self.pos + len];
            self.pos += len;
            Ok(result)
        } else if prefix <= 0xbf {
            let len_len = (prefix - 0xb7) as usize;
            self.ensure(len_len)?;
            let len = decode_length(&self.data[self.pos..self.pos + len_len]);
            self.pos += len_len;
            self.ensure(len)?;
            let result = &self.data[self.pos..self.pos + len];
            self.pos += len;
            Ok(result)
        } else {
            Err(ChainforgeError::Serialization(
                "unexpected list prefix in byte decoding".to_string(),
            ))
        }
    }

    pub fn decode_list<F, T>(&mut self, f: F) -> Result<Vec<T>, ChainforgeError>
    where
        F: Fn(&mut RlpDecoder<'a>) -> Result<T, ChainforgeError>,
    {
        let prefix = self.read_byte()?;
        if prefix < 0xc0 {
            return Err(ChainforgeError::Serialization(
                "expected list prefix".to_string(),
            ));
        }
        let payload = if prefix <= 0xf7 {
            let len = (prefix - 0xc0) as usize;
            self.ensure(len)?;
            let payload = &self.data[self.pos..self.pos + len];
            self.pos += len;
            payload
        } else {
            let len_len = (prefix - 0xf7) as usize;
            self.ensure(len_len)?;
            let len = decode_length(&self.data[self.pos..self.pos + len_len]);
            self.pos += len_len;
            self.ensure(len)?;
            let payload = &self.data[self.pos..self.pos + len];
            self.pos += len;
            payload
        };

        let mut inner = RlpDecoder::new(payload);
        let mut result = Vec::new();
        while inner.pos < inner.data.len() {
            result.push(f(&mut inner)?);
        }
        Ok(result)
    }

    pub fn decode_u64(&mut self) -> Result<u64, ChainforgeError> {
        let bytes = self.decode_bytes()?;
        if bytes.is_empty() {
            Ok(0)
        } else {
            let mut arr = [0u8; 8];
            arr[8 - bytes.len()..].copy_from_slice(bytes);
            Ok(u64::from_be_bytes(arr))
        }
    }

    pub fn decode_u128(&mut self) -> Result<u128, ChainforgeError> {
        let bytes = self.decode_bytes()?;
        if bytes.is_empty() {
            Ok(0)
        } else {
            let mut arr = [0u8; 16];
            arr[16 - bytes.len()..].copy_from_slice(bytes);
            Ok(u128::from_be_bytes(arr))
        }
    }

    fn read_byte(&mut self) -> Result<u8, ChainforgeError> {
        if self.pos >= self.data.len() {
            return Err(ChainforgeError::Serialization(
                "unexpected end of RLP data".to_string(),
            ));
        }
        let b = self.data[self.pos];
        self.pos += 1;
        Ok(b)
    }

    fn ensure(&self, len: usize) -> Result<(), ChainforgeError> {
        if self.pos + len > self.data.len() {
            return Err(ChainforgeError::Serialization(
                "RLP data too short".to_string(),
            ));
        }
        Ok(())
    }
}

fn decode_length(bytes: &[u8]) -> usize {
    let mut result = 0usize;
    for &b in bytes {
        result = (result << 8) | (b as usize);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode_u64() {
        let mut enc = RlpEncoder::new();
        enc.encode_u64(0);
        enc.encode_u64(1);
        enc.encode_u64(127);
        enc.encode_u64(128);
        enc.encode_u64(255);
        enc.encode_u64(0x010000);
        let data = enc.finish();

        let mut dec = RlpDecoder::new(&data);
        assert_eq!(dec.decode_u64().unwrap(), 0);
        assert_eq!(dec.decode_u64().unwrap(), 1);
        assert_eq!(dec.decode_u64().unwrap(), 127);
        assert_eq!(dec.decode_u64().unwrap(), 128);
        assert_eq!(dec.decode_u64().unwrap(), 255);
        assert_eq!(dec.decode_u64().unwrap(), 0x010000);
    }

    #[test]
    fn test_encode_decode_bytes() {
        let mut enc = RlpEncoder::new();
        enc.encode_bytes(&[0x7f]);
        enc.encode_bytes(&[0x80]);
        enc.encode_bytes(&[1, 2, 3, 4, 5]);
        let data = enc.finish();

        let mut dec = RlpDecoder::new(&data);
        assert_eq!(dec.decode_bytes().unwrap(), &[0x7f]);
        assert_eq!(dec.decode_bytes().unwrap(), &[0x80]);
        assert_eq!(dec.decode_bytes().unwrap(), &[1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_encode_decode_list() {
        let mut enc = RlpEncoder::new();
        enc.encode_list(|e| {
            e.encode_u64(1);
            e.encode_u64(2);
            e.encode_u64(3);
        });
        let data = enc.finish();

        let mut dec = RlpDecoder::new(&data);
        let items: Vec<u64> = dec.decode_list(|d| d.decode_u64()).unwrap();
        assert_eq!(items, vec![1, 2, 3]);
    }
}
