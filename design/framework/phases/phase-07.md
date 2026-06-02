# Phase 07: 存储层 Trait + 内存后端

## 目标

定义存储抽象 trait，并实现一个纯内存后端 `InMemoryStorage`。此阶段不涉及 RocksDB，目的是确立存储接口契约，并为上层逻辑提供可测试的 mock。

---

## 交付物清单

### 源码

| 文件 | 说明 |
|------|------|
| `crates/kilnchain-storage/src/lib.rs` | 导出 `traits`, `memory` 模块 |
| `crates/kilnchain-storage/src/traits.rs` | `StorageEngine`, `BatchWrite`, `Snapshot` |
| `crates/kilnchain-storage/src/memory.rs` | `InMemoryStorage` 实现 |

### 测试

| 文件 | 说明 |
|------|------|
| `crates/kilnchain-storage/src/memory.rs` (内联测试) | CRUD、Batch、Snapshot 测试 |

---

## 核心代码规格

### Trait 定义

```rust
use async_trait::async_trait;
use kilnchain_core::KilnchainError;

#[async_trait]
pub trait StorageEngine: Send + Sync {
    async fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, KilnchainError>;
    async fn put(&self, key: &[u8], value: &[u8]) -> Result<(), KilnchainError>;
    async fn delete(&self, key: &[u8]) -> Result<(), KilnchainError>;
    async fn contains(&self, key: &[u8]) -> Result<bool, KilnchainError>;
}

#[async_trait]
pub trait BatchWrite: StorageEngine {
    async fn write_batch(&self, items: Vec<(Vec<u8>, Option<Vec<u8>>)>) -> Result<(), KilnchainError>;
    // None value = delete
}

pub trait Snapshot: Send + Sync {
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, KilnchainError>;
}

#[async_trait]
pub trait Snapshotable: StorageEngine {
    async fn snapshot(&self) -> Result<Box<dyn Snapshot>, KilnchainError>;
}
```

### InMemoryStorage

```rust
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub struct InMemoryStorage {
    data: Arc<RwLock<HashMap<Vec<u8>, Vec<u8>>>>,
}
```

- `put`：写入 `HashMap`
- `get`：读取 `HashMap`
- `delete`：从 `HashMap` 移除
- `write_batch`：在单个 `write` 锁内原子执行所有操作
- `snapshot`：克隆当前 `HashMap` 状态到新的只读结构

---

## 验收标准（必须全部通过）

- [ ] `cargo test -p kilnchain-storage` 全部通过
- [ ] `put` → `get` 往返正确
- [ ] `delete` 后 `get` 返回 `None`
- [ ] `write_batch` 原子性：同一 batch 内 100 条写入，要么全成功，要么全失败（模拟失败场景）
- [ ] `snapshot` 隔离性：创建 snapshot 后修改数据，snapshot 读取仍为旧值
- [ ] `InMemoryStorage` 实现 `Send + Sync`，可跨线程共享

---

## 预计工时

1 ~ 2 天

---

## 前置依赖

Phase 02: 跨语言错误体系（使用 `KilnchainError`）

---

## 下一步

Phase 08: RocksDB 集成与缓存
