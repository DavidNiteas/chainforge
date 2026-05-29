// chainforge-storage: KV 存储抽象与实现

pub mod memory;
pub mod traits;

pub use memory::InMemoryStorage;
pub use traits::{BatchWrite, Snapshot, Snapshotable, StorageEngine};
