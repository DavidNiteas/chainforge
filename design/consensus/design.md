# 共识算法设计

## 目标

为 Kilnchain 实现拜占庭容错（BFT）共识算法，确保在部分节点故障或作恶的情况下，网络仍能就区块顺序达成一致。选择 **HotStuff** 作为核心算法（类 LibraBFT / DiemBFT），因其流水线设计、线性通信复杂度和成熟的工程实践。

## 技术选型

| 组件 | 技术 | 理由 |
|------|------|------|
| 共识算法 | HotStuff | 流水线 BFT，线性复杂度，Chained HotStuff 优化 |
| 密码学签名 | Secp256k1（已有） | 复用 kilnchain-crypto 的 ECDSA |
|  quorum 证书 | 聚合签名（可选） | 降低通信量，可先使用多签集合 |
| 领导者轮换 | 轮询 + VRF（可选） | 简单轮询起步，VRF 增强公平性 |
| 状态机复制 | 与 kilnchain-storage 集成 | 共识决定的区块直接写入存储 |

## 核心概念

### Chained HotStuff 三阶段

```
NewView → Prepare → PreCommit → Commit → Decide
    ↑        ↑          ↑          ↑        ↑
   视图    提案投票    锁定区块    最终确认  执行区块
```

- **Prepare**：领导者提议区块，收集 2f+1 个 PrepareVote → 形成 QC
- **PreCommit**：对 Prepare-QC 投票，形成 PreCommit-QC
- **Commit**：对 PreCommit-QC 投票，形成 Commit-QC（此时区块被锁定）
- **Decide**：对 Commit-QC 投票，区块最终确认并执行

### 流水线化

Chained HotStuff 将四阶段流水线化，每个阶段同时处理不同高度的区块：

```
Height=N:   Prepare    →  PreCommit  →  Commit     →  Decide
Height=N+1:            Prepare      →  PreCommit   →  Commit
Height=N+2:                         Prepare        →  PreCommit
```

## 模块划分

### `kilnchain-consensus` crate（新增）

```
crates/kilnchain-consensus/
├── Cargo.toml
└── src/
    ├── lib.rs          # 模块导出
    ├── hotstuff.rs     # Chained HotStuff 核心状态机
    ├── replica.rs      # 副本（非领导者节点）逻辑
    ├── leader.rs       # 领导者提议逻辑
    ├── vote.rs         # Vote 消息、QC 构造与验证
    ├── pacemaker.rs    # 视图超时、领导者轮换
    ├── block_tree.rs   # 区块树（分叉选择规则）
    └── safety.rs       # 安全规则（锁定、提交条件）
```

### 核心结构

```rust
pub struct ConsensusEngine {
    config: NodeConfig,
    block_tree: BlockTree,
    pacemaker: Pacemaker,
    network: NetworkAdapter,  // 与 kilnchain-p2p 交互
    storage: Arc<dyn StorageEngine>,
}

pub struct QuorumCertificate {
    block_hash: [u8; 32],
    view_number: u64,
    signatures: Vec<Signature>,  // 2f+1 个签名
}

pub struct BlockTree {
    pending: HashMap<[u8; 32], Block>,
    locked: Option<QuorumCertificate>,
    committed: Vec<Block>,
}
```

## 与现有模块的交互

```
ConsensusEngine
  ├── 通过 NetworkAdapter 发送/接收共识消息（依赖 kilnchain-p2p）
  ├── 从 mempool 获取待打包交易（依赖 kilnchain-core Transaction）
  ├── 将确认区块写入存储（依赖 kilnchain-storage）
  └── 使用 Secp256k1 签名/验证投票（依赖 kilnchain-crypto）
```

## 迭代阶段划分

| 阶段 | 目标 | 交付物 |
|------|------|--------|
| CON-01 | 单节点共识状态机 | BlockTree、Vote、QC 基础结构 |
| CON-02 | 多节点 Prepare 阶段 | 领导者提议 + 副本投票 + QC 形成 |
| CON-03 | 完整四阶段流水线 | Chained HotStuff 全部阶段 |
| CON-04 | Pacemaker 与视图切换 | 超时处理、领导者轮换、活性保证 |
| CON-05 | 安全性验证 | 双重投票检测、锁定规则、分叉选择 |
| CON-06 | 容错测试 | 1/3 节点崩溃/作恶场景测试 |

## 验收标准

- [ ] 4 节点网络中，3 个诚实节点能持续出块
- [ ] 1 个拜占庭节点无法导致双花或分叉
- [ ] 领导者掉线后，视图能在 5 秒内切换并恢复出块
- [ ] 网络分区恢复后，能自动切换到最长合法链
