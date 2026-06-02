use crate::traits::StorageEngine;
use async_trait::async_trait;
use kilnchain_error::KilnchainError;
use lru::LruCache;
use std::num::NonZeroUsize;
use std::sync::{Arc, RwLock};

/// LRU 缓存包装层
pub struct CachedStorage<E: StorageEngine> {
    inner: E,
    cache: Arc<RwLock<LruCache<Vec<u8>, Vec<u8>>>>,
}

impl<E: StorageEngine + Clone> Clone for CachedStorage<E> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            cache: self.cache.clone(),
        }
    }
}

impl<E: StorageEngine> CachedStorage<E> {
    pub fn new(inner: E, capacity: usize) -> Self {
        let capacity = NonZeroUsize::new(capacity).unwrap_or_else(|| NonZeroUsize::new(1).unwrap());
        CachedStorage {
            inner,
            cache: Arc::new(RwLock::new(LruCache::new(capacity))),
        }
    }
}

#[async_trait]
impl<E: StorageEngine> StorageEngine for CachedStorage<E> {
    async fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, KilnchainError> {
        {
            let mut cache = self
                .cache
                .write()
                .map_err(|_| KilnchainError::Storage("cache lock poisoned".to_string()))?;
            if let Some(value) = cache.get(key) {
                return Ok(Some(value.clone()));
            }
        }
        let value = self.inner.get(key).await?;
        if let Some(ref v) = value {
            let mut cache = self
                .cache
                .write()
                .map_err(|_| KilnchainError::Storage("cache lock poisoned".to_string()))?;
            cache.put(key.to_vec(), v.clone());
        }
        Ok(value)
    }

    async fn put(&self, key: &[u8], value: &[u8]) -> Result<(), KilnchainError> {
        self.inner.put(key, value).await?;
        let mut cache = self
            .cache
            .write()
            .map_err(|_| KilnchainError::Storage("cache lock poisoned".to_string()))?;
        cache.put(key.to_vec(), value.to_vec());
        Ok(())
    }

    async fn delete(&self, key: &[u8]) -> Result<(), KilnchainError> {
        self.inner.delete(key).await?;
        let mut cache = self
            .cache
            .write()
            .map_err(|_| KilnchainError::Storage("cache lock poisoned".to_string()))?;
        cache.pop(key);
        Ok(())
    }

    async fn contains(&self, key: &[u8]) -> Result<bool, KilnchainError> {
        {
            let cache = self
                .cache
                .read()
                .map_err(|_| KilnchainError::Storage("cache lock poisoned".to_string()))?;
            if cache.contains(key) {
                return Ok(true);
            }
        }
        self.inner.contains(key).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::InMemoryStorage;

    #[tokio::test]
    async fn test_cache_hit() {
        let inner = InMemoryStorage::new();
        inner.put(b"key1", b"value1").await.unwrap();
        let cached = CachedStorage::new(inner, 10);

        // 第一次读取，缓存未命中
        assert_eq!(cached.get(b"key1").await.unwrap(), Some(b"value1".to_vec()));
        // 第二次读取，缓存命中
        assert_eq!(cached.get(b"key1").await.unwrap(), Some(b"value1".to_vec()));
    }

    #[tokio::test]
    async fn test_cache_invalidation_on_put() {
        let inner = InMemoryStorage::new();
        let cached = CachedStorage::new(inner, 10);

        cached.put(b"key1", b"old").await.unwrap();
        assert_eq!(cached.get(b"key1").await.unwrap(), Some(b"old".to_vec()));

        cached.put(b"key1", b"new").await.unwrap();
        assert_eq!(cached.get(b"key1").await.unwrap(), Some(b"new".to_vec()));
    }

    #[tokio::test]
    async fn test_cache_invalidation_on_delete() {
        let inner = InMemoryStorage::new();
        let cached = CachedStorage::new(inner, 10);

        cached.put(b"key1", b"value1").await.unwrap();
        assert_eq!(cached.get(b"key1").await.unwrap(), Some(b"value1".to_vec()));

        cached.delete(b"key1").await.unwrap();
        assert_eq!(cached.get(b"key1").await.unwrap(), None);
    }
}
