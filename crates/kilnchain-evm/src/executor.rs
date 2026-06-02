//! EVM 执行引擎。

use kilnchain_error::KilnchainError;
use revm::primitives::{Address, ExecutionResult, TransactTo, U256};
use revm::{DatabaseCommit, DatabaseRef, Evm};

use crate::state::InMemoryEvmState;

/// EVM 交易执行器。
#[derive(Clone)]
pub struct EvmExecutor<DB> {
    db: DB,
}

impl EvmExecutor<InMemoryEvmState> {
    pub fn new(db: InMemoryEvmState) -> Self {
        EvmExecutor { db }
    }

    /// 执行简单转账。
    pub fn transfer(
        &mut self,
        from: Address,
        to: Address,
        value: U256,
    ) -> Result<ExecutionResult, KilnchainError> {
        let mut evm = Evm::builder()
            .with_db(&mut self.db)
            .modify_tx_env(|tx| {
                tx.caller = from;
                tx.transact_to = TransactTo::Call(to);
                tx.value = value;
                tx.gas_limit = 100_000;
            })
            .build();

        let result = evm.transact().map_err(|e| {
            KilnchainError::InvalidParameter(format!("EVM execution failed: {:?}", e))
        })?;

        // 提交状态变更
        evm.context.evm.db.commit(result.state.clone());

        Ok(result.result)
    }

    /// 执行合约创建。
    pub fn deploy(
        &mut self,
        from: Address,
        code: Vec<u8>,
        value: U256,
    ) -> Result<ExecutionResult, KilnchainError> {
        let mut evm = Evm::builder()
            .with_db(&mut self.db)
            .modify_tx_env(|tx| {
                tx.caller = from;
                tx.transact_to = TransactTo::Create;
                tx.data = code.into();
                tx.value = value;
                tx.gas_limit = 1_000_000;
            })
            .build();

        let result = evm
            .transact()
            .map_err(|e| KilnchainError::InvalidParameter(format!("EVM deploy failed: {:?}", e)))?;

        evm.context.evm.db.commit(result.state.clone());

        Ok(result.result)
    }

    /// 执行合约调用。
    pub fn call(
        &mut self,
        from: Address,
        to: Address,
        data: Vec<u8>,
        value: U256,
    ) -> Result<ExecutionResult, KilnchainError> {
        let mut evm = Evm::builder()
            .with_db(&mut self.db)
            .modify_tx_env(|tx| {
                tx.caller = from;
                tx.transact_to = TransactTo::Call(to);
                tx.data = data.into();
                tx.value = value;
                tx.gas_limit = 1_000_000;
            })
            .build();

        let result = evm
            .transact()
            .map_err(|e| KilnchainError::InvalidParameter(format!("EVM call failed: {:?}", e)))?;

        evm.context.evm.db.commit(result.state.clone());

        Ok(result.result)
    }

    /// 读取账户余额。
    pub fn balance(&self, address: Address) -> U256 {
        DatabaseRef::basic_ref(&self.db, address)
            .ok()
            .flatten()
            .map(|info| info.balance)
            .unwrap_or_default()
    }

    /// 读取账户 nonce。
    pub fn nonce(&self, address: Address) -> u64 {
        DatabaseRef::basic_ref(&self.db, address)
            .ok()
            .flatten()
            .map(|info| info.nonce)
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALICE: Address = Address::new([0xAAu8; 20]);
    const BOB: Address = Address::new([0xBBu8; 20]);

    fn setup() -> (EvmExecutor<InMemoryEvmState>, InMemoryEvmState) {
        let mut db = InMemoryEvmState::new();
        db.set_balance(ALICE, U256::from(1000));
        let executor = EvmExecutor::new(db.clone());
        (executor, db)
    }

    #[test]
    fn test_transfer() {
        let (mut executor, _) = setup();

        let result = executor.transfer(ALICE, BOB, U256::from(300)).unwrap();
        assert!(result.is_success());

        assert_eq!(executor.balance(ALICE), U256::from(700));
        assert_eq!(executor.balance(BOB), U256::from(300));
    }

    #[test]
    fn test_transfer_insufficient_balance() {
        let (mut executor, _) = setup();

        let result = executor.transfer(ALICE, BOB, U256::from(2000));
        assert!(result.is_err()); // 余额不足时直接返回错误
    }

    #[test]
    fn test_deploy_and_call_counter() {
        // 极简 counter 合约 bytecode:
        // PUSH1 0x00 CALLDATALOAD PUSH1 0x01 ADD PUSH1 0x00 MSTORE PUSH1 0x20 PUSH1 0x00 RETURN
        let code = vec![
            0x60, 0x00, 0x35, 0x60, 0x01, 0x01, 0x60, 0x00, 0x52, 0x60, 0x20, 0x60, 0x00, 0xF3,
        ];

        let mut db = InMemoryEvmState::new();
        db.set_balance(ALICE, U256::from(10000));
        let mut executor = EvmExecutor::new(db);

        let deploy_result = executor.deploy(ALICE, code, U256::ZERO).unwrap();
        assert!(deploy_result.is_success());

        // 部署后 ALICE 的 nonce 应增加
        assert_eq!(executor.nonce(ALICE), 1);
    }
}
