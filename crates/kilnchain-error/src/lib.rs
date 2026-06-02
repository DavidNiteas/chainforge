use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum KilnchainError {
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
