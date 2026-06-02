# Phase 02: 跨语言错误体系

## 目标

建立贯穿 Rust 全 workspace 的统一错误类型 `KilnchainError`，并实现到 Python 异常的自动映射。这是后续所有 FFI 调用的基础设施，必须在任何业务代码之前落地。

---

## 交付物清单

### Rust 侧

| 文件 | 说明 |
|------|------|
| `crates/kilnchain-core/src/error.rs` | `KilnchainError` enum 定义 |
| `crates/kilnchain-core/src/lib.rs` | 导出 `error` 模块 |
| `crates/kilnchain-py/src/error.rs` | `From<KilnchainError> for pyo3::PyErr` 实现 |
| `crates/kilnchain-py/src/lib.rs` | 在 `_internal` 模块注册自定义异常类型 |

### 测试

| 文件 | 说明 |
|------|------|
| `crates/kilnchain-core/src/error.rs` (内联 `#[cfg(test)]`) | Rust 单元测试：错误消息格式化、变体构造 |
| `crates/kilnchain-py/src/error.rs` (内联 `#[cfg(test)]`) | Rust 侧测试 PyErr 转换（使用 `Python::with_gil`） |
| `src/tests/unit/test_exceptions.py` | Python 侧测试：捕获 Rust 抛出的各类异常 |

---

## 核心代码规格

### KilnchainError (kilnchain-core)

```rust
use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum KilnchainError {
    #[error("cryptographic operation failed: {0}")]
    Crypto(String),

    #[error("serialization error: {0}")]
    Serialization(String),

    #[error("storage engine error: {0}")]
    Storage(String),

    #[error("invalid parameter: {0}")]
    InvalidParameter(String),

    #[error("state root mismatch: expected {expected}, got {actual}")]
    StateRootMismatch { expected: String, actual: String },
}
```

### Python 异常映射 (kilnchain-py)

| KilnchainError 变体 | Python 异常类型 | 理由 |
|---------------------|----------------|------|
| `InvalidParameter` | `pyo3::exceptions::PyValueError` | 用户输入错误 |
| `Crypto` | `pyo3::exceptions::PyRuntimeError` | 运行时密码学失败 |
| `Serialization` | `pyo3::exceptions::PyValueError` | 数据格式错误 |
| `Storage` | `pyo3::exceptions::PyRuntimeError` | IO/数据库错误 |
| `StateRootMismatch` | `pyo3::exceptions::PyRuntimeError` | 状态校验失败 |

### 自定义异常注册

在 `_internal` 模块中额外注册一个泛型异常：

```rust
m.add("KilnchainError", m.py().get_type::<pyo3::exceptions::PyRuntimeError>())?;
```

（后续可升级为真正的自定义异常类，先用 RuntimeError 代理。）

---

## 验收标准（必须全部通过）

- [ ] `cargo test -p kilnchain-core -p kilnchain-py` 全部通过
- [ ] Python 侧 `from kilnchain._internal import KilnchainError` 成功
- [ ] `InvalidParameter` 能被 Python `except ValueError` 捕获
- [ ] `Crypto` / `Storage` 能被 Python `except RuntimeError` 捕获
- [ ] `StateRootMismatch` 的错误消息包含 `expected` 和 `got` 的具体值
- [ ] 使用 `thiserror` 派生，不出现手写的 `Display` 实现

---

## 预计工时

0.5 ~ 1 天

---

## 前置依赖

Phase 01: 最小可编译工程骨架

---

## 下一步

Phase 03: 密码学哈希原语
