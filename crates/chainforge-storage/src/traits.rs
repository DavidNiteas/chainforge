use async_trait::async_trait;
use chainforge_error::ChainforgeError;

/// 异步存储引擎核心 trait
#[async_trait]
pub trait StorageEngine: Send + Sync {
    async fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, ChainforgeError>;
    async fn put(&self, key: &[u8], value: &[u8]) -> Result<(), ChainforgeError>;
    async fn delete(&self, key: &[u8]) -> Result<(), ChainforgeError>;
    async fn contains(&self, key: &[u8]) -> Result<bool, ChainforgeError>;
}

/// 批量写入 trait
#[async_trait]
pub trait BatchWrite: StorageEngine {
    async fn write_batch(
        &self,
        items: Vec<(Vec<u8>, Option<Vec<u8>>)>,
    ) -> Result<(), ChainforgeError>;
}

/// 只读快照
pub trait Snapshot: Send + Sync {
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, ChainforgeError>;
}

/// 支持创建快照的存储引擎
#[async_trait]
pub trait Snapshotable: StorageEngine {
    async fn snapshot(&self) -> Result<Box<dyn Snapshot>, ChainforgeError>;
}
