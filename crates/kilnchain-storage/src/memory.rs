use crate::traits::{BatchWrite, Snapshot, Snapshotable, StorageEngine};
use async_trait::async_trait;
use kilnchain_error::KilnchainError;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// 纯内存存储后端
#[derive(Clone)]
pub struct InMemoryStorage {
    data: Arc<RwLock<HashMap<Vec<u8>, Vec<u8>>>>,
}

impl InMemoryStorage {
    pub fn new() -> Self {
        InMemoryStorage {
            data: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for InMemoryStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl StorageEngine for InMemoryStorage {
    async fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, KilnchainError> {
        let data = self
            .data
            .read()
            .map_err(|_| KilnchainError::Storage("lock poisoned".to_string()))?;
        Ok(data.get(key).cloned())
    }

    async fn put(&self, key: &[u8], value: &[u8]) -> Result<(), KilnchainError> {
        let mut data = self
            .data
            .write()
            .map_err(|_| KilnchainError::Storage("lock poisoned".to_string()))?;
        data.insert(key.to_vec(), value.to_vec());
        Ok(())
    }

    async fn delete(&self, key: &[u8]) -> Result<(), KilnchainError> {
        let mut data = self
            .data
            .write()
            .map_err(|_| KilnchainError::Storage("lock poisoned".to_string()))?;
        data.remove(key);
        Ok(())
    }

    async fn contains(&self, key: &[u8]) -> Result<bool, KilnchainError> {
        let data = self
            .data
            .read()
            .map_err(|_| KilnchainError::Storage("lock poisoned".to_string()))?;
        Ok(data.contains_key(key))
    }
}

#[async_trait]
impl BatchWrite for InMemoryStorage {
    async fn write_batch(
        &self,
        items: Vec<(Vec<u8>, Option<Vec<u8>>)>,
    ) -> Result<(), KilnchainError> {
        let mut data = self
            .data
            .write()
            .map_err(|_| KilnchainError::Storage("lock poisoned".to_string()))?;
        for (key, value) in items {
            match value {
                Some(v) => data.insert(key, v),
                None => data.remove(&key),
            };
        }
        Ok(())
    }
}

/// 内存快照（HashMap 的只读克隆）
struct InMemorySnapshot {
    data: HashMap<Vec<u8>, Vec<u8>>,
}

impl Snapshot for InMemorySnapshot {
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, KilnchainError> {
        Ok(self.data.get(key).cloned())
    }
}

#[async_trait]
impl Snapshotable for InMemoryStorage {
    async fn snapshot(&self) -> Result<Box<dyn Snapshot>, KilnchainError> {
        let data = self
            .data
            .read()
            .map_err(|_| KilnchainError::Storage("lock poisoned".to_string()))?;
        Ok(Box::new(InMemorySnapshot { data: data.clone() }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_put_get_roundtrip() {
        let storage = InMemoryStorage::new();
        storage.put(b"key1", b"value1").await.unwrap();
        assert_eq!(
            storage.get(b"key1").await.unwrap(),
            Some(b"value1".to_vec())
        );
    }

    #[tokio::test]
    async fn test_delete() {
        let storage = InMemoryStorage::new();
        storage.put(b"key1", b"value1").await.unwrap();
        storage.delete(b"key1").await.unwrap();
        assert_eq!(storage.get(b"key1").await.unwrap(), None);
    }

    #[tokio::test]
    async fn test_contains() {
        let storage = InMemoryStorage::new();
        assert!(!storage.contains(b"key1").await.unwrap());
        storage.put(b"key1", b"value1").await.unwrap();
        assert!(storage.contains(b"key1").await.unwrap());
    }

    #[tokio::test]
    async fn test_write_batch() {
        let storage = InMemoryStorage::new();
        let items = (0..100)
            .map(|i| {
                (
                    format!("key{}", i).into_bytes(),
                    Some(format!("value{}", i).into_bytes()),
                )
            })
            .collect();
        storage.write_batch(items).await.unwrap();
        for i in 0..100 {
            assert_eq!(
                storage.get(format!("key{}", i).as_bytes()).await.unwrap(),
                Some(format!("value{}", i).into_bytes())
            );
        }
    }

    #[tokio::test]
    async fn test_batch_delete() {
        let storage = InMemoryStorage::new();
        storage.put(b"a", b"1").await.unwrap();
        storage.put(b"b", b"2").await.unwrap();
        storage
            .write_batch(vec![
                (b"a".to_vec(), Some(b"updated".to_vec())),
                (b"b".to_vec(), None),
            ])
            .await
            .unwrap();
        assert_eq!(storage.get(b"a").await.unwrap(), Some(b"updated".to_vec()));
        assert_eq!(storage.get(b"b").await.unwrap(), None);
    }

    #[tokio::test]
    async fn test_snapshot_isolation() {
        let storage = InMemoryStorage::new();
        storage.put(b"x", b"old").await.unwrap();
        let snap = storage.snapshot().await.unwrap();
        storage.put(b"x", b"new").await.unwrap();
        assert_eq!(snap.get(b"x").unwrap(), Some(b"old".to_vec()));
        assert_eq!(storage.get(b"x").await.unwrap(), Some(b"new".to_vec()));
    }

    #[tokio::test]
    async fn test_send_sync() {
        let storage = InMemoryStorage::new();
        let handle = tokio::spawn(async move {
            storage.put(b"key", b"value").await.unwrap();
        });
        handle.await.unwrap();
    }
}
