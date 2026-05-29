#[cfg(feature = "rocksdb-backend")]
use crate::traits::{BatchWrite, StorageEngine};
#[cfg(feature = "rocksdb-backend")]
use async_trait::async_trait;
#[cfg(feature = "rocksdb-backend")]
use chainforge_error::ChainforgeError;
#[cfg(feature = "rocksdb-backend")]
use std::path::Path;

#[cfg(feature = "rocksdb-backend")]
pub const CF_META: &str = "meta";
#[cfg(feature = "rocksdb-backend")]
pub const CF_BLOCKS: &str = "blocks";
#[cfg(feature = "rocksdb-backend")]
pub const CF_STATE: &str = "state";
#[cfg(feature = "rocksdb-backend")]
pub const CF_INDEX: &str = "index";

#[cfg(feature = "rocksdb-backend")]
pub const ALL_CFS: &[&str] = &[CF_META, CF_BLOCKS, CF_STATE, CF_INDEX];

/// RocksDB 存储引擎
#[cfg(feature = "rocksdb-backend")]
pub struct RocksDBEngine {
    db: rocksdb::DB,
}

#[cfg(feature = "rocksdb-backend")]
impl RocksDBEngine {
    pub fn open(path: &Path) -> Result<Self, ChainforgeError> {
        let mut opts = rocksdb::Options::default();
        opts.create_if_missing(true);
        opts.create_missing_column_families(true);
        let db = rocksdb::DB::open_cf(&opts, path, ALL_CFS)
            .map_err(|e| ChainforgeError::Storage(e.to_string()))?;
        Ok(Self { db })
    }

    fn cf_handle(&self, cf: &str) -> Result<&rocksdb::ColumnFamily, ChainforgeError> {
        self.db.cf_handle(cf).ok_or_else(|| {
            ChainforgeError::Storage(format!("column family '{}' not found", cf))
        })
    }
}

#[cfg(feature = "rocksdb-backend")]
#[async_trait]
impl StorageEngine for RocksDBEngine {
    async fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, ChainforgeError> {
        let cf = self.cf_handle(CF_STATE)?;
        let result = self.db.get_cf(cf, key)
            .map_err(|e| ChainforgeError::Storage(e.to_string()))?;
        Ok(result)
    }

    async fn put(&self, key: &[u8], value: &[u8]) -> Result<(), ChainforgeError> {
        let cf = self.cf_handle(CF_STATE)?;
        self.db.put_cf(cf, key, value)
            .map_err(|e| ChainforgeError::Storage(e.to_string()))?;
        Ok(())
    }

    async fn delete(&self, key: &[u8]) -> Result<(), ChainforgeError> {
        let cf = self.cf_handle(CF_STATE)?;
        self.db.delete_cf(cf, key)
            .map_err(|e| ChainforgeError::Storage(e.to_string()))?;
        Ok(())
    }

    async fn contains(&self, key: &[u8]) -> Result<bool, ChainforgeError> {
        let cf = self.cf_handle(CF_STATE)?;
        let result = self.db.get_cf(cf, key)
            .map_err(|e| ChainforgeError::Storage(e.to_string()))?;
        Ok(result.is_some())
    }
}

#[cfg(feature = "rocksdb-backend")]
#[async_trait]
impl BatchWrite for RocksDBEngine {
    async fn write_batch(
        &self,
        items: Vec<(Vec<u8>, Option<Vec<u8>>)>,
    ) -> Result<(), ChainforgeError> {
        let cf = self.cf_handle(CF_STATE)?;
        let mut batch = rocksdb::WriteBatch::default();
        for (key, value) in items {
            match value {
                Some(v) => batch.put_cf(cf, &key, &v),
                None => batch.delete_cf(cf, &key),
            };
        }
        self.db.write(batch)
            .map_err(|e| ChainforgeError::Storage(e.to_string()))?;
        Ok(())
    }
}

#[cfg(feature = "rocksdb-backend")]
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn temp_db() -> (RocksDBEngine, PathBuf) {
        let dir = std::env::temp_dir().join(format!("chainforge_test_{}", std::process::id()));
        let engine = RocksDBEngine::open(&dir).unwrap();
        (engine, dir)
    }

    #[tokio::test]
    async fn test_persistence() {
        let (db, path) = temp_db();
        db.put(b"key1", b"value1").await.unwrap();
        drop(db);

        let db2 = RocksDBEngine::open(&path).unwrap();
        assert_eq!(db2.get(b"key1").await.unwrap(), Some(b"value1".to_vec()));

        std::fs::remove_dir_all(&path).ok();
    }

    #[tokio::test]
    async fn test_column_family_isolation() {
        let (db, path) = temp_db();
        let cf_blocks = db.cf_handle(CF_BLOCKS).unwrap();
        db.db.put_cf(cf_blocks, b"key1", b"block_value").unwrap();

        // 通过 StorageEngine trait 读取的是 CF_STATE
        assert_eq!(db.get(b"key1").await.unwrap(), None);

        std::fs::remove_dir_all(&path).ok();
    }

    #[tokio::test]
    async fn test_write_batch() {
        let (db, path) = temp_db();
        let items = (0..10)
            .map(|i| (format!("key{}", i).into_bytes(), Some(format!("value{}", i).into_bytes())))
            .collect();
        db.write_batch(items).await.unwrap();

        for i in 0..10 {
            assert_eq!(
                db.get(format!("key{}", i).as_bytes()).await.unwrap(),
                Some(format!("value{}", i).into_bytes())
            );
        }

        std::fs::remove_dir_all(&path).ok();
    }
}
