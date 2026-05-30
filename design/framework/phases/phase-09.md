# Phase 09: PyO3 完整绑定层

## 目标

将所有 Rust 核心功能通过 PyO3 暴露给 Python，确保 GIL 正确释放、异常正确传递、异步方法可用。

---

## 交付物清单

### Rust 侧源码

| 文件 | 说明 |
|------|------|
| `crates/chainforge-py/src/lib.rs` | `_internal` 模块注册所有类 |
| `crates/chainforge-py/src/types.rs` | `PyTransaction`, `PyBlockHeader`, `PyMerkleTree` |
| `crates/chainforge-py/src/crypto.rs` | `PySecretKey`, `PyPublicKey`, `PySignature` |
| `crates/chainforge-py/src/storage.rs` | `PyInMemoryStorage`, `PyRocksDB` |

### Python 侧测试

| 文件 | 说明 |
|------|------|
| `src/tests/unit/test_py_merkle.py` | Python 侧构建 Merkle 树、验证证明 |
| `src/tests/unit/test_py_crypto.py` | 签名/验签、密钥生成 |
| `src/tests/unit/test_py_storage.py` | 同步/异步存储读写 |
| `src/tests/unit/test_py_types.py` | Transaction/BlockHeader 构造与属性读取 |

---

## 核心代码规格

### GIL 管理规则

所有以下操作必须用 `py.allow_threads(|| { ... })` 包裹：
- `MerkleTree::root()`（叶子数 > 100 时）
- `Transaction::hash()`
- `Transaction::recover_sender()`
- `SecretKey::sign()` / `PublicKey::verify()`
- 所有存储 `get` / `put` / `write_batch`

### 返回值规范

- 字节数据一律返回 `PyBytes`（`Bound<'py, PyBytes>`），禁止返回 `Vec<u8>` 或 `PyList`
- 可选值使用 `Option<Bound<'py, PyBytes>>`
- 布尔值直接返回 `bool`

### 异步方法示例

```rust
use pyo3_async_runtimes::tokio::future_into_py;

#[pymethods]
impl PyRocksDB {
    fn get<'py>(&self, py: Python<'py>, key: Vec<u8>) -> PyResult<Bound<'py, PyAny>> {
        let engine = self.engine.clone();
        future_into_py(py, async move {
            let result = engine.get(&key).await
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
            Python::with_gil(|py| {
                Ok(result.map(|v| PyBytes::new(py, &v).into_py(py)))
            })
        })
    }
}
```

### Panic 隔离

所有 `#[pyfunction]` 和 `#[pymethods]` 的公共入口需用 `std::panic::catch_unwind` 包裹，将 panic 转换为 `PyRuntimeError`。

---

## 验收标准（必须全部通过）

- [ ] `pixi run dev-build` 成功
- [ ] `pixi run test-py` 全部通过
- [ ] `PyMerkleTree.root()` 返回 `bytes` 类型（`isinstance(root, bytes)`）
- [ ] `PyMerkleTree.proof(index)` 返回的对象可在 Python 侧正确反序列化
- [ ] 并发测试：在 4 线程中同时计算 Merkle root，耗时与单线程相近（验证 GIL 释放）
- [ ] Rust panic 被捕获：故意触发 panic 的测试函数在 Python 侧抛出 `RuntimeError` 而非进程崩溃
- [ ] 异步存储测试：`await db.put(key, value)` + `await db.get(key)` 往返正确

---

## 预计工时

3 ~ 4 天

---

## 前置依赖

Phase 02: 跨语言错误体系（异常映射）
Phase 03~08: 所有 Rust 核心功能完成

---

## 下一步

Phase 10: Python API 与类型校验层
