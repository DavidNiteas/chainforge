use ring::digest::{digest, SHA256};
use ripemd::Ripemd160;
use tiny_keccak::{Hasher, Keccak};

/// SHA-256 哈希
pub fn sha256(data: &[u8]) -> [u8; 32] {
    let digest = digest(&SHA256, data);
    let mut result = [0u8; 32];
    result.copy_from_slice(digest.as_ref());
    result
}

/// Keccak-256 哈希（Ethereum 标准）
pub fn keccak256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Keccak::v256();
    hasher.update(data);
    let mut result = [0u8; 32];
    hasher.finalize(&mut result);
    result
}

/// RIPEMD-160 哈希
pub fn ripemd160(data: &[u8]) -> [u8; 20] {
    use ripemd::Digest;
    let digest = Ripemd160::digest(data);
    let mut result = [0u8; 20];
    result.copy_from_slice(&digest);
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn test_sha256_empty() {
        let result = sha256(b"");
        assert_eq!(
            hex::encode(result),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn test_keccak256_empty() {
        let result = keccak256(b"");
        assert_eq!(
            hex::encode(result),
            "c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470"
        );
    }

    #[test]
    fn test_sha256_known_vector() {
        let result = sha256(b"hello");
        assert_eq!(
            hex::encode(result),
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
    }

    #[test]
    fn test_keccak256_known_vector() {
        let result = keccak256(b"hello");
        assert_eq!(
            hex::encode(result),
            "1c8aff950685c2ed4bc3174f3472287b56d9517b9c948127319a09a7a36deac8"
        );
    }

    #[test]
    fn test_ripemd160_known_vector() {
        let result = ripemd160(b"hello");
        assert_eq!(
            hex::encode(result),
            "108f07b8382412612c048d07d13f814118445acd"
        );
    }

    proptest! {
        #[test]
        fn prop_sha256_output_length(data in prop::collection::vec(any::<u8>(), 0..1024)) {
            assert_eq!(sha256(&data).len(), 32);
        }

        #[test]
        fn prop_keccak256_output_length(data in prop::collection::vec(any::<u8>(), 0..1024)) {
            assert_eq!(keccak256(&data).len(), 32);
        }

        #[test]
        fn prop_ripemd160_output_length(data in prop::collection::vec(any::<u8>(), 0..1024)) {
            assert_eq!(ripemd160(&data).len(), 20);
        }

        #[test]
        fn prop_sha256_deterministic(data in prop::collection::vec(any::<u8>(), 0..1024)) {
            assert_eq!(sha256(&data), sha256(&data));
        }

        #[test]
        fn prop_keccak256_deterministic(data in prop::collection::vec(any::<u8>(), 0..1024)) {
            assert_eq!(keccak256(&data), keccak256(&data));
        }

        #[test]
        fn prop_ripemd160_deterministic(data in prop::collection::vec(any::<u8>(), 0..1024)) {
            assert_eq!(ripemd160(&data), ripemd160(&data));
        }
    }
}
