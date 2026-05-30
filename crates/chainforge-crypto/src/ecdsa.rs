use chainforge_error::ChainforgeError;
use secp256k1::ecdsa::{RecoverableSignature, RecoveryId, Signature as SecpSignature};
use secp256k1::{Message, PublicKey as SecpPublicKey, SecretKey as SecpSecretKey};

/// Secp256k1 私钥（32 字节）
#[derive(Debug, Clone)]
pub struct SecretKey([u8; 32]);

/// Secp256k1 公钥（33 字节压缩格式）
#[derive(Debug, Clone, PartialEq)]
pub struct PublicKey([u8; 33]);

/// Secp256k1 ECDSA 签名（64 字节 + recovery id）
#[derive(Debug, Clone, PartialEq)]
pub struct Signature {
    bytes: [u8; 64],
    recovery_id: u8,
}

impl SecretKey {
    /// 生成密码学安全的随机私钥
    pub fn random() -> Self {
        let sk = SecpSecretKey::new(&mut rand::thread_rng());
        let bytes = sk.secret_bytes();
        SecretKey(bytes)
    }

    /// 从 32 字节数组构造私钥
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ChainforgeError> {
        if bytes.len() != 32 {
            return Err(ChainforgeError::Crypto(format!(
                "invalid secret key length: expected 32, got {}",
                bytes.len()
            )));
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(bytes);
        Ok(SecretKey(arr))
    }

    /// 派生公钥
    pub fn public_key(&self) -> PublicKey {
        let secp_sk = SecpSecretKey::from_slice(&self.0).expect("valid secret key bytes");
        let secp_pk = SecpPublicKey::from_secret_key_global(&secp_sk);
        let bytes = secp_pk.serialize();
        PublicKey(bytes)
    }

    /// 对消息进行 ECDSA 签名（消息内部会先被 keccak256 哈希）
    pub fn sign(&self, msg: &[u8]) -> Result<Signature, ChainforgeError> {
        let secp_sk = SecpSecretKey::from_slice(&self.0)
            .map_err(|e| ChainforgeError::Crypto(e.to_string()))?;
        let msg_hash = crate::hash::keccak256(msg);
        let message = Message::from_digest_slice(&msg_hash)
            .map_err(|e| ChainforgeError::Crypto(e.to_string()))?;
        let recoverable_sig =
            secp256k1::global::SECP256K1.sign_ecdsa_recoverable(&message, &secp_sk);
        let (recid, bytes) = recoverable_sig.serialize_compact();
        let recovery_id = match recid {
            RecoveryId::Zero => 0,
            RecoveryId::One => 1,
            RecoveryId::Two => 2,
            RecoveryId::Three => 3,
        };
        Ok(Signature { bytes, recovery_id })
    }
}

impl PublicKey {
    /// 从 33 字节压缩公钥构造
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ChainforgeError> {
        if bytes.len() != 33 {
            return Err(ChainforgeError::Crypto(format!(
                "invalid public key length: expected 33, got {}",
                bytes.len()
            )));
        }
        let mut arr = [0u8; 33];
        arr.copy_from_slice(bytes);
        Ok(PublicKey(arr))
    }

    /// 验证签名（消息内部会先被 keccak256 哈希）
    pub fn verify(&self, msg: &[u8], sig: &Signature) -> Result<bool, ChainforgeError> {
        let secp_pk = SecpPublicKey::from_slice(&self.0)
            .map_err(|e| ChainforgeError::Crypto(e.to_string()))?;
        let msg_hash = crate::hash::keccak256(msg);
        let message = Message::from_digest_slice(&msg_hash)
            .map_err(|e| ChainforgeError::Crypto(e.to_string()))?;
        let secp_sig = SecpSignature::from_compact(&sig.bytes)
            .map_err(|e| ChainforgeError::Crypto(e.to_string()))?;
        match secp256k1::global::SECP256K1.verify_ecdsa(&message, &secp_sig, &secp_pk) {
            Ok(()) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// 从签名恢复公钥
    pub fn recover_from_msg(msg: &[u8], sig: &Signature) -> Result<Self, ChainforgeError> {
        let msg_hash = crate::hash::keccak256(msg);
        let message = Message::from_digest_slice(&msg_hash)
            .map_err(|e| ChainforgeError::Crypto(e.to_string()))?;
        // 尝试 recovery id 0..3
        let recid = match sig.recovery_id {
            0 => RecoveryId::Zero,
            1 => RecoveryId::One,
            2 => RecoveryId::Two,
            3 => RecoveryId::Three,
            _ => return Err(ChainforgeError::Crypto("invalid recovery id".to_string())),
        };
        let recoverable = RecoverableSignature::from_compact(&sig.bytes, recid)
            .map_err(|e| ChainforgeError::Crypto(e.to_string()))?;
        let recovered = secp256k1::global::SECP256K1
            .recover_ecdsa(&message, &recoverable)
            .map_err(|e| ChainforgeError::Crypto(e.to_string()))?;
        Ok(PublicKey(recovered.serialize()))
    }

    /// 返回压缩格式公钥字节
    pub fn to_bytes(&self) -> [u8; 33] {
        self.0
    }
}

impl Signature {
    /// 返回 64 字节签名
    pub fn to_bytes(&self) -> [u8; 64] {
        self.bytes
    }

    /// 返回 recovery id
    pub fn recovery_id(&self) -> u8 {
        self.recovery_id
    }

    /// 从 64 字节 + recovery id 构造签名
    pub fn from_bytes(bytes: &[u8], recovery_id: u8) -> Result<Self, ChainforgeError> {
        if bytes.len() != 64 {
            return Err(ChainforgeError::Crypto(format!(
                "invalid signature length: expected 64, got {}",
                bytes.len()
            )));
        }
        if recovery_id > 3 {
            return Err(ChainforgeError::Crypto(
                "invalid recovery id: expected 0..=3".to_string(),
            ));
        }
        let mut arr = [0u8; 64];
        arr.copy_from_slice(bytes);
        Ok(Signature {
            bytes: arr,
            recovery_id,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_verify_roundtrip() {
        let sk = SecretKey::random();
        let pk = sk.public_key();
        let msg = b"hello world";
        let sig = sk.sign(msg).unwrap();
        assert!(pk.verify(msg, &sig).unwrap());
    }

    #[test]
    fn test_verify_rejects_wrong_message() {
        let sk = SecretKey::random();
        let pk = sk.public_key();
        let msg = b"hello world";
        let sig = sk.sign(msg).unwrap();
        assert!(!pk.verify(b"wrong message", &sig).unwrap());
    }

    #[test]
    fn test_public_key_recovery() {
        let sk = SecretKey::random();
        let pk = sk.public_key();
        let msg = b"recover me";
        let sig = sk.sign(msg).unwrap();
        let recovered = PublicKey::recover_from_msg(msg, &sig).unwrap();
        assert_eq!(pk, recovered);
    }

    #[test]
    fn test_invalid_secret_key_length() {
        let result = SecretKey::from_bytes(&[1u8; 31]);
        assert!(matches!(result, Err(ChainforgeError::Crypto(_))));
    }

    #[test]
    fn test_signature_serialization_roundtrip() {
        let sk = SecretKey::random();
        let msg = b"serialization test";
        let sig = sk.sign(msg).unwrap();
        let bytes = sig.to_bytes();
        let sig2 = Signature::from_bytes(&bytes, sig.recovery_id()).unwrap();
        assert_eq!(sig, sig2);
    }
}
