use crate::rlp::{RlpDecoder, RlpEncoder};
use chainforge_crypto::ecdsa::{PublicKey, SecretKey, Signature};
use chainforge_crypto::keccak256;
use chainforge_error::ChainforgeError;
use serde::{Deserialize, Serialize};

/// 交易结构（兼容 Ethereum 格式）
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Transaction {
    pub nonce: u64,
    pub gas_price: u128,
    pub gas_limit: u64,
    pub to: Option<[u8; 20]>,
    pub value: u128,
    pub data: Vec<u8>,
    pub v: u64,
    pub r: [u8; 32],
    pub s: [u8; 32],
}

impl Transaction {
    /// RLP 编码交易
    pub fn encode_rlp(&self) -> Vec<u8> {
        let mut enc = RlpEncoder::new();
        enc.encode_list(|e| {
            e.encode_u64(self.nonce);
            e.encode_u128(self.gas_price);
            e.encode_u64(self.gas_limit);
            match self.to {
                Some(addr) => e.encode_bytes(&addr),
                None => e.encode_bytes(&[]),
            }
            e.encode_u128(self.value);
            e.encode_bytes(&self.data);
            e.encode_u64(self.v);
            e.encode_bytes(&self.r);
            e.encode_bytes(&self.s);
        });
        enc.finish()
    }

    /// RLP 解码交易
    pub fn decode_rlp(data: &[u8]) -> Result<Self, ChainforgeError> {
        let mut dec = RlpDecoder::new(data);
        let items: Vec<Vec<u8>> = dec.decode_list(|d| Ok(d.decode_bytes()?.to_vec()))?;
        if items.len() != 9 {
            return Err(ChainforgeError::Serialization(format!(
                "expected 9 RLP items, got {}",
                items.len()
            )));
        }

        fn bytes_to_u64(bytes: &[u8]) -> u64 {
            if bytes.is_empty() {
                0
            } else {
                let mut arr = [0u8; 8];
                arr[8 - bytes.len()..].copy_from_slice(bytes);
                u64::from_be_bytes(arr)
            }
        }
        fn bytes_to_u128(bytes: &[u8]) -> u128 {
            if bytes.is_empty() {
                0
            } else {
                let mut arr = [0u8; 16];
                arr[16 - bytes.len()..].copy_from_slice(bytes);
                u128::from_be_bytes(arr)
            }
        }

        let to = if items[3].is_empty() {
            None
        } else {
            if items[3].len() != 20 {
                return Err(ChainforgeError::InvalidParameter(
                    "invalid to address length".to_string(),
                ));
            }
            let mut addr = [0u8; 20];
            addr.copy_from_slice(&items[3]);
            Some(addr)
        };

        let r = if items[7].len() == 32 {
            let mut arr = [0u8; 32];
            arr.copy_from_slice(&items[7]);
            arr
        } else {
            [0u8; 32]
        };

        let s = if items[8].len() == 32 {
            let mut arr = [0u8; 32];
            arr.copy_from_slice(&items[8]);
            arr
        } else {
            [0u8; 32]
        };

        Ok(Transaction {
            nonce: bytes_to_u64(&items[0]),
            gas_price: bytes_to_u128(&items[1]),
            gas_limit: bytes_to_u64(&items[2]),
            to,
            value: bytes_to_u128(&items[4]),
            data: items[5].clone(),
            v: bytes_to_u64(&items[6]),
            r,
            s,
        })
    }

    /// 计算交易哈希（RLP 编码后 Keccak-256）
    pub fn hash(&self) -> [u8; 32] {
        keccak256(&self.encode_rlp())
    }

    /// 恢复发送方地址（20 字节）
    pub fn recover_sender(&self) -> Result<[u8; 20], ChainforgeError> {
        let mut sig_bytes = [0u8; 64];
        sig_bytes[..32].copy_from_slice(&self.r);
        sig_bytes[32..].copy_from_slice(&self.s);
        let sig = Signature::from_bytes(&sig_bytes, ((self.v - 27) % 2) as u8)
            .map_err(|e| ChainforgeError::Crypto(e.to_string()))?;
        let pk = PublicKey::recover_from_msg(&self.unsigned_hash(), &sig)
            .map_err(|e| ChainforgeError::Crypto(e.to_string()))?;
        let pk_bytes = pk.to_bytes();
        // 解压公钥（去掉 0x02/0x03 前缀）后哈希
        let uncompressed = &pk_bytes[1..];
        let hash = keccak256(uncompressed);
        let mut addr = [0u8; 20];
        addr.copy_from_slice(&hash[12..]);
        Ok(addr)
    }

    /// 返回待签名的哈希（不含 v, r, s 的 RLP 编码后 Keccak-256）
    pub fn unsigned_hash(&self) -> [u8; 32] {
        let mut enc = RlpEncoder::new();
        enc.encode_list(|e| {
            e.encode_u64(self.nonce);
            e.encode_u128(self.gas_price);
            e.encode_u64(self.gas_limit);
            match self.to {
                Some(addr) => e.encode_bytes(&addr),
                None => e.encode_bytes(&[]),
            }
            e.encode_u128(self.value);
            e.encode_bytes(&self.data);
        });
        keccak256(&enc.finish())
    }

    /// 使用私钥对交易进行签名
    pub fn sign(&mut self, sk: &SecretKey) -> Result<(), ChainforgeError> {
        let hash = self.unsigned_hash();
        let sig = sk
            .sign(&hash)
            .map_err(|e| ChainforgeError::Crypto(e.to_string()))?;
        let sig_bytes = sig.to_bytes();
        self.r.copy_from_slice(&sig_bytes[..32]);
        self.s.copy_from_slice(&sig_bytes[32..]);
        self.v = 27 + sig.recovery_id() as u64;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tx_rlp_roundtrip() {
        let tx = Transaction {
            nonce: 0,
            gas_price: 1,
            gas_limit: 21000,
            to: Some([0xabu8; 20]),
            value: 1000,
            data: vec![1, 2, 3],
            v: 27,
            r: [0u8; 32],
            s: [0u8; 32],
        };
        let encoded = tx.encode_rlp();
        let decoded = Transaction::decode_rlp(&encoded).unwrap();
        assert_eq!(tx, decoded);
    }

    #[test]
    fn test_tx_hash_length() {
        let tx = Transaction {
            nonce: 0,
            gas_price: 0,
            gas_limit: 0,
            to: None,
            value: 0,
            data: vec![],
            v: 0,
            r: [0u8; 32],
            s: [0u8; 32],
        };
        assert_eq!(tx.hash().len(), 32);
    }

    #[test]
    fn test_recover_sender() {
        let sk = SecretKey::random();
        let pk = sk.public_key();
        let expected_addr = {
            let pk_bytes = pk.to_bytes();
            let uncompressed = &pk_bytes[1..];
            let hash = keccak256(uncompressed);
            let mut addr = [0u8; 20];
            addr.copy_from_slice(&hash[12..]);
            addr
        };

        let mut tx = Transaction {
            nonce: 0,
            gas_price: 1,
            gas_limit: 21000,
            to: Some([0xabu8; 20]),
            value: 1000,
            data: vec![1, 2, 3],
            v: 0,
            r: [0u8; 32],
            s: [0u8; 32],
        };
        tx.sign(&sk).unwrap();
        let sender = tx.recover_sender().unwrap();
        assert_eq!(sender, expected_addr);
    }
}
