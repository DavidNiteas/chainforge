# CON-01: 单节点共识状态机

## 目标

实现 HotStuff 共识的核心数据结构：BlockTree、Vote、QuorumCertificate，以及单节点视角的状态转换逻辑。本阶段不实现网络通信，仅完成内存中的状态机。

## 交付物

### 源码

| 文件 | 说明 |
|------|------|
| `crates/kilnchain-consensus/Cargo.toml` | crate 配置 |
| `crates/kilnchain-consensus/src/lib.rs` | 模块导出 |
| `crates/kilnchain-consensus/src/vote.rs` | `Vote`、`QuorumCertificate` |
| `crates/kilnchain-consensus/src/block_tree.rs` | `BlockTree` 分叉管理 |
| `crates/kilnchain-consensus/src/safety.rs` | 安全规则（锁定、提交） |

### 测试

| 文件 | 说明 |
|------|------|
| `crates/kilnchain-consensus/src/block_tree.rs` (内联测试) | 区块插入、QC 形成、提交 |

## 核心代码规格

### QuorumCertificate

```rust
pub struct QuorumCertificate {
    pub block_hash: [u8; 32],
    pub view_number: u64,
    pub phase: Phase, // Prepare / PreCommit / Commit / Decide
    pub signatures: Vec<(PeerId, Signature)>,
}

impl QuorumCertificate {
    pub fn verify(&self, public_keys: &[PublicKey], quorum: usize) -> bool {
        // 验证 2f+1 个签名的有效性
    }
}
```

### BlockTree

```rust
pub struct BlockTree {
    /// 所有已知区块
    blocks: HashMap<[u8; 32], Block>,
    /// 每个区块的父指针
    parent: HashMap<[u8; 32], [u8; 32]>,
    /// 每个区块高度上的最佳 QC
    qcs: HashMap<(u64, Phase), QuorumCertificate>,
    /// 已提交的区块链
    committed: Vec<Block>,
    /// 当前锁定的高度
    locked_view: u64,
}

impl BlockTree {
    /// 插入新区块
    pub fn insert(&mut self, block: Block, parent_qc: QuorumCertificate);
    
    /// 对某个区块投票形成 QC
    pub fn add_vote(&mut self, vote: Vote) -> Option<QuorumCertificate>;
    
    /// 获取从指定区块到 genesis 的路径
    pub fn chain_from(&self, hash: &[u8; 32]) -> Vec<Block>;
    
    /// 提交某个 QC 对应的区块及其祖先
    pub fn commit(&mut self, qc: QuorumCertificate);
}
```

### 安全规则

```rust
pub struct SafetyRules {
    locked_view: u64,
    locked_qc: Option<QuorumCertificate>,
}

impl SafetyRules {
    /// 判断是否可以投票给某个提案
    pub fn can_vote(&self, proposal: &Block, qc: &QuorumCertificate) -> bool {
        // 1. 提案的 view 必须大于 locked_view
        // 2. 提案携带的 QC 必须有效
    }
}
```

## 验收标准

- [ ] `cargo test -p kilnchain-consensus` 通过
- [ ] 能构建 5 个区块的链并依次提交
- [ ] 分叉场景下，只提交拥有最强 QC 的分支
- [ ] 对无有效父 QC 的区块拒绝投票

## 预计工时

2 ~ 3 天

## 前置依赖

Phase 01 ~ 11

## 下一步

CON-02: 多节点 Prepare 阶段
