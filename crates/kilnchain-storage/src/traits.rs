use async_trait::async_trait;
use kilnchain_error::KilnchainError;

/// 异步存储引擎核心 trait
#[async_trait]
pub trait StorageEngine: Send + Sync {
    async fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, KilnchainError>;
    async fn put(&self, key: &[u8], value: &[u8]) -> Result<(), KilnchainError>;
    async fn delete(&self, key: &[u8]) -> Result<(), KilnchainError>;
    async fn contains(&self, key: &[u8]) -> Result<bool, KilnchainError>;
}

/// 批量写入 trait
#[async_trait]
pub trait BatchWrite: StorageEngine {
    async fn write_batch(
        &self,
        items: Vec<(Vec<u8>, Option<Vec<u8>>)>,
    ) -> Result<(), KilnchainError>;
}

/// 只读快照
pub trait Snapshot: Send + Sync {
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, KilnchainError>;
}

/// 支持创建快照的存储引擎
#[async_trait]
pub trait Snapshotable: StorageEngine {
    async fn snapshot(&self) -> Result<Box<dyn Snapshot>, KilnchainError>;
}
