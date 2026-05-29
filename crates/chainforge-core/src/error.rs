use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum ChainforgeError {
    #[error("cryptographic operation failed: {0}")]
    Crypto(String),

    #[error("serialization error: {0}")]
    Serialization(String),

    #[error("storage engine error: {0}")]
    Storage(String),

    #[error("invalid parameter: {0}")]
    InvalidParameter(String),

    #[error("state root mismatch: expected {expected}, got {actual}")]
    StateRootMismatch { expected: String, actual: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crypto_error_display() {
        let err = ChainforgeError::Crypto("bad signature".to_string());
        assert_eq!(
            err.to_string(),
            "cryptographic operation failed: bad signature"
        );
    }

    #[test]
    fn test_serialization_error_display() {
        let err = ChainforgeError::Serialization("unexpected EOF".to_string());
        assert_eq!(err.to_string(), "serialization error: unexpected EOF");
    }

    #[test]
    fn test_storage_error_display() {
        let err = ChainforgeError::Storage("disk full".to_string());
        assert_eq!(err.to_string(), "storage engine error: disk full");
    }

    #[test]
    fn test_invalid_parameter_display() {
        let err = ChainforgeError::InvalidParameter("negative amount".to_string());
        assert_eq!(err.to_string(), "invalid parameter: negative amount");
    }

    #[test]
    fn test_state_root_mismatch_display() {
        let err = ChainforgeError::StateRootMismatch {
            expected: "0xabc".to_string(),
            actual: "0xdef".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "state root mismatch: expected 0xabc, got 0xdef"
        );
    }

    #[test]
    fn test_error_clone_and_equality() {
        let err1 = ChainforgeError::Crypto("x".to_string());
        let err2 = err1.clone();
        assert_eq!(err1, err2);
    }
}
