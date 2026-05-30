# EVM 兼容执行层设计

## 目标

为 Chainforge 添加以太坊虚拟机（EVM）兼容的执行层，使现有 Solidity 智能合约能够部署和运行。这是区块链平台从「账本」升级为「可编程平台」的关键步骤。

## 技术选型

| 组件 | 技术 | 理由 |
|------|------|------|
| EVM 解释器 | `revm` (Rust) | 纯 Rust、高性能、活跃维护、Apache-2.0 |
| 账户模型 | Ethereum 状态模型 | 兼容现有钱包和工具 |
| 状态 trie | 稀疏 Merkle Patricia Trie（MPT） | 以太坊标准，支持状态证明 |
| Gas 计费 | 与 revm 集成 | 复用成熟计费表 |
| 合约字节码 | EVM bytecode（Solidity 编译输出） | 完全兼容 |

## 模块划分

### `chainforge-evm` crate（新增）

```
crates/chainforge-evm/
├── Cargo.toml
└── src/
    ├── lib.rs           # 模块导出
    ├── executor.rs      # 交易执行引擎（调用 revm）
    ├── state.rs         # 账户状态管理（balance, nonce, code, storage）
    ├── mpt.rs           # Merkle Patricia Trie 实现
    ├── state_trie.rs    # 全局状态根计算
    ├── gas.rs           # Gas 限制与计费（复用 revm）
    └── precompiles.rs   # 预编译合约（ecrecover、sha256 等）
```

### 核心结构

```rust
pub struct AccountState {
    pub nonce: u64,
    pub balance: U256,
    pub code_hash: [u8; 32],     // 合约代码哈希
    pub storage_root: [u8; 32],  // 存储 trie 根
}

pub struct StateManager {
    trie: MerklePatriciaTrie,
    db: Arc<dyn StorageEngine>,
}

pub struct EvmExecutor {
    state: StateManager,
    config: EvmConfig,
}
```

## 与现有模块的交互

```
EvmExecutor
  ├── 接收 Transaction → 解析 to/data/value → 调用 revm
  ├── 读写账户状态 → 通过 StateManager → 底层 chainforge-storage
  ├── 计算状态根 → MerklePatriciaTrie → 更新 BlockHeader.state_root
  ├── 使用 chainforge-crypto 的 keccak256 计算地址和 trie 节点
  └── 区块确认后，状态变更批量写入持久存储
```

## 执行流程

```
1. 从 mempool 取出交易
2. 检查 nonce、balance、gas_limit
3. 创建 revm EVM 实例，注入 StateManager 作为 DB
4. 执行交易（transfer / contract creation / contract call）
5. 收集状态变更（touched accounts、logs、gas used）
6. 更新 Merkle Patricia Trie，计算新的 state_root
7. 将交易回执和状态变更写入存储
```

## 迭代阶段划分

| 阶段 | 目标 | 交付物 |
|------|------|--------|
| EVM-01 | 集成 revm，基础转账执行 | 简单 ETH transfer，状态更新 |
| EVM-02 | 账户状态模型 + MPT 骨架 | AccountState、MPT 插入/查询 |
| EVM-03 | 合约创建与调用 | CREATE、CALL 操作码，字节码执行 |
| EVM-04 | 预编译合约 | ecrecover、sha256、 ripemd160、identity |
| EVM-05 | 完整状态根计算 | state_root、storage_root、receipt_root |
| EVM-06 | Solidity 合约部署测试 | ERC-20 合约编译部署运行测试 |

## 验收标准

- [ ] 两个账户间转账，余额正确更新
- [ ] 合约创建后，代码可通过地址查询
- [ ] 调用合约函数，状态变更和返回值正确
- [ ] 状态根在每次区块后正确更新，与参考实现一致
- [ ] Gas 限制超限交易被正确回滚
- [ ] ERC-20 合约（totalSupply/transfer/balanceOf）完整运行
