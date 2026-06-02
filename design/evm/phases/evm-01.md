# EVM-01: 集成 revm，基础转账执行

## 目标

将 `revm` EVM 解释器集成到 Kilnchain 中，实现最简单的 ETH 转账交易执行，验证状态更新（余额、nonce）的正确性。

## 交付物

### 源码

| 文件 | 说明 |
|------|------|
| `crates/kilnchain-evm/Cargo.toml` | crate 配置，依赖 revm、kilnchain-storage、kilnchain-core |
| `crates/kilnchain-evm/src/lib.rs` | 模块导出 |
| `crates/kilnchain-evm/src/executor.rs` | `EvmExecutor`，封装 revm |
| `crates/kilnchain-evm/src/state.rs` | `StateManager`，适配 revm 的 Database trait |

### 测试

| 文件 | 说明 |
|------|------|
| `crates/kilnchain-evm/src/executor.rs` (内联测试) | 转账执行、余额更新验证 |

## 核心代码规格

### StateManager（实现 revm::Database）

```rust
use revm::{Database, AccountInfo};

pub struct StateManager {
    db: Arc<dyn StorageEngine>,
}

impl Database for StateManager {
    type Error = KilnchainError;

    fn basic(&mut self, address: Address) -> Result<Option<AccountInfo>, Self::Error> {
        // 从 kilnchain-storage 读取账户状态
    }

    fn code_by_hash(&mut self, code_hash: B256) -> Result<Bytecode, Self::Error> {
        // 读取合约字节码
    }

    fn storage(&mut self, address: Address, index: U256) -> Result<U256, Self::Error> {
        // 读取存储槽
    }

    fn block_hash(&mut self, number: U256) -> Result<B256, Self::Error> {
        // 读取区块哈希
    }
}
```

### EvmExecutor

```rust
pub struct EvmExecutor {
    db: StateManager,
}

impl EvmExecutor {
    pub fn execute(&mut self, tx: &Transaction) -> Result<ExecutionResult, KilnchainError> {
        let mut evm = EVM::builder()
            .with_db(&mut self.db)
            .modify_tx_env(|env| {
                env.caller = Address::from_slice(&tx.sender);
                env.transact_to = TransactTo::Call(Address::from_slice(&tx.to.unwrap_or_default()));
                env.value = U256::from(tx.value);
                env.data = tx.data.clone().into();
                env.gas_limit = tx.gas_limit;
            })
            .build();
        
        let result = evm.transact().map_err(|e| ...)?;
        Ok(result)
    }
}
```

## 验收标准

- [ ] `cargo test -p kilnchain-evm` 通过
- [ ] 账户 A（余额 100）向账户 B 转账 30，执行后 A=70, B=30
- [ ] nonce 正确递增
- [ ] 余额不足交易返回 OutOfFunds 错误
- [ ] Gas 消耗大于 0

## 预计工时

2 ~ 3 天

## 前置依赖

Phase 01 ~ 11（需要 kilnchain-core Transaction、kilnchain-storage）

## 下一步

EVM-02: 账户状态模型 + MPT 骨架
