# Chainforge —— AI 编码代理指南

## 项目概览

**Chainforge** 是一个计划中的高性能区块链核心库，目标是用 **Rust** 实现底层密码学、共识原语与数据结构，并通过 **PyO3** 向 Python 提供符合 Python 生态习惯的绑定层。目标用户为数据科学家、量化研究员及需要快速原型验证的区块链开发者。

**当前状态：** Phase 01（最小可编译工程骨架）已完成。Rust workspace、Pixi 环境、Maturin 配置及空 crate 均已就绪，`cargo check --workspace`、`pixi run dev-build` 与 `python -c "import chainforge._internal"` 均通过。

---

## 现有文件清单

| 路径 | 说明 |
|------|------|
| `AGENTS.md` | 本文件，供 AI 编码代理阅读的项目指南 |
| `README.md` | 项目简介（占位） |
| `Cargo.toml` | Rust workspace 根配置 |
| `rust-toolchain.toml` | Rust 工具链锁定（stable + rustfmt + clippy） |
| `pixi.toml` | Pixi 项目配置与 task 定义 |
| `pyproject.toml` | Python 包元数据 + maturin 配置 |
| `design/design.md` | 747 行的总纲设计文档 |
| `design/phases/phase-01.md` ~ `phase-11.md` | 11 份分阶段实施文档 |
| `crates/chainforge-core/` | 空 crate：区块、交易、Merkle 树 |
| `crates/chainforge-crypto/` | 空 crate：密码学原语 |
| `crates/chainforge-storage/` | 空 crate：KV 存储抽象 |
| `crates/chainforge-py/` | PyO3 绑定层（已暴露空 `_internal` 模块） |
| `src/chainforge/__init__.py` | Python 包入口 |
| `src/chainforge/py.typed` | PEP 561 类型标记 |
| `src/tests/conftest.py` | pytest 共享配置 |
| `src/tests/unit/` | 单元测试目录 |
| `src/tests/integration/` | 集成测试目录 |

**注意：** `design/` 下的文档均使用中文撰写。

---

## 计划中的技术栈

| 层级 | 技术 | 用途 |
|------|------|------|
| 核心实现 | Rust (stable) | 密码学、Merkle 树、存储引擎、序列化 |
| Python 绑定 | PyO3 (v0.23+) | FFI 层，将 Rust 类型暴露给 Python |
| 异步运行时 | Tokio | Rust 侧异步 IO，桥接 Python `asyncio` |
| 序列化 | `serde` + `bincode` / `serde_json` | 内部存储用 bincode，RPC 用 JSON |
| 密码学 | `ring` + `secp256k1` | SHA-256/AES 与 ECDSA |
| 存储 | RocksDB (via `rust-rocksdb`) | LSM-Tree KV 后端，支持列族 |
| 错误处理 | `thiserror` + `anyhow` | Rust 侧结构化错误 |
| 属性测试 | `proptest` | 随机化正确性测试 |
| 基准测试 | `criterion` | 性能回归测试 |
| Python 打包 | `maturin` (v1.7+) | PEP 517 / PyO3 wheel 构建后端 |
| Python 环境管理 | `pixi` | 跨平台可复现的 Python + Rust 开发环境 |
| Python 静态检查 | `mypy` | Type stub 校验 |
| Python 运行时检查 | `pydantic` (v2) | 输入校验与 JSON Schema 生成 |
| Python 测试 | `pytest` + `pytest-asyncio` + `hypothesis` | 单元、异步与属性测试 |

---

## 计划中的目录结构

设计文档提出的目录结构为 Rust workspace + Python package monorepo：

```
chainforge/
├── Cargo.toml                 # Rust workspace 根
├── pixi.toml                  # Pixi 项目配置
├── pyproject.toml             # Python 包元数据 + maturin 配置
├── rust-toolchain.toml        # Rust 工具链锁定
├── src/
│   ├── chainforge/            # Python 源码包
│   │   ├── __init__.py
│   │   ├── types.py           # Pydantic 模型与类型别名
│   │   ├── client.py          # 高层 Pythonic API
│   │   └── py.typed           # PEP 561 标记
│   └── tests/
│       ├── unit/
│       ├── integration/
│       └── conftest.py
├── crates/
│   ├── chainforge-core/       # 纯 Rust：区块、交易、Merkle 树
│   ├── chainforge-crypto/     # 密码学原语
│   ├── chainforge-storage/    # KV 存储抽象 + RocksDB 实现
│   └── chainforge-py/         # PyO3 绑定层（唯一依赖 pyo3 的 crate）
└── .github/
    └── workflows/
        └── ci.yml
```

**当前实际状态：** 上述目录与文件均不存在，仅在 `design.md` 及分阶段文档中做了详细规格说明。

---

## 计划中的构建与开发命令

设计文档规定以下 Pixi task（需在 `pixi.toml` 创建后通过 `pixi run <task>` 执行）：

| 任务 | 命令 | 说明 |
|------|------|------|
| 验证 Rust 工具链 | `pixi run install-rust` | 执行 `rustup show` |
| 可编辑安装 | `pixi run dev-build` | `maturin develop --release` |
| Rust 测试 | `pixi run test-rust` | `cargo test --workspace` |
| Python 测试 | `pixi run test-py` | `pytest src/tests -v --tb=short` |
| 全量测试 | `pixi run test` | 同时运行 Rust 与 Python 测试 |
| 类型检查 | `pixi run typecheck` | `mypy src/chainforge` |
| 格式化 | `pixi run fmt` | `cargo fmt && ruff format src/` |
| 基准测试 | `pixi run bench` | `cargo bench --workspace` |

**计划中的快速启动：**
```bash
pixi install
pixi run dev-build
pixi run test
```

---

## 计划中的 Crate / 模块划分

### Rust crates

1. **`chainforge-core`**
   - Transaction, BlockHeader, Block, AccountState, StateTrie
   - RLP / 自定义二进制编解码器
   - Merkle Tree（二叉 SHA-256）及稀疏 Merkle Tree 扩展接口

2. **`chainforge-crypto`**
   - 哈希：SHA-256, Keccak-256, RIPEMD-160
   - 签名：Secp256k1 ECDSA, Ed25519
   - 密钥派生：PBKDF2, BIP-39（可选）

3. **`chainforge-storage`**
   - Traits：`StorageEngine`, `BatchWrite`, `Snapshot`
   - 默认实现：RocksDB 封装，列族隔离（metadata, blocks, state, index）
   - LRU 缓存层用于热点状态节点

4. **`chainforge-py`**
   - PyO3 模块 `_internal`
   - 暴露：`PyTransaction`, `PyBlockHeader`, `PyMerkleTree`, `PySecp256k1`, `PyRocksDB`
   - 将 Rust 错误转换为 Python 异常
   - 所有耗时操作必须通过 `py.allow_threads(...)` 释放 GIL
   - 异步存储方法通过 `future_into_py` 桥接

### Python 包 (`src/chainforge`)

- **`__init__.py`** —— 从 `_internal` 重导出并定义 `__all__`
- **`types.py`** —— Pydantic 模型（如 `TxInput`），用于跨越 FFI 边界前的输入校验
- **`client.py`** —— 高层便利封装，如 `open_db()` 上下文管理器

---

## 测试策略（计划中）

### Rust 侧

| 测试类型 | 工具 | 目标 | 命令 |
|----------|------|------|------|
| 单元测试 | `cargo test` | 公共 API、边界条件 | `cargo test --workspace` |
| 文档测试 | 内置 | 编译并运行文档示例 | 内置于 `cargo test` |
| 属性测试 | `proptest` | 随机交易序列、状态一致性 | `cargo test --features proptest` |
| 模糊测试 | `cargo-fuzz` + `libfuzzer-sys` | RLP 解码器、网络包解析 | `cargo fuzz run rlp_decode` |
| 基准测试 | `criterion` | 签名吞吐量、Merkle 根延迟 | `cargo bench` |

**关键必须覆盖的场景：**
- 空 Merkle 根等于已知固定值
- 签名往返（签名 → 验证，拒绝错误消息）
- 交易序列化往返

### Python 侧

| 测试类型 | 工具 | 目标 |
|----------|------|------|
| 单元测试 | `pytest` | API 边界、类型转换、异常抛出 |
| 异步测试 | `pytest-asyncio` | 异步存储读写 |
| 属性测试 | `hypothesis` | 随机字节序列编解码不变性 |
| 内存安全 | `memray` + `pytest-memray` | FFI 无泄漏、大对象正确释放 |
| 类型检查 | `mypy` | Stub 文件与实际实现一致 |

### 跨语言集成

- 用已知参考向量（如 Ethereum 测试向量）验证 Rust 结果
- 确保 Python 对 Rust 内部状态的视图保持同步

---

## CI / CD（计划中）

设计文档提议的 GitHub Actions 工作流（`.github/workflows/ci.yml`）包含三个 job：

1. **`rust-checks`** (ubuntu-latest)
   - `cargo fmt -- --check`
   - `cargo clippy --workspace -- -D warnings`
   - `cargo test --workspace --all-features`
   - `cargo bench -- --no-run`

2. **`python-checks`** (矩阵：Ubuntu / macOS / Windows × Python 3.10 / 3.11 / 3.12)
   - 安装 Pixi (`prefix-dev/setup-pixi`)
   - `pixi run dev-build`
   - `pixi run test-py`
   - `pixi run typecheck`

3. **`build-wheels`** (依赖上述 job)
   - 运行于 Ubuntu, macOS-13, macOS-14, Windows
   - 使用 `PyO3/maturin-action` 并启用 `sccache`
   - 上传 wheel 为 artifact

**发布流程（计划）：**
1. 在 `Cargo.toml` workspace、`pyproject.toml`、`pixi.toml` 中同步 bump 版本号
2. 生成 changelog（`git-cliff` 或 conventional commits）
3. 创建 GitHub Release；CI 自动构建多平台 wheel 并上传至 PyPI

---

## 代码风格与安全规范（计划中）

### Rust
- 库边界使用 `thiserror`，内部/应用逻辑可使用 `anyhow`
- 实现 `From<ChainforgeError> for pyo3::PyErr`，确保错误干净地跨越语言边界
- Panic 隔离：在 FFI 入口点捕获 unwinding，转换为 `PyRuntimeError`
- **GIL 策略：** 任何预期耗时超过 1 ms 的操作必须使用 `py.allow_threads(...)`
- 返回 `PyBytes` 而非 `PyList[int]`，减少分配
- 只读大数据缓冲区应使用 `PyBuffer` / memory view，避免拷贝 `Vec<u8>`
- 暴露给 Python 的共享可变状态必须使用 `Arc<tokio::sync::RwLock<T>>`；禁止直接暴露 `&mut`

### Python
- 所有公共 API 必须带类型注解
- 面向用户的输入提供 Pydantic 模型
- 需要 teardown 的资源使用上下文管理器（`@contextmanager`）
- 公共类与函数遵循 NumPy 风格 docstring

---

## 安全考量

- **密码学实现** 依赖经过审计的 crate（`ring`、`secp256k1`）。禁止自行实现加密算法。
- **Panic 安全：** 跨越 FFI 边界的 unwinding 是未定义行为。每个 `#[pyfunction]` 和 `#[pymethods]` 入口都必须加防护。
- **内存安全：** 避免在 `allow_threads` 闭包中捕获 `Bound<'_, PyAny>`；使用 `Py<T>` 或原始数据。
- **循环引用：** 除非必要，不要在 Rust `Arc` 结构中存储 `PyObject`；优先使用弱引用或显式清理回调。
- **CI：** 计划在 Linux 上用 `valgrind --tool=memcheck` 检测未初始化内存访问。

---

## 分阶段实施路线

项目按 11 个阶段递进实施，各阶段有明确交付物、验收标准与前置依赖：

| 阶段 | 目标 | 前置依赖 |
|------|------|----------|
| Phase 01 | 最小可编译工程骨架（目录 + 配置 + 空 crate） | 无 |
| Phase 02 | 跨语言错误体系（`ChainforgeError` + PyO3 映射） | Phase 01 |
| Phase 03 | 密码学哈希原语（SHA-256 / Keccak-256 / RIPEMD-160） | Phase 01 |
| Phase 04 | 数字签名原语（Secp256k1 ECDSA） | Phase 02, 03 |
| Phase 05 | Merkle 树与属性测试 | Phase 01, 03 |
| Phase 06 | 交易与区块核心结构 + RLP 编解码 | Phase 02, 03, 04, 05 |
| Phase 07 | 存储层 Trait + 内存后端 | Phase 02 |
| Phase 08 | RocksDB 集成与缓存 | Phase 07 |
| Phase 09 | PyO3 完整绑定层 | Phase 02 ~ 08 |
| Phase 10 | Python API 与类型校验层（Pydantic / mypy） | Phase 09 |
| Phase 11 | CI/CD 与基准测试 | Phase 01 ~ 10 |

---

## 给实施代理的建议

1. **严格按阶段顺序推进**：每个 phase 文档中都列出了验收标准，必须全部通过后再进入下一阶段。
2. **Phase 01 是基础**：先创建 `Cargo.toml` workspace、`pixi.toml`、`pyproject.toml` 和空 crate，确保 `cargo check --workspace` 与 `pixi run dev-build` 零错误。
3. **优先实现错误类型**：`ChainforgeError` 及其 PyO3 转换（Phase 02）是所有 FFI 调用的根基。
4. **尽早配置 CI**：Phase 11 虽然排在最后，但建议在工作区可编译后就先搭建 `.github/workflows/ci.yml` 的基础 job，使后续 PR 都能被自动验证。
5. **中文是主要文档语言**：`design.md` 与所有 phase 文件均以中文撰写。若新增注释或文档，建议保持中文一致性。
6. **每次修改后更新本文件**：`AGENTS.md` 必须反映仓库的最新现实。创建了新文件、目录或配置后，请同步更新本指南中的「当前状态」部分。

---

*本文档版本：0.1.0*  
*最后更新：2026-05-30*  
*维护者：工程团队*
