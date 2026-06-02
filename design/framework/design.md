# 基于 Rust + PyO3 的专业区块链库工程方案

## 1. 项目概述与架构哲学

本项目旨在构建一个高性能、内存安全的区块链核心库，以 Rust 实现底层密码学、共识原语与数据结构，通过 PyO3 提供符合 Python 生态习惯的绑定层。目标用户为数据科学家、量化研究员及需要快速原型验证的区块链开发者。

**核心原则：**
- **零成本抽象**：Python 层仅做 API 编排，所有计算密集型任务下沉至 Rust
- **类型安全边界**：Rust 侧使用强类型系统，Python 侧通过 Type Hints + Stub 文件保证静态检查
- **错误隔离**：Rust panic 必须在边界层捕获并转换为 Python 异常，禁止跨语言传播未处理 panic
- **GIL 释放**：所有耗时操作（哈希计算、签名验证、状态根重建）必须显式释放 GIL

---

## 2. 技术选型与依赖策略

### 2.1 Rust 核心依赖

| 功能域 | 选型 | 理由 |
|--------|------|------|
| Python 绑定 | `pyo3` (v0.23+) | 原生 `async` 支持，`Py<T>` GIL 无关类型，避免 GIL 瓶颈 |
| 异步运行时 | `tokio` (v1.x) | 与 pyo3-asyncio 兼容，支持多线程 IO |
| 序列化 | `serde` + `bincode` / `serde_json` | 跨语言一致性，bincode 用于内部存储，JSON 用于 RPC |
| 密码学 | `ring` (通用) + `secp256k1` (链特定) | ring 提供 SHA-256/AES，secp256k1 提供 ECDSA |
| 哈希树 | `rs_merkle` 或自研 | 根据性能需求选择，自研需配套 proptest |
| 存储引擎 | `rocksdb` (via `rust-rocksdb`) | LSM-Tree 适合链式追加写，支持快照 |
| 错误处理 | `thiserror` + `anyhow` | thiserror 用于库错误定义，anyhow 限于内部逻辑 |
| 属性测试 | `proptest` | 生成随机区块/交易结构，验证状态一致性 |
| 基准测试 | `criterion` | 回归测试性能退化 |

### 2.2 Python 侧依赖

| 功能域 | 选型 |
|--------|------|
| 类型系统 | `typing` (内置) + `mypy` (静态检查) |
| 运行时校验 | `pydantic` (v2) | 用于输入参数校验与 JSON Schema 生成 |
| 异步支持 | `asyncio` (标准库) |
| 测试框架 | `pytest` + `pytest-asyncio` + `hypothesis` |
| 内存分析 | `memray` (可选) | 检测 FFI 边界内存泄漏 |
| 构建后端 | `maturin` (v1.7+) | PyO3 官方推荐，支持 PEP 517 |

---

## 3. 开发环境搭建（Pixi）

Pixi 提供跨平台的可复现环境，优于纯 pip 的依赖解析。项目采用 Monorepo 结构，Rust 与 Python 源码共存。

### 3.1 目录结构

```
kilnchain/
├── Cargo.toml                 # Rust workspace 根
├── pixi.toml                  # Pixi 项目配置
├── pyproject.toml             # Python 包元数据 + maturin 配置
├── rust-toolchain.toml        # Rust 工具链锁定
├── src/                       # Python 源码包
│   ├── kilnchain/
│   │   ├── __init__.py
│   │   ├── types.py           # Pydantic 模型与类型别名
│   │   ├── client.py          # 高层 Pythonic API
│   │   └── py.typed           # PEP 561 标记
│   └── tests/
│       ├── unit/
│       ├── integration/
│       └── conftest.py
├── crates/
│   ├── kilnchain-core/       # 纯 Rust 核心：区块、交易、Merkle 树
│   ├── kilnchain-crypto/     # 密码学原语封装
│   ├── kilnchain-storage/    # KV 存储抽象与 RocksDB 实现
│   └── kilnchain-py/         # PyO3 绑定层（唯一依赖 pyo3 的 crate）
└── .github/
    └── workflows/
        └── ci.yml
```

### 3.2 Pixi 配置 (`pixi.toml`)

```toml
[project]
name = "kilnchain"
version = "0.1.0"
description = "High-performance blockchain primitives with Python bindings"
authors = ["Your Name <you@example.com>"]
channels = ["conda-forge"]
platforms = ["linux-64", "osx-64", "osx-arm64", "win-64"]

[dependencies]
python = ">=3.10,<3.13"
pip = "*"

[pypi-dependencies]
maturin = ">=1.7.0"
pytest = ">=8.0.0"
pytest-asyncio = ">=0.23.0"
hypothesis = ">=6.100.0"
mypy = ">=1.10.0"
pydantic = ">=2.7.0"
memray = ">=1.12.0"

[tasks]
# 安装 Rust 工具链（通过 rustup，pixi 不直接管理 Rust，需外部安装）
install-rust = { cmd = "rustup show", description = "Verify Rust toolchain" }

# 构建开发版本（editable install）
dev-build = { cmd = "maturin develop --release", depends-on = ["install-rust"] }

# 运行 Rust 测试套件
test-rust = { cmd = "cargo test --workspace", depends-on = ["install-rust"] }

# 运行 Python 测试套件
test-py = { cmd = "pytest src/tests -v --tb=short", depends-on = ["dev-build"] }

# 全量测试
test = { depends-on = ["test-rust", "test-py"] }

# 静态类型检查
typecheck = { cmd = "mypy src/kilnchain" }

# 格式化（双端）
fmt = { cmd = "cargo fmt && ruff format src/" }

# 基准测试
bench = { cmd = "cargo bench --workspace" }

[feature.dev.dependencies]
ruff = "*"
pre-commit = "*"
```

### 3.3 Rust 工具链 (`rust-toolchain.toml`)

```toml
[toolchain]
channel = "stable"
components = ["rustfmt", "clippy", "llvm-tools-preview"]
targets = ["x86_64-unknown-linux-gnu", "x86_64-pc-windows-msvc", "aarch64-apple-darwin"]
```

### 3.4 Python 包配置 (`pyproject.toml`)

```toml
[build-system]
requires = ["maturin>=1.7.0"]
build-backend = "maturin"

[project]
name = "kilnchain"
version = "0.1.0"
requires-python = ">=3.10"
classifiers = [
    "Programming Language :: Rust",
    "Programming Language :: Python :: Implementation :: CPython",
    "Topic :: Security :: Cryptography",
]

[tool.maturin]
manifest-path = "crates/kilnchain-py/Cargo.toml"
module-name = "kilnchain._internal"
python-source = "src"

[tool.pytest.ini_options]
asyncio_mode = "auto"
testpaths = ["src/tests"]
```

---

## 4. 核心功能模块设计

### 4.1 密码学原语 (`kilnchain-crypto`)

- **哈希**：SHA-256, Keccak-256, RIPEMD-160（通过 `ring` 与自研轻量封装）
- **数字签名**：Secp256k1 ECDSA, Ed25519（公钥恢复、签名聚合基础结构）
- **Merkle 树**：二叉 SHA-256 Merkle Tree，支持稀疏 Merkle Tree（SMT）扩展接口
- **密钥派生**：PBKDF2, BIP-39 助记词生成（可选模块）

### 4.2 核心数据结构 (`kilnchain-core`)

- **交易 (Transaction)**：RLP / 自定义二进制编码，支持 EIP-155 式链 ID 隔离
- **区块头 (BlockHeader)**：包含难度、nonce、时间戳、extra_data（字节长度限制 32 bytes）
- **区块 (Block)**：区块头 + 交易列表 + 叔块头列表
- **账户状态 (AccountState)**：nonce, balance, storage_root, code_hash（四元组）
- **状态树 (StateTrie)**：基于 Merkle Patricia Trie 的抽象，后端可插拔

### 4.3 存储层 (`kilnchain-storage`)

- **抽象 trait**：`StorageEngine`, `BatchWrite`, `Snapshot`
- **默认实现**：RocksDB 封装，支持列族（Column Family）隔离：元数据、区块体、状态、索引
- **缓存层**：LRU 缓存热点状态节点，减少 RocksDB 读放大

### 4.4 网络与序列化（预留接口）

- **RLP 编解码器**：零拷贝解析，支持流式解码大区块
- **JSON-RPC 类型**：仅定义数据结构，不实现网络 IO，保持库的中立性

---

## 5. Rust 实现层架构

### 5.1 错误体系（跨语言一致）

```rust
// crates/kilnchain-core/src/error.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum KilnchainError {
    #[error("cryptographic operation failed: {0}")]
    Crypto(String),
    
    #[error("serialization error: {0}")]
    Serialization(#[from] bincode::Error),
    
    #[error("storage engine error: {0}")]
    Storage(String),
    
    #[error("invalid parameter: {0}")]
    InvalidParameter(String),
    
    #[error("state root mismatch: expected {expected}, got {actual}")]
    StateRootMismatch { expected: String, actual: String },
}

// 实现 PyO3 自动转换
impl std::convert::From<<KilnchainError> for pyo3::PyErr {
    fn from(err: KilnchainError) -> pyo3::PyErr {
        match err {
            KilnchainError::Crypto(_) => pyo3::exceptions::PyRuntimeError::new_err(err.to_string()),
            KilnchainError::InvalidParameter(_) => pyo3::exceptions::PyValueError::new_err(err.to_string()),
            _ => pyo3::exceptions::PyException::new_err(err.to_string()),
        }
    }
}
```

### 5.2 核心类型示例（交易）

```rust
// crates/kilnchain-core/src/tx.rs
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Transaction {
    pub nonce: u64,
    pub gas_price: u128,
    pub gas_limit: u64,
    pub to: Option<[u8; 20]>,  // None for contract creation
    pub value: u128,
    pub data: Vec<u8>,
    pub v: u64,
    pub r: [u8; 32],
    pub s: [u8; 32],
}

impl Transaction {
    /// 计算交易哈希（RLP 编码后 Keccak-256）
    pub fn hash(&self) -> [u8; 32] {
        // 实现...
    }
    
    /// 从裸字节恢复发送地址（ECDSA 公钥恢复）
    pub fn recover_sender(&self) -> Result<[u8; 20], KilnchainError> {
        // 实现...
    }
}
```

### 5.3 Merkle 树实现（属性测试驱动）

```rust
// crates/kilnchain-core/src/merkle.rs
pub struct MerkleTree {
    leaves: Vec<[u8; 32]>,
    layers: Vec<Vec<[u8; 32]>>,
}

impl MerkleTree {
    pub fn new(leaves: Vec<[u8; 32]>) -> Self { /* ... */ }
    pub fn root(&self) -> [u8; 32] { /* ... */ }
    pub fn proof(&self, index: usize) -> Option<MerkleProof> { /* ... */ }
    pub fn verify(root: &[u8; 32], leaf: &[u8; 32], proof: &MerkleProof) -> bool { /* ... */ }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn merkle_root_deterministic(leaves in prop::collection::vec(any::<[u8; 32]>(), 1..1000)) {
            let tree1 = MerkleTree::new(leaves.clone());
            let tree2 = MerkleTree::new(leaves);
            assert_eq!(tree1.root(), tree2.root());
        }
    }
}
```

---

## 6. PyO3 绑定层设计 (`kilnchain-py`)

### 6.1 模块暴露与命名空间

```rust
// crates/kilnchain-py/src/lib.rs
use pyo3::prelude::*;

mod crypto;
mod types;
mod storage;

#[pymodule]
fn _internal(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<types::PyTransaction>()?;
    m.add_class::<types::PyBlockHeader>()?;
    m.add_class::<types::PyMerkleTree>()?;
    m.add_class::<crypto::PySecp256k1>()?;
    m.add_class::<storage::PyRocksDB>()?;
    
    // 注册异常类型
    m.add("KilnchainError", m.py().get_type::<pyo3::exceptions::PyRuntimeError>())?;
    Ok(())
}
```

### 6.2 类型转换与 GIL 管理

```rust
// crates/kilnchain-py/src/types.rs
use pyo3::prelude::*;
use kilnchain_core::{Transaction, BlockHeader};
use kilnchain_crypto::MerkleTree;

#[pyclass(name = "Transaction", frozen)]
pub struct PyTransaction {
    inner: Transaction,  // 不可变结构，支持多线程共享
}

#[pymethods]
impl PyTransaction {
    #[new]
    fn py_new(
        nonce: u64,
        gas_price: u128,
        to: Option<[u8; 20]>,
        value: u128,
        data: Vec<u8>,
    ) -> PyResult<Self> {
        Ok(Self {
            inner: Transaction {
                nonce,
                gas_price,
                gas_limit: 21_000, // 默认
                to,
                value,
                data,
                v: 0, r: [0; 32], s: [0; 32],
            },
        })
    }

    /// 计算哈希，显式释放 GIL 允许其他 Python 线程并行
    fn hash<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
        py.allow_threads(|| {
            let hash = self.inner.hash();
            PyBytes::new(py, &hash)
        })
    }

    #[getter]
    fn nonce(&self) -> u64 { self.inner.nonce }

    #[getter]
    fn to(&self) -> Option<[u8; 20]> { self.inner.to }

    /// 类方法：从 RLP 编码字节恢复
    #[classmethod]
    fn from_rlp(_cls: &Bound<'_, PyType>, data: &[u8]) -> PyResult<Self> {
        // RLP 解码逻辑...
    }
}
```

### 6.3 异步支持（pyo3 0.23+ 原生 async）

```rust
// crates/kilnchain-py/src/storage.rs
use pyo3::prelude::*;
use pyo3_async_runtimes::tokio::future_into_py;
use kilnchain_storage::RocksDBEngine;

#[pyclass]
pub struct PyRocksDB {
    engine: RocksDBEngine,
    runtime: tokio::runtime::Runtime,
}

#[pymethods]
impl PyRocksDB {
    fn get_block<'py>(&self, py: Python<'py>, hash: [u8; 32]) -> PyResult<<Bound<'py, PyAny>> {
        let engine = self.engine.clone();
        future_into_py(py, async move {
            let block = engine.get_block(&hash).await
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
            Python::with_gil(|py| {
                // 转换为 Python 对象
                Ok(PyBlock::from_inner(py, block)?.into_py(py))
            })
        })
    }
}
```

---

## 7. Python API 层设计

Python 层不做重逻辑，仅提供：
1. **类型别名与 Pydantic 校验**：确保传入 Rust 前的数据格式合法
2. **上下文管理器**：自动关闭 RocksDB 句柄
3. **异常重导出**：统一异常捕获体验

```python
# src/kilnchain/__init__.py
from kilnchain._internal import (
    Transaction,
    BlockHeader,
    MerkleTree,
    Secp256k1,
    RocksDB,
    KilnchainError,
)

__all__ = [
    "Transaction", "BlockHeader", "MerkleTree",
    "Secp256k1", "RocksDB", "KilnchainError",
    "BlockChain", "Account",
]

# src/kilnchain/types.py
from pydantic import BaseModel, Field
from typing import Optional

class TxInput(BaseModel):
    nonce: int = Field(ge=0, le=2**64-1)
    gas_price: int
    to: Optional[bytes] = Field(default=None, max_length=20)
    value: int = Field(default=0, ge=0)
    data: bytes = b""

# src/kilnchain/client.py（高层封装）
from contextlib import contextmanager
from kilnchain._internal import RocksDB as _RocksDB

@contextmanager
def open_db(path: str):
    db = _RocksDB(path)
    try:
        yield db
    finally:
        db.close()
```

---

## 8. 测试策略与覆盖方案

### 8.1 Rust 侧测试矩阵

| 测试类型 | 工具 | 覆盖目标 | 执行命令 |
|----------|------|----------|----------|
| 单元测试 | `cargo test` | 所有 public API，边界条件（空 Merkle 树、最大深度） | `cargo test --workspace` |
| 文档测试 | `rustdoc --test` | 代码示例可编译执行 | 内置于 `cargo test` |
| 属性测试 | `proptest` | 随机交易序列的状态一致性、哈希碰撞抵抗 | `cargo test --features proptest` |
| 模糊测试 | `cargo-fuzz` + `libfuzzer-sys` | RLP 解码器、网络包解析 | `cargo fuzz run rlp_decode` |
| 基准测试 | `criterion` | 签名验证吞吐量、Merkle 根计算延迟 | `cargo bench` |

**关键测试场景：**

```rust
// crates/kilnchain-core/src/tests/state_tests.rs
#[test]
fn test_empty_merkle_root() {
    let tree = MerkleTree::new(vec![]);
    // 空树根应为固定值（如 Ethereum 的空 trie root）
    assert_eq!(tree.root(), EMPTY_ROOT);
}

#[test]
fn test_signature_roundtrip() {
    let sk = SecretKey::random();
    let msg = b"hello kilnchain";
    let sig = sk.sign(msg);
    let pk = sk.public_key();
    assert!(pk.verify(msg, &sig));
    assert!(!pk.verify(b"wrong message", &sig));
}

proptest! {
    #[test]
    fn prop_tx_serialization_roundtrip(tx in arb_transaction()) {
        let encoded = rlp_encode(&tx);
        let decoded = Transaction::from_rlp(&encoded).unwrap();
        assert_eq!(tx, decoded);
    }
}
```

### 8.2 Python 侧测试矩阵

| 测试类型 | 工具 | 覆盖目标 |
|----------|------|----------|
| 单元测试 | `pytest` | Python API 边界、类型转换、异常抛出 |
| 异步测试 | `pytest-asyncio` | 异步存储读写、并发签名验证 |
| 属性测试 | `hypothesis` | 随机字节序列的编解码不变性 |
| 内存安全 | `memray` + `pytest-memray` | FFI 调用无内存泄漏，大对象正确释放 |
| 类型检查 | `mypy` | Stub 文件与实际实现一致 |

```python
# src/tests/unit/test_crypto.py
import pytest
from kilnchain import Secp256k1, KilnchainError

def test_sign_and_recover():
    sk = Secp256k1.generate_key()
    msg = b"test message"
    sig = sk.sign(msg)
    pk = sk.public_key()
    assert pk.verify(msg, sig)
    
    # 错误消息应抛出异常
    with pytest.raises(KilnchainError):
        pk.verify(b"wrong", sig)

def test_invalid_key_length():
    with pytest.raises(ValueError):
        Secp256k1.from_bytes(b"too short")

# src/tests/integration/test_storage.py
import pytest
from kilnchain import open_db
import tempfile
import os

@pytest.mark.asyncio
async def test_db_persistence():
    with tempfile.TemporaryDirectory() as tmp:
        with open_db(tmp) as db:
            header = BlockHeader(
                parent_hash=b'\x00'*32,
                number=1,
                timestamp=1234567890
            )
            await db.put_header(header.hash(), header)
        
        # 重新打开验证持久化
        with open_db(tmp) as db:
            retrieved = await db.get_header(header.hash())
            assert retrieved.number == 1
```

### 8.3 跨语言集成测试

```python
# src/tests/integration/test_rust_python_parity.py
"""
验证 Rust 与 Python 实现（如有）的结果一致性，
或验证 Rust 内部状态与 Python 视图同步。
"""
from kilnchain import MerkleTree
import hashlib

def test_merkle_root_against_reference():
    leaves = [hashlib.sha256(str(i).encode()).digest() for i in range(100)]
    tree = MerkleTree(leaves)
    root = tree.root()
    
    # 与已知参考实现对比（如 Ethereum 测试向量）
    assert len(root) == 32
    assert isinstance(root, bytes)
```

---

## 9. CI/CD 与发布流程

### 9.1 GitHub Actions 工作流 (`.github/workflows/ci.yml`)

```yaml
name: CI

on: [push, pull_request]

jobs:
  rust-checks:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy
      - run: cargo fmt -- --check
      - run: cargo clippy --workspace -- -D warnings
      - run: cargo test --workspace --all-features
      - run: cargo bench -- --no-run  # 确保基准测试可编译

  python-checks:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
        python-version: ["3.10", "3.11", "3.12"]
    steps:
      - uses: actions/checkout@v4
      - uses: prefix-dev/setup-pixi@v0.8.0
        with:
          pixi-version: v0.25.0
          cache: true
      - run: pixi run dev-build
      - run: pixi run test-py
      - run: pixi run typecheck

  build-wheels:
    needs: [rust-checks, python-checks]
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-13, macos-14, windows-latest]
    steps:
      - uses: actions/checkout@v4
      - uses: PyO3/maturin-action@v1
        with:
          target: ${{ matrix.target }}
          args: --release --out dist
          sccache: 'true'
      - uses: actions/upload-artifact@v4
        with:
          name: wheels-${{ matrix.os }}
          path: dist
```

### 9.2 发布流程

1. **版本 bump**：更新 `Cargo.toml` workspace version、`pyproject.toml` version、`pixi.toml` version，三者必须同步
2. **Changelog 生成**：使用 `git-cliff` 或 `conventional commits` 规范自动生成
3. **GitHub Release**：CI 自动构建多平台 wheel 并上传至 PyPI
4. **Pixi 包发布**（可选）：若需 conda 生态支持，通过 `rattler-build` 构建 conda 包

---

## 10. 性能优化与内存安全

### 10.1 GIL 策略

- **禁止在 Rust 侧长时间持有 GIL**：所有超过 1ms 的操作必须使用 `py.allow_threads(|| { ... })`
- **返回字节数据**：优先返回 `PyBytes` 而非 `PyList[int]`，减少 Python 对象分配
- **零拷贝视图**：对于只读大数据（如区块体），使用 `PyBuffer` 协议暴露内存视图，避免 `Vec<u8>` 深拷贝

### 10.2 内存安全红线

- **Panic 隔离**：在 `lib.rs` 顶层使用 `std::panic::catch_unwind` 包裹所有 FFI 入口，panic 转换为 `PyRuntimeError`
- **引用计数审计**：对 `Py<T>` 和 `Bound<'_, T>` 的使用进行 clippy 审查，禁止循环引用
- **Valgrind 测试**：Linux CI 中运行 `valgrind --tool=memcheck` 检测未初始化内存访问

### 10.3 并发模型

- Rust 侧使用 `tokio::runtime::Runtime`（多线程），与 Python `asyncio` 通过 `future_into_py` 桥接
- 共享可变状态（如数据库句柄）使用 `Arc<<tokio::sync::RwLock<T>>`，禁止直接暴露 `&mut` 给 Python

---

## 11. 文档与示例

### 11.1 文档结构

- **Rust API Docs**：`cargo doc --no-deps`，托管于 docs.rs（发布时自动构建）
- **Python API Docs**：使用 `mkdocs` + `mkdocstrings-python` + `griffe`，从 docstring 自动生成
- **教程**：`docs/tutorials/01-quickstart.md` —— 创建私钥、签名交易、计算 Merkle 根
- **架构决策记录 (ADR)**：`docs/adr/` 记录重大设计选择（如为何选择 RocksDB 而非 sled）

### 11.2 Python Docstring 规范

```python
class Transaction:
    """
    Immutable blockchain transaction structure.
    
    Parameters
    ----------
    nonce : int
        Transaction sequence number per sender.
    gas_price : int
        Price in wei per unit of gas.
    
    Examples
    --------
    >>> from kilnchain import Transaction
    >>> tx = Transaction(nonce=0, gas_price=20_000_000_000, to=b'\\x00'*20, value=1000)
    >>> len(tx.hash())
    32
    """
```

---

## 12. 风险与注意事项

| 风险点 | 影响 | 缓解措施 |
|--------|------|----------|
| **PyO3 版本升级 Breaking Change** | 高 | 锁定 `pyo3` 版本至 minor 号（如 `0.23.*`），升级前阅读 Migration Guide |
| **RocksDB 跨平台编译失败** | 高 | CI 覆盖三大平台；Windows 使用 `vcpkg` 或预编译 lib；提供 `storage-mem` feature 作为纯内存后端 fallback |
| **GIL 释放导致的 Python 对象悬空** | 中 | 禁止在 `allow_threads` 闭包中捕获 `Bound<'_, PyAny>`；仅传递原始数据或 `Py<T>` |
| **Rust panic 跨 FFI 边界** | 高 | 所有 `#[pyfunction]` 和 `#[pymethods]` 入口强制 `catch_unwind`；CI 集成 `panic=abort` 测试 |
| **Python 循环引用 + Rust Arc = 内存泄漏** | 中 | 避免在 Rust 结构中存储 `PyObject` 引用；必要时使用 `weakref` 回调释放 Rust 资源 |
| **Maturin 与 Pixi 环境冲突** | 低 | 使用 `pixi run maturin develop` 而非全局 maturin；`pyproject.toml` 中 `tool.maturin.manifest-path` 必须正确指向 binding crate |

---

## 附录：快速启动命令

```bash
# 1. 克隆并进入项目
git clone https://github.com/your-org/kilnchain.git && cd kilnchain

# 2. 安装 Pixi 环境（自动安装 Python 依赖）
pixi install

# 3. 构建并安装到当前 Pixi 环境（editable）
pixi run dev-build

# 4. 运行全量测试
pixi run test

# 5. 运行基准测试
pixi run bench

# 6. 静态类型检查
pixi run typecheck
```

---

**文档版本：** 0.1.0  
**最后更新：** 2026-05-30  
**维护者：** 工程团队