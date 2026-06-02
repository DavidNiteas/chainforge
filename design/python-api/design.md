# Kilnchain Python API 扩展设计文档

> **版本**：v0.1.0-draft  
> **日期**：2026-05-30  
> **目标**：将 Kilnchain 全部 Rust 子系统通过 PyO3 暴露为 Python API，使 Python 成为 Rust 核心库的表达层（Skin），而非逻辑层。

---

## 一、设计理念

### 1.1 核心原则：Python 是 Rust 的皮

| 原则 | 含义 | 后果 |
|------|------|------|
| **单向调用** | Python 调用 Rust；Rust 不调用 Python。 | 所有业务逻辑、状态机、事件循环均驻留在 Rust 侧。 |
| **零 Python 回调** | 不允许将 Python 函数（`Callable`）传入 Rust 作为钩子或事件处理器。 | 事件通知必须通过**拉取模式**（轮询、队列、`async for`）实现。 |
| **Rust 自闭环** | Rust 模块之间自由调用；Python 只负责配置、编排与观测。 | P2P 消息处理、共识状态机推进、EVM 执行等均在 Rust 内部完成。 |
| **配置即代码** | Python API 的首要职责是让开发者用 Pythonic 语法描述"要做什么"，而非"怎么做"。 | 类似 Polars 的惰性求值 / 链式表达式风格。 |

### 1.2 与 Polars 的对照

| Polars | Kilnchain (目标) |
|--------|-------------------|
| `pl.DataFrame(...).filter(...).group_by(...).agg(...)` | `cf.mempool().insert(tx).pop_highest(100).build_block(...)` |
| 计算图在 Rust 中构建与执行 | 区块链操作在 Rust 中构建与执行 |
| Python 侧只有表达式树，无实际计算 | Python 侧只有配置与触发，无实际共识 / EVM / P2P 逻辑 |
| `.collect()` 触发执行 | `.execute()` / `await .run()` 触发执行 |

---

## 二、架构总览

```
┌──────────────────────────────────────────────────────────────────────┐
│ Layer 3: Application DSL (src/kilnchain/)                           │
│                                                                      │
│   node    = cf.NodeBuilder().with_mempool(...).with_evm(...).build() │
│   result  = await node.run()                                         │
│   block   = node.chain.latest_block()                                │
│                                                                      │
│   # 合约交互（链式表达式）                                              │
│   result = cf.vm(state).deploy(bytecode).call(calldata).collect()    │
├──────────────────────────────────────────────────────────────────────┤
│ Layer 2: Pythonic Wrappers (src/kilnchain/wrappers/)                │
│                                                                      │
│   对 Layer 1 的薄包装：添加类型注解、文档字符串、                         │
│   上下文管理器、异常细化、异步生成器适配。                                │
├──────────────────────────────────────────────────────────────────────┤
│ Layer 1: PyO3 Raw Bindings (crates/kilnchain-py/src/)               │
│                                                                      │
│   #[pyclass] / #[pymethods] / #[pyfunction]                          │
│   直接映射 Rust 公共 API。所有 PyO3 代码集中在此 crate。                 │
│   禁止在此层引入 Python 逻辑。                                          │
└──────────────────────────────────────────────────────────────────────┘
```

---

## 三、模块映射方案

### 3.1 暴露优先级矩阵

| Rust Crate | 当前暴露 | Phase A | Phase B | Phase C | Phase D |
|-----------|---------|---------|---------|---------|---------|
| `kilnchain-crypto` | SecretKey, PublicKey | +Hash 函数 | — | — | — |
| `kilnchain-core` | Tx, Header, MerkleTree | +Block, +LightClient | +MPT | — | — |
| `kilnchain-storage` | InMemoryStorage | +CachedStorage | +RocksDBEngine | — | — |
| `kilnchain-mempool` | ❌ | Mempool | — | — | — |
| `kilnchain-block-producer` | ❌ | BlockBuilder, produce_block | — | — | — |
| `kilnchain-evm` | ❌ | EvmExecutor, EvmState | — | — | +Contract DSL |
| `kilnchain-consensus` | ❌ | — | ConsensusEngine | +Vote/QC/Phase | — |
| `kilnchain-p2p` | ❌ | — | — | Node, Message, Transport | +Sync/Gossip |
| `kilnchain-rpc` | ❌ | — | — | RpcState, RpcServer | +EventBus |

### 3.2 各模块详细映射

#### 3.2.1 kilnchain-crypto（Phase PA-01）

新增 PyFunction：

```rust
#[pyfunction] fn sha256(data: &[u8]) -> [u8; 32]
#[pyfunction] fn keccak256(data: &[u8]) -> [u8; 32]  // 已存在，保持不变
#[pyfunction] fn ripemd160(data: &[u8]) -> [u8; 20]
```

#### 3.2.2 kilnchain-core（Phase PA-01）

| Rust 类型 | Python 类名 | 暴露方法 |
|-----------|------------|---------|
| `Block` | `Block` | `hash()`, `to_rlp()`, `header` (getter), `transactions` (getter) |
| `LightClient` | `LightClient` | `add_header()`, `verify_tx_inclusion()` |
| `MptProof` | `MptProof` | `verify()` |

#### 3.2.3 kilnchain-storage（Phase PA-01 + PA-02）

| Rust 类型 | Python 类名 | 暴露方法 |
|-----------|------------|---------|
| `CachedStorage` | `CachedStorage` | `get()`, `put()`, `delete()` (async) |
| `RocksDBEngine` | `RocksDBEngine` | `get()`, `put()`, `delete()` (async) |

> RocksDBEngine 为可选 feature `rocksdb-backend`，PyO3 侧需做 feature-gate 或运行时检测。

#### 3.2.4 kilnchain-mempool（Phase PA-01）

```python
from kilnchain import Mempool, Transaction

pool = Mempool.with_capacity(10000)
pool.insert(tx)
pool.get(tx_hash)
pool.remove(tx_hash)
pool.contains(tx_hash)
pool.len()
pool.is_empty()
pool.txs()               # -> dict[bytes, Transaction]
pool.pop_highest_priority(limit=100)  # -> list[Transaction]
pool.next_nonce(sender)  # -> int
pool.is_nonce_valid(tx)  # -> bool
```

#### 3.2.5 kilnchain-block-producer（Phase PA-01）

```python
from kilnchain import BlockBuilder, Block, produce_block

# Builder 模式
block = (
    BlockBuilder(parent_hash=prev_hash, number=1)
    .timestamp(1234567890)
    .extra_data(b"py")
    .state_root(root)
    .transactions(txs)
    .gas_limit(30_000_000)
    .build()
)

# 便捷函数
block = produce_block(
    parent_hash=prev_hash,
    number=1,
    timestamp=1234567890,
    mempool=pool,
    max_txs=100,
)
```

#### 3.2.6 kilnchain-evm（Phase PA-01 + PA-06）

**Phase PA-01 —— 底层暴露：**

```python
from kilnchain import EvmState, EvmExecutor, Address

state = EvmState()
state.set_balance(address, 10**18)
state.set_nonce(address, 0)
state.set_code(address, bytecode)
state.set_storage(address, slot, value)

executor = EvmExecutor(state)
result = executor.transfer(from_addr, to_addr, value)
result = executor.deploy(from_addr, bytecode, value)
result = executor.call(from_addr, to_addr, data, value)
balance = executor.balance(address)
nonce = executor.nonce(address)
```

**Phase PA-06 —— Contract DSL（链式表达式）：**

```python
from kilnchain import vm

# 方式一：链式配置 + 批量执行
counter = (
    vm.state()
    .with_balance(deployer, 10**18)
    .executor()
    .deploy(deployer, bytecode)
)

result = counter.call(deployer, method="increment", args=[])
result = counter.call(deployer, method="get_count", args=[])
count = result.decode_output("uint256")

# 方式二：预绑定 ABI（推荐）
contract = vm.contract(abi_json, address=contract_addr)
receipt = await contract.increment(from_addr)
count = await contract.get_count()
```

> **关键约束**：`Contract` 对象的所有方法调用最终都转换为 Rust 侧的 `EvmExecutor.call()`，Python 不介入 ABI 编解码逻辑（除非 ABI 解析本身也在 Rust 中完成）。

#### 3.2.7 kilnchain-consensus（Phase PA-02 + PA-03）

```python
from kilnchain import (
    ConsensusEngine, BlockTree, Vote, QuorumCertificate,
    Phase, SafetyRules, Pacemaker, LeaderRotator,
)

engine = ConsensusEngine(node_id=0, node_count=4)

# 领导者
block = engine.propose_block(parent_hash, number, txs, high_qc)

# 副本
vote = engine.vote_prepare(block, high_qc)

# 收集投票
qc = engine.form_qc(votes, phase=Phase.PREPARE, quorum=3)

# QC 处理
engine.on_prepare_qc(block, qc)
engine.on_precommit_qc(block_hash, qc)
engine.on_commit_qc(block_hash, qc)

# 视图推进
engine.advance_view(view_number)

# 查询
committed = engine.block_tree.committed_blocks()
node = engine.block_tree.get(block_hash)
height = engine.block_tree.committed_height()
leader = engine.pacemaker.current_leader()
is_leader = engine.pacemaker.is_leader()
```

#### 3.2.8 kilnchain-p2p（Phase PA-03 + PA-04）

**核心约束**：P2P Node 的消息处理逻辑完全在 Rust 内部。Python 不参与任何消息处理回调。

```python
from kilnchain import Node, NodeConfig, Message, PeerId, PeerInfo

config = NodeConfig(static_key=my_key, gossip_fanout=5, gossip_ttl_secs=60)
node = Node(config)

# 启动监听（Rust 内部启动 tokio 任务）
await node.start_listen("0.0.0.0:30303")

# 主动拨号
stream = await node.dial("192.168.1.10:30303")

# 发送消息
await node.broadcast(Message.transaction(tx.to_rlp()))
await node.broadcast(Message.block(block.to_rlp()))

# 状态查询（同步/只读）
peers: list[PeerInfo] = node.routing_table.find_closest(target, k=8)
count = node.routing_table.len()

# 接收消息 —— 拉取模式（非回调）
msgs = node.drain_inbox()  # -> list[Message]
for msg in msgs:
    if msg.is_transaction():
        pool.insert(msg.decode_transaction())
```

> **设计说明**：`Node` 内部维护一个 `VecDeque<Message>` 作为收件箱。Rust 的 `handle_message()` 将符合条件的消息推入此队列。Python 通过 `drain_inbox()` 批量拉取。这是"拉取模式"（Pull）而非"回调模式"（Push），符合"Python 是皮"的原则。

#### 3.2.9 kilnchain-rpc（Phase PA-03 + PA-04）

```python
from kilnchain import RpcServer, RpcState

state = RpcState(
    chain_id=1337,
    mempool=pool,
    evm_state=evm_state,
    storage=storage,
)

server = RpcServer(state)

# 启动（Rust 内部启动 axum + tokio）
await server.start(host="127.0.0.1", port=8545)

# 停止
await server.stop()

# 查询订阅者数量
subs = server.ws_subscriber_count()

# 手动推送事件（从 Python 侧触发 Rust EventBus）
server.publish_new_head(header.to_dict())
server.publish_pending_tx(tx_hash.hex())
```

---

## 四、事件与通知模型（拉取模式）

由于禁止 Rust 调用 Python，所有"事件"必须通过以下三种方式传递：

### 4.1 轮询（Polling）

```python
# 适用于低频状态查询
while True:
    msgs = node.drain_inbox()
    for msg in msgs:
        handle(msg)
    await asyncio.sleep(0.1)
```

### 4.2 异步生成器（Async Generator）

```python
# 适用于流式数据
async for msg in node.inbox_stream():
    handle(msg)
```

> 实现方式：Rust 侧维护一个 `tokio::sync::mpsc` 或 `async_channel`，PyO3 侧通过 `future_into_py` 包装为 Python 异步生成器。

### 4.3 Future 等待（Await）

```python
# 适用于一次性等待
msg = await node.wait_for_message(timeout=5.0)
```

---

## 五、错误映射规范

细化 Python 异常类型，便于调用方精确捕获：

| Rust 错误变体 | Python 异常类 | 父类 |
|--------------|--------------|------|
| `InvalidParameter` | `KilnchainValueError` | `ValueError` |
| `Serialization` | `KilnchainValueError` | `ValueError` |
| `Crypto` | `KilnchainCryptoError` | `RuntimeError` |
| `Storage` | `KilnchainStorageError` | `RuntimeError` |
| `StateRootMismatch` | `KilnchainStateError` | `RuntimeError` |
| 其他 / 新增 | `KilnchainRuntimeError` | `RuntimeError` |

```python
from kilnchain import (
    KilnchainValueError,
    KilnchainCryptoError,
    KilnchainStorageError,
    KilnchainStateError,
    KilnchainRuntimeError,
)
```

---

## 六、GIL 与性能规范

1. **计算密集型**：签名、哈希、Merkle 根计算、EVM 执行、RLP 编解码 → 使用 `py.allow_threads(|| ...)` 释放 GIL。
2. **异步 I/O**：Storage、P2P、RPC → 使用 `future_into_py` 桥接到 Python `asyncio`。
3. **只读大数据**：返回 `PyBytes` 或 memory view，避免 `list[int]`。
4. **禁止捕获 `Bound<'_, PyAny>` 到异步闭包中**。

---

## 七、阶段实施计划

### Phase PA-01：基础补齐（密码学 + 核心 + 存储 + Mempool + 区块生产）

**目标**：暴露链的"数据层"与"构建层"。

**任务清单**：
1. `kilnchain-py/src/crypto.rs`：新增 `sha256`, `ripemd160` PyFunction。
2. `kilnchain-py/src/types.rs`：新增 `PyBlock`, `PyLightClient`, `PyMptProof`。
3. `kilnchain-py/src/storage.rs`：新增 `PyCachedStorage`, `PyRocksDBEngine`（feature-gate）。
4. `kilnchain-py/src/mempool.rs`（新建）：暴露 `PyMempool`。
5. `kilnchain-py/src/block_producer.rs`（新建）：暴露 `PyBlockBuilder`, `produce_block`。
6. `kilnchain-py/src/lib.rs`：注册新增类与函数；细化异常映射。
7. `src/kilnchain/__init__.py`：重新导出新增公共 API。
8. `src/kilnchain/wrappers/`：添加 Pythonic 包装（类型注解、文档）。
9. 补充 Python 单元测试（`test_py_mempool.py`, `test_py_block.py`, `test_py_evm.py` 等）。

**验收标准**：
- `cargo test --workspace` 通过。
- `pixi run test-py` 通过。
- `pixi run typecheck` 零错误。
- 能从 Python 完成：`Mempool 插入/查询/驱逐 → BlockBuilder 构建 → Block RLP 编解码` 完整链路。

---

### Phase PA-02：EVM 执行层

**目标**：暴露 EVM 执行能力。

**任务清单**：
1. `kilnchain-py/src/evm.rs`（新建）：暴露 `PyEvmState`, `PyEvmExecutor`, `PyExecutionResult`。
2. 映射 `revm::primitives::Address` 为 Python `bytes`（20 字节）。
3. 映射 `revm::primitives::U256` 为 Python `int`。
4. `src/kilnchain/vm.py`（新建）：Layer 2/3 包装，提供 `vm.state()`, `vm.executor()` 入口。

**验收标准**：
- Python 侧可完成：创建 EvmState → 设置余额/Nonce → 转账 → 部署合约 → 调用合约 → 查询状态。
- 所有 EVM 测试向量与 Rust 侧一致。

---

### Phase PA-03：共识引擎 ✅ 已完成

**目标**：暴露 HotStuff 共识全部类型。

**任务清单**：
1. `kilnchain-py/src/consensus.rs`（新建）：暴露 `PyConsensusEngine`, `PyBlockTree`, `PyVote`, `PyQuorumCertificate`, `PyPhase`, `PySafetyRules`, `PyPacemaker`, `PyLeaderRotator`, `PyBlockNode`。
2. 映射 `kilnchain_crypto::ecdsa::PublicKey` / `Signature`（Vote 中通过 `voter` / `signature` / `recovery_id` getter 暴露为 bytes）。
3. `BlockTree.committed_blocks()` 在绑定层通过 `clone()` 解决引用生命周期问题。
4. `types.rs`：为 `PyBlock` 补充 `compute_txs_root()` 方法，供 Python 侧在构造测试区块时使用。
5. `kilnchain-consensus`：为 `BlockTree`, `SafetyRules`, `Pacemaker`, `LeaderRotator` 添加 `#[derive(Clone)]`，支持 Python getter 返回独立副本。

**验收标准**：
- Python 侧可复现 Rust 侧 `test_full_pipeline` 测试：提议 → 投票 → 形成 QC → Prepare → PreCommit → Commit。
- 新增 14 个 Python 测试，全部通过。
- `cargo test --workspace` 102 个通过，`pytest` 77 个通过，`clippy` / `mypy` 零错误。

---

### Phase PA-04：P2P 网络层 ✅ 已完成

**目标**：暴露 P2P Node，支持消息处理与拉取。

**任务清单**：
1. `kilnchain-py/src/p2p.rs`（新建）：暴露 `PyNode`, `PyNodeConfig`, `PyMessage`, `PyPeerId`, `PyPeerInfo`, `PyRoutingTable`。
2. 在 Rust 侧为 `Node` 添加内部收件箱队列（`VecDeque<Message>`），支持 `drain_inbox()` 和 `push_inbox()`。
3. `handle_message()` / `gossip_targets()` / `drain_inbox()` / `routing_table()` 通过 `future_into_py` 暴露为 async。
4. `kilnchain-p2p/src/node.rs`：为 `Node` 添加 `#[derive(Clone)]`（基于 `Arc` 字段）。
5. `kilnchain-p2p/src/discovery.rs`：为 `KBucket` 和 `RoutingTable` 添加 `#[derive(Clone)]`。

**验收标准**：
- Python 侧可创建 Node，发送/接收 Message，通过 `drain_inbox()` 拉取消息。
- 新增 11 个 Python 测试，全部通过。
- `cargo test --workspace` 102 个通过，`pytest` 97 个通过，`clippy` / `mypy` 零错误。

---

### Phase PA-05：RPC 服务层

**目标**：暴露 RPC Server，支持启动/停止/事件推送。

**任务清单**：
1. `kilnchain-py/src/rpc.rs`（新建）：暴露 `PyRpcState`, `PyRpcServer`。
2. `PyRpcServer.start(host, port)` → 内部 `tokio::spawn(axum::serve(...))`。
3. `PyRpcServer.stop()` → 触发 graceful shutdown。
4. 暴露 `publish_new_head()`, `publish_pending_tx()` 供 Python 侧手动推送事件。

**验收标准**：
- Python 侧可启动 RPC Server，通过 `requests` 库调用 `eth_chainId` 等接口。
- WebSocket 订阅可通过 `websockets` 库验证。

---

### Phase PA-06：高阶 DSL 与整合（Layer 3）

**目标**：提供 Polars 风格的链式 API，将各子系统整合为开箱即用的应用框架。

**任务清单**：
1. `src/kilnchain/node.py`：提供 `NodeBuilder` / `BlockchainNode` 高阶类。
2. `src/kilnchain/vm.py`：提供 `Contract` DSL（预绑定 ABI、链式调用）。
3. `src/kilnchain/events.py`：统一事件总线（基于 `asyncio.Queue` 的 Python 侧事件分发）。
4. 编写示例：`examples/minimal_chain.py`（单节点内存链）、`examples/p2p_network.py`（多节点组网）。
5. 编写 `examples/contract_counter.py`（部署并调用 Counter 合约）。

**验收标准**：
- `examples/minimal_chain.py` 可运行并输出区块。
- `examples/contract_counter.py` 可部署并调用合约。
- `mypy` 对所有示例通过类型检查。

---

## 八、目录结构变更

```
crates/kilnchain-py/src/
├── lib.rs              # 模块注册、异常映射
├── error.rs            # KilnchainError -> PyErr（细化）
├── types.rs            # PyMerkleTree, PyTransaction, PyBlockHeader, PyBlock, PyLightClient, PyMptProof
├── crypto.rs           # PySecretKey, PyPublicKey, hash 函数
├── storage.rs          # PyInMemoryStorage, PyCachedStorage, PyRocksDBEngine
├── mempool.rs          # PyMempool（新建）
├── block_producer.rs   # PyBlockBuilder, produce_block（新建）
├── evm.rs              # PyEvmState, PyEvmExecutor, PyExecutionResult（新建）
├── consensus.rs        # PyConsensusEngine, PyBlockTree, PyVote, PyQC...（新建）
├── p2p.rs              # PyNode, PyNodeConfig, PyMessage...（新建）
└── rpc.rs              # PyRpcState, PyRpcServer（新建）

src/kilnchain/
├── __init__.py
├── types.py            # Pydantic 输入模型
├── client.py           # open_db()
├── vm.py               # EVM DSL（Phase PA-06）
├── node.py             # NodeBuilder / BlockchainNode（Phase PA-06）
├── events.py           # Python 侧事件总线（Phase PA-06）
└── wrappers/           # Layer 2 薄包装
    ├── __init__.py
    ├── mempool.py
    ├── evm.py
    ├── consensus.py
    ├── p2p.py
    └── rpc.py

examples/
├── minimal_chain.py
├── p2p_network.py
└── contract_counter.py
```

---

## 九、兼容性承诺

- **Layer 1（PyO3 绑定）**：与 Rust 公共 API 保持一一对应，Rust API 变更时同步更新。
- **Layer 2/3（Python 包装）**：遵循语义化版本控制，非大版本不破坏向后兼容。
- **Python 版本**：继续支持 3.10 / 3.11 / 3.12。

---

*文档版本：v0.1.0-draft*  
*维护者：工程团队*
