<!-- AGENTS.md —— Chainforge 项目 AI 编码代理指南 -->

## 项目概览

**Chainforge** 是一个高性能区块链核心库，使用 **Rust** 实现底层密码学、共识原语与数据结构，并通过 **PyO3** 向 Python 提供符合 Python 生态习惯的绑定层。目标用户为数据科学家、量化研究员及需要快速原型验证的区块链开发者。

**当前状态：** 全部框架期（Phase 01 ~ Phase 11）与全部迭代开发方向（P2P、Mempool、Block Producer、Consensus、EVM、RPC、Light Client）均已完成并通过测试。

- **Rust 测试：** 102 个全部通过（`cargo test --workspace`）
- **Python 测试：** 97 个全部通过（`pytest src/tests -v --tb=short`）
- **Clippy / 格式化：** 零警告
- **mypy 类型检查：** 零错误（`pixi run typecheck`）

---

## 仓库结构

```
chainforge/
├── Cargo.toml                 # Rust workspace 根配置
├── pixi.toml                  # Pixi 项目配置与 task 定义
├── pyproject.toml             # Python 包元数据 + maturin 配置
├── rust-toolchain.toml        # Rust 工具链锁定（stable + rustfmt + clippy）
├── .cargo/config.toml         # 指定 MinGW linker / ar 路径（Windows）
├── .github/workflows/ci.yml   # GitHub Actions CI（rust-checks / python-checks / build-wheels）
├── src/
│   ├── chainforge/            # Python 源码包
│   │   ├── __init__.py        # 统一导出公共 API + __all__
│   │   ├── types.py           # Pydantic v2 输入校验模型（TxInput / BlockInput）
│   │   ├── client.py          # 高层 Pythonic API（open_db 异步上下文管理器）
│   │   └── py.typed           # PEP 561 类型标记
│   └── tests/
│       ├── conftest.py        # pytest 共享配置（当前为空）
│       ├── unit/              # 单元测试（异常、密码学、Merkle、存储、类型、Pydantic、客户端）
│       └── integration/       # 集成测试（端到端链路测试）
├── crates/
│   ├── chainforge-error/      # 统一错误类型 ChainforgeError（thiserror）
│   ├── chainforge-crypto/     # 密码学原语：SHA-256、Keccak-256、RIPEMD-160、secp256k1 ECDSA
│   ├── chainforge-core/       # 核心结构：Transaction、BlockHeader、Block、MerkleTree、RLP 编解码、LightClient、MPT
│   ├── chainforge-storage/    # 存储抽象 Trait + InMemoryStorage + CachedStorage（LRU）+ RocksDBEngine（可选 feature）
│   ├── chainforge-py/         # PyO3 绑定层：暴露 Transaction/BlockHeader/Block/MerkleTree/SecretKey/PublicKey/InMemoryStorage/CachedStorage/RocksDBEngine/Mempool/BlockBuilder/EvmState/EvmExecutor/LightClient/MptProof/ConsensusEngine/Vote/QC/Phase/SafetyRules/Pacemaker/LeaderRotator/BlockTree/Node/NodeConfig/Message/PeerId/PeerInfo/RoutingTable + 哈希函数 + 异常类型
│   ├── chainforge-p2p/        # P2P 网络层：Noise 握手、Message 编解码、Kademlia 路由表、Gossip 广播、Sync 同步、Node 集成
│   ├── chainforge-mempool/    # 交易内存池：CRUD、优先级队列、nonce 验证、容量限制与驱逐
│   ├── chainforge-block-producer/  # 区块生产者：BlockBuilder、从 mempool 取交易构建区块
│   ├── chainforge-consensus/  # HotStuff BFT 共识：BlockTree、Vote、QC、SafetyRules、Pacemaker、ConsensusEngine
│   ├── chainforge-evm/        # EVM 执行层：revm 集成、转账、合约部署、合约调用、DatabaseCommit
│   └── chainforge-rpc/        # JSON-RPC 服务层：axum HTTP + WebSocket、eth_sendRawTransaction/eth_getBalance/eth_call/eth_subscribe 等
└── design/                    # 架构设计文档（中文撰写）
    ├── framework/design.md    # v0.1.0 总纲设计文档
    ├── framework/phases/      # Phase 01 ~ Phase 11 分阶段实施文档
    ├── p2p/design.md
    ├── consensus/design.md
    ├── evm/design.md
    ├── rpc/design.md
    ├── mempool/design.md
    ├── block-producer/design.md
    └── light-client/design.md
```

---

## 技术栈

| 层级 | 技术 | 用途 |
|------|------|------|
| 核心实现 | Rust (stable, edition 2021) | 密码学、Merkle 树、存储引擎、序列化、共识、P2P、EVM、RPC |
| Python 绑定 | PyO3 (v0.23+) + pyo3-async-runtimes | FFI 层，将 Rust 类型暴露给 Python |
| 异步运行时 | Tokio (v1.x) | Rust 侧异步 IO，桥接 Python `asyncio` |
| 序列化 | `serde` + `bincode` / `serde_json` | 内部存储用 bincode，RPC 用 JSON |
| 密码学 | `ring` (SHA-256/AES) + `secp256k1` (ECDSA) + `tiny-keccak` + `ripemd` | 哈希与签名 |
| EVM 执行 | `revm` (v14) | 以太坊兼容合约执行层 |
| P2P 网络 | `snow` (Noise XX 握手) + `tokio` | 加密传输与节点发现 |
| RPC 服务 | `axum` (v0.7, WebSocket) + `serde_json` + `hex` | JSON-RPC HTTP 与 WebSocket 服务 |
| 存储引擎 | `rocksdb` (可选, v0.23) + `lru` (v0.12) | LSM-Tree KV 后端 + LRU 缓存 |
| 错误处理 | `thiserror` + `anyhow` | Rust 侧结构化错误 |
| 属性测试 | `proptest` (v1.5) | 随机化正确性测试 |
| 基准测试 | `criterion` (v0.5) | 性能回归测试 |
| Python 打包 | `maturin` (v1.7+) | PEP 517 / PyO3 wheel 构建后端 |
| Python 环境 | `pixi` | 跨平台可复现的 Python + Rust 开发环境 |
| Python 静态检查 | `mypy` | Type stub 校验 |
| Python 运行时检查 | `pydantic` (v2) | 输入校验与 JSON Schema 生成 |
| Python 测试 | `pytest` + `pytest-asyncio` | 单元、异步测试 |
| Python 格式化 | `ruff` | Python 代码格式化 |

---

## Crate 说明与依赖关系

### 1. chainforge-error
- **职责：** 整个 workspace 的统一错误枚举 `ChainforgeError`
- **变体：** `Crypto`、`Serialization`、`Storage`、`InvalidParameter`、`StateRootMismatch`
- **依赖：** 仅 `thiserror`

### 2. chainforge-crypto
- **职责：** 密码学哈希与数字签名
- **模块：** `hash`（sha256、keccak256、ripemd160）、`ecdsa`（SecretKey、PublicKey、Signature）
- **依赖：** `ring`、`tiny-keccak`、`ripemd`、`secp256k1`、`rand`
- **测试：** 16 个（含 6 个 proptest 属性测试）
- **基准：** `sign_bench`（签名吞吐量）

### 3. chainforge-core
- **职责：** 区块链核心数据结构、编解码、轻客户端、MPT
- **模块：**
  - `tx` —— Transaction（RLP 编解码、hash、recover_sender、sign）
  - `block` —— BlockHeader、Block（RLP 编解码、compute_txs_root）
  - `merkle` —— MerkleTree（二叉 SHA-256）、MerkleProof
  - `rlp` —— RlpEncoder / RlpDecoder
  - `light_client` —— LightClient（区块头同步验证、交易包含验证）
  - `mpt` —— MPT（Merkle Patricia Trie）证明验证
- **测试：** 32 个
- **基准：** `merkle_bench`

### 4. chainforge-storage
- **职责：** 异步 KV 存储抽象与实现
- **Trait：** `StorageEngine`、`BatchWrite`、`Snapshot`、`Snapshotable`
- **实现：**
  - `InMemoryStorage` —— 纯内存 HashMap 后端（`Arc<RwLock<HashMap>>`）
  - `CachedStorage` —— LRU 缓存包装层
  - `RocksDBEngine` —— RocksDB 后端（可选 feature `rocksdb-backend`）
- **测试：** 含异步测试（put/get/delete/batch/snapshot/isolation）
- **基准：** `rocksdb_bench`

### 5. chainforge-mempool
- **职责：** 交易内存池
- **功能：** 插入/查询/删除、按 gas_price 优先级排序、按账户 nonce 跟踪、容量上限与最低优先级驱逐
- **测试：** 7 个

### 6. chainforge-p2p
- **职责：** P2P 网络层
- **模块：** `transport`（Noise XX 握手）、`message`（Message 枚举）、`discovery`（Kademlia 路由表）、`gossip`（Gossip 广播去重）、`sync`（区块同步）、`node`（Node 主结构整合所有子系统）、`peer`（PeerId / PeerInfo）
- **测试：** 含异步测试

### 7. chainforge-consensus
- **职责：** Chained HotStuff BFT 共识引擎
- **模块：** `block_tree`（BlockTree）、`vote`（Vote / QC / Phase）、`safety`（SafetyRules）、`pacemaker`（Pacemaker / LeaderRotator）、`hotstuff`（ConsensusEngine）
- **测试：** 5 个（propose、full pipeline commit、safety reject、leader rotation、fork choice）

### 8. chainforge-block-producer
- **职责：** 区块构建与出块
- **功能：** `BlockBuilder`（builder 模式）、`produce_block`（从 mempool 取交易构建区块）
- **测试：** 2 个

### 9. chainforge-evm
- **职责：** EVM 兼容执行层
- **模块：** `executor`（EvmExecutor — transfer / deploy / call / balance / nonce）、`state`（InMemoryEvmState — 实现 revm 的 Database / DatabaseRef / DatabaseCommit）
- **依赖：** `revm`
- **测试：** 3 个（转账、余额不足、部署并调用 counter 合约）

### 10. chainforge-rpc
- **职责：** 以太坊兼容 JSON-RPC 服务
- **模块：** `server`（axum HTTP 路由与处理器）、`types`（RpcRequest / RpcResponse）、`ws`（WebSocket 事件订阅）
- **方法：** eth_chainId、eth_sendRawTransaction、eth_getBalance、eth_getTransactionCount、eth_getBlockByNumber、eth_getBlockByHash、eth_call、eth_getCode、eth_blockNumber、net_version
- **测试：** 含异步 HTTP 测试（使用 axum + tower::ServiceExt）

### 11. chainforge-py
- **职责：** 唯一依赖 PyO3 的 crate，将 Rust 核心类型暴露为 Python 扩展模块 `chainforge._internal`
- **暴露类型：**
  - `PyMerkleTree`（`MerkleTree`）
  - `PyTransaction`（`Transaction`）
  - `PyBlockHeader`（`BlockHeader`）
  - `PyBlock`（`Block`）
  - `PyLightClient`（`LightClient`）
  - `PyMptProof`（`MptProof`）
  - `PySecretKey` / `PyPublicKey`（secp256k1）
  - `PyInMemoryStorage`（`InMemoryStorage`，异步方法通过 `future_into_py` 桥接）
  - `PyCachedStorage`（`CachedStorage`）
  - `PyRocksDBEngine`（`RocksDBEngine`，可选 feature `rocksdb-backend`）
  - `PyMempool`（`Mempool`）
  - `PyBlockBuilder`（`BlockBuilder`）
  - `PyEvmState`（`InMemoryEvmState`）
  - `PyEvmExecutor`（`EvmExecutor<InMemoryEvmState>`）
  - `PyExecutionResult`（`ExecutionResult`）
- **错误映射：** `ChainforgeError` → 细化的自定义异常：`ChainforgeValueError`（InvalidParameter / Serialization）、`ChainforgeCryptoError`（Crypto）、`ChainforgeStorageError`（Storage）、`ChainforgeStateError`（StateRootMismatch）、`ChainforgeRuntimeError`（其他）
- **辅助函数：** `keccak256`、`sha256`、`ripemd160`、`raise_invalid_parameter`、`raise_crypto`、`raise_storage`、`raise_serialization`、`raise_state_root_mismatch`

---

## Python 包结构

```python
# src/chainforge/__init__.py
from chainforge._internal import (
    Block, BlockBuilder, BlockHeader, CachedStorage, ChainforgeCryptoError,
    ChainforgeError, ChainforgeRuntimeError, ChainforgeStateError,
    ChainforgeStorageError, ChainforgeValueError, LightClient, MerkleTree,
    Mempool, MptProof, PublicKey, RocksDBEngine, SecretKey, Transaction,
    keccak256, ripemd160, sha256,
)
from chainforge.client import open_db
from chainforge.types import BlockInput, TxInput

__all__ = [
    "Block", "BlockBuilder", "BlockHeader", "BlockInput", "CachedStorage",
    "ChainforgeCryptoError", "ChainforgeError", "ChainforgeRuntimeError",
    "ChainforgeStateError", "ChainforgeStorageError", "ChainforgeValueError",
    "LightClient", "Mempool", "MerkleTree", "MptProof", "PublicKey",
    "RocksDBEngine", "SecretKey", "Transaction", "TxInput",
    "keccak256", "open_db", "ripemd160", "sha256",
]
```

- `types.py` —— `TxInput`（nonce、gas_price、gas_limit、to、value、data）与 `BlockInput`（parent_hash、number、timestamp、extra_data）的 Pydantic v2 校验模型
- `client.py` —— `open_db()` 异步上下文管理器，包装 `PyInMemoryStorage`

---

## 构建与开发命令

所有命令均通过 `pixi run <task>` 执行（定义于 `pixi.toml`）：

| 任务 | 命令 | 说明 |
|------|------|------|
| 验证 Rust 工具链 | `pixi run install-rust` | `rustup show` |
| 可编辑安装 | `pixi run dev-build` | `maturin develop --release` |
| Rust 测试 | `pixi run test-rust` | `cargo test --workspace` |
| Python 测试 | `pixi run test-py` | `pytest src/tests -v --tb=short` |
| 全量测试 | `pixi run test` | 同时运行 Rust 与 Python 测试 |
| 类型检查 | `pixi run typecheck` | `mypy src/chainforge` |
| 格式化 | `pixi run fmt` | `cargo fmt && ruff format src/` |
| 基准测试 | `pixi run bench` | `cargo bench --workspace` |

**快速启动：**
```bash
pixi install
pixi run dev-build
pixi run test
```

---

## CI / CD

`.github/workflows/ci.yml` 包含三个 job：

1. **rust-checks**（ubuntu-latest）
   - `cargo fmt -- --check`
   - `cargo clippy --workspace -- -D warnings`
   - `cargo test --workspace --all-features`
   - `cargo bench --workspace -- --no-run`

2. **python-checks**（矩阵：Ubuntu / macOS / Windows × Python 3.10 / 3.11 / 3.12）
   - 安装 Pixi (`prefix-dev/setup-pixi@v0.8.0`)
   - `pixi run dev-build`
   - `pixi run test-py`
   - `pixi run typecheck`

3. **build-wheels**（依赖上述两个 job）
   - 运行于 Ubuntu、macOS-13、macOS-14、Windows
   - 使用 `PyO3/maturin-action@v1` 并启用 `sccache`
   - 上传 wheel 为 artifact

---

## 测试策略

### Rust 侧

| 测试类型 | 工具 | 目标 | 命令 |
|----------|------|------|------|
| 单元测试 | `cargo test` | 公共 API、边界条件 | `cargo test --workspace` |
| 文档测试 | 内置 | 编译并运行文档示例 | 内置于 `cargo test` |
| 属性测试 | `proptest` | 随机交易序列、状态一致性、哈希输出长度 | `cargo test`（部分 crate） |
| 基准测试 | `criterion` | 签名吞吐量、Merkle 根延迟、RocksDB 性能 | `cargo bench --workspace` |

**关键已覆盖场景：**
- 空 Merkle 根等于已知固定值
- 签名往返（签名 → 验证，拒绝错误消息）
- 交易 RLP 序列化往返
- 交易发送方恢复（sign → recover_sender）
- 存储快照隔离
- Mempool 容量驱逐与优先级队列
- HotStuff 完整三阶段提交流水线
- EVM 转账、余额不足、合约部署
- RPC HTTP 端点（chainId、sendRawTransaction、getBalance、call 等）
- P2P Node 消息去重与 peer discovery
- LightClient 区块头连续性与交易包含验证
- MPT 证明验证（branch、extension + leaf）

### Python 侧

| 测试类型 | 工具 | 目标 |
|----------|------|------|
| 单元测试 | `pytest` | API 边界、类型转换、异常抛出 |
| 异步测试 | `pytest-asyncio` | 异步存储读写 |
| 类型检查 | `mypy` | Stub 文件与实际实现一致 |

**测试文件清单：**
- `test_exceptions.py` —— ChainforgeError 异常映射测试
- `test_py_crypto.py` —— SecretKey 生成、签名验证
- `test_py_merkle.py` —— MerkleTree root、proof、verify
- `test_py_storage.py` —— InMemoryStorage put/get/delete
- `test_py_types.py` —— Transaction / BlockHeader 构造、hash、RLP 往返
- `test_pydantic_types.py` —— TxInput / BlockInput 校验（有效值、默认值、负值、长度超限）
- `test_py_block.py` —— Block / BlockBuilder / produce_block 构造、hash、RLP 往返
- `test_py_mempool.py` —— Mempool 插入/查询/删除/优先级/驱逐/nonce 跟踪
- `test_py_light_client.py` —— LightClient 区块头同步、MptProof 构造
- `test_py_evm.py` —— EvmState / EvmExecutor 转账、部署、状态查询
- `test_py_storage.py` —— InMemoryStorage / CachedStorage put/get/delete
- `test_consensus.py` —— HotStuff 共识：Phase / Vote / QC / BlockTree / SafetyRules / Pacemaker / LeaderRotator / ConsensusEngine
- `test_p2p.py` —— P2P 网络：PeerId / PeerInfo / Message / NodeConfig / RoutingTable / Node（消息处理、收件箱拉取）
- `test_client.py` —— `open_db` 上下文管理器
- `test_e2e.py` —— 端到端签名验证、Merkle 树、BlockHeader hash

---

## 代码风格与安全规范

### Rust

- 库边界使用 `thiserror` 定义结构化错误；内部逻辑可使用 `anyhow`
- `ChainforgeError` 通过 `into_py_err` 映射到细化的自定义 Python 异常：`InvalidParameter` / `Serialization` → `ChainforgeValueError`（`ValueError` 子类），`Crypto` → `ChainforgeCryptoError`（`RuntimeError` 子类），`Storage` → `ChainforgeStorageError`（`RuntimeError` 子类），`StateRootMismatch` → `ChainforgeStateError`（`RuntimeError` 子类）
- **GIL 策略：** 异步存储方法通过 `future_into_py` 桥接并释放 GIL；计算密集型操作应在 `py.allow_threads(...)` 中执行
- 返回 `PyBytes` 而非 `PyList[int]`，减少 Python 对象分配
- 暴露给 Python 的共享可变状态使用 `Arc<tokio::sync::RwLock<T>>` 或 `Arc<std::sync::RwLock<T>>`；禁止直接暴露 `&mut`
- 所有 Rust crate 均通过 `cargo clippy --workspace -- -D warnings`
- 使用 `cargo fmt` 统一格式化

### Python

- 所有公共 API 必须带类型注解
- 面向用户的输入提供 Pydantic 模型（`TxInput`、`BlockInput`）
- 需要 teardown 的资源使用上下文管理器（`@asynccontextmanager`）
- 公共类与函数遵循 NumPy 风格 docstring
- 使用 `ruff format src/` 格式化 Python 代码

### 安全考量

- **密码学实现** 依赖经过审计的 crate（`ring`、`secp256k1`）。禁止自行实现加密算法。
- **Panic 安全：** 跨越 FFI 边界的 unwinding 是未定义行为。PyO3 入口需避免 panic；若可能 panic，应使用 `catch_unwind` 捕获并转换为 `PyRuntimeError`。
- **内存安全：** 避免在 `allow_threads` 闭包中捕获 `Bound<'_, PyAny>`；使用 `Py<T>` 或原始数据。
- **循环引用：** 除非必要，不要在 Rust `Arc` 结构中存储 `PyObject`；优先使用弱引用或显式清理回调。
- RocksDB 后端为可选 feature（`rocksdb-backend`），在无法编译 RocksDB 的平台可回退到纯内存存储。

---

## 版本管理与发布

- 版本号同步维护于三处：`Cargo.toml`（workspace.package.version）、`pyproject.toml`（project.version）、`pixi.toml`（workspace.version）
- 当前版本：**0.1.0**
- 许可证：**MIT OR Apache-2.0**

---

## 设计文档

所有设计文档位于 `design/` 目录，以 **中文** 撰写：

| 文档 | 内容 |
|------|------|
| `design/framework/design.md` | v0.1.0 总纲：技术选型、目录结构、PyO3 绑定设计、Python API 层、测试策略、CI/CD、性能优化 |
| `design/framework/phases/phase-01.md` ~ `phase-11.md` | 分阶段实施文档，每阶段含交付物与验收标准 |
| `design/p2p/design.md` | P2P 网络层设计（Noise、Kademlia、Gossip、Sync） |
| `design/consensus/design.md` | HotStuff BFT 共识设计 |
| `design/evm/design.md` | EVM 兼容执行层设计 |
| `design/rpc/design.md` | JSON-RPC API 层设计 |
| `design/mempool/design.md` | 交易池设计 |
| `design/block-producer/design.md` | 区块生产者设计 |
| `design/light-client/design.md` | 轻客户端设计 |

---

## 给编码代理的建议

1. **修改后必须运行测试：** 任何 Rust 代码变更后执行 `cargo test --workspace`；任何 Python 代码变更后执行 `pixi run test-py` 与 `pixi run typecheck`。
2. **新增 crate：** 需在 `Cargo.toml` workspace members 中注册，并遵循现有 crate 的 `version.workspace = true`、`edition.workspace = true` 约定。
3. **新增 PyO3 暴露类型：** 仅在 `chainforge-py` crate 中进行，并在 `src/chainforge/__init__.py` 中重新导出。
4. **错误处理：** 新增 Rust 错误变体时，同步更新 `chainforge-error/src/lib.rs` 与 `chainforge-py/src/error.rs` 的映射逻辑。
5. **文档语言：** 代码注释与新增文档建议保持中文一致性，以匹配现有设计文档风格。
6. **更新本文件：** 若新增/删除 crate、修改 CI、变更构建命令或调整测试策略，请同步更新本 `AGENTS.md`。

---

*本文档版本：0.2.0*  
*最后更新：2026-05-30*  
*维护者：工程团队*
