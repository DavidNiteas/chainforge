# Phase 08: RocksDB 集成与缓存

## 目标

实现基于 RocksDB 的存储引擎 `RocksDBEngine`，支持列族隔离和 LRU 缓存层。解决跨平台编译问题，并提供 `storage-mem` feature 作为 fallback。

---

## 交付物清单

### 源码

| 文件 | 说明 |
|------|------|
| `crates/chainforge-storage/src/lib.rs` | 条件编译导出 `rocksdb` 模块 |
| `crates/chainforge-storage/src/rocksdb.rs` | `RocksDBEngine` 实现 |
| `crates/chainforge-storage/src/cache.rs` | `LRUCache` 包装层（可选，可先用 `lru` crate） |

### Cargo.toml 变更

```toml
[features]
default = ["rocksdb-backend"]
rocksdb-backend = ["rocksdb"]

[dependencies]
rocksdb = { version = "0.23", optional = true }
lru = "0.12"  # 用于缓存层
```

### 测试

| 文件 | 说明 |
|------|------|
| `crates/chainforge-storage/src/rocksdb.rs` (内联测试) | 持久化、列族隔离测试 |

---

## 核心代码规格

### 列族定义

```rust
pub const CF_META: &str = "meta";
pub const CF_BLOCKS: &str = "blocks";
pub const CF_STATE: &str = "state";
pub const CF_INDEX: &str = "index";

pub const ALL_CFS: &[&str] = &[CF_META, CF_BLOCKS, CF_STATE, CF_INDEX];
```

### RocksDBEngine

```rust
pub struct RocksDBEngine {
    db: rocksdb::DB,
}

impl RocksDBEngine {
    pub fn open(path: &std::path::Path) -> Result<Self, ChainforgeError> {
        let mut opts = rocksdb::Options::default();
        opts.create_if_missing(true);
        opts.create_missing_column_families(true);
        let db = rocksdb::DB::open_cf(&opts, path, ALL_CFS)
            .map_err(|e| ChainforgeError::Storage(e.to_string()))?;
        Ok(Self { db })
    }
}
```

### 缓存层（可选，可留接口）

```rust
pub struct CachedStorage<E: StorageEngine> {
    inner: E,
    cache: Arc<RwLock<LruCache<Vec<u8>, Vec<u8>>>>,
}
```

本阶段可先实现 `RocksDBEngine` 本身，缓存层作为增强在测试稳定后追加。

---

## 验收标准（必须全部通过）

- [ ] `cargo test -p chainforge-storage --all-features` 全部通过
- [ ] `cargo check --workspace` 在 Windows 上通过（或 `storage-mem` feature 能绕过 RocksDB）
- [ ] 持久化测试：临时目录 → 写入 → `drop(db)` → 重新 `open` → 读取数据一致
- [ ] 列族隔离：向 `blocks` 写入的 key，在 `state` 中 `get` 返回 `None`
- [ ] `write_batch` 原子性：RocksDB WriteBatch 模式
- [ ] 关闭后资源释放：连续打开/关闭 1000 次不泄漏文件句柄

---

## 预计工时

2 ~ 3 天

---

## 前置依赖

Phase 07: 存储层 Trait + 内存后端（确立接口契约）

---

## 下一步

Phase 09: PyO3 完整绑定层
