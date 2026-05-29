// chainforge-storage: KV 存储抽象与实现

pub mod cache;
pub mod memory;
pub mod traits;

#[cfg(feature = "rocksdb-backend")]
pub mod rocksdb;

pub use cache::CachedStorage;
pub use memory::InMemoryStorage;
pub use traits::{BatchWrite, Snapshot, Snapshotable, StorageEngine};

#[cfg(feature = "rocksdb-backend")]
pub use rocksdb::RocksDBEngine;
