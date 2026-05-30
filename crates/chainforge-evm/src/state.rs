//! EVM 状态管理器。

use std::collections::HashMap;

use chainforge_error::ChainforgeError;
use revm::primitives::{Account, AccountInfo, Address, Bytecode, B256, U256};
use revm::{Database, DatabaseCommit, DatabaseRef};

/// 内存中的 EVM 状态数据库（用于测试和轻量场景）。
#[derive(Clone, Debug, Default)]
pub struct InMemoryEvmState {
    accounts: HashMap<Address, AccountInfo>,
    storage: HashMap<(Address, U256), U256>,
    block_hashes: HashMap<u64, B256>,
}

impl InMemoryEvmState {
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置账户余额。
    pub fn set_balance(&mut self, address: Address, balance: U256) {
        let info = self.accounts.entry(address).or_insert_with(|| AccountInfo {
            balance: U256::ZERO,
            nonce: 0,
            code_hash: B256::ZERO,
            code: None,
        });
        info.balance = balance;
    }

    /// 设置账户 nonce。
    pub fn set_nonce(&mut self, address: Address, nonce: u64) {
        let info = self.accounts.entry(address).or_insert_with(|| AccountInfo {
            balance: U256::ZERO,
            nonce: 0,
            code_hash: B256::ZERO,
            code: None,
        });
        info.nonce = nonce;
    }

    /// 设置合约代码。
    pub fn set_code(&mut self, address: Address, code: Bytecode) {
        let mut info = AccountInfo {
            balance: U256::ZERO,
            nonce: 1,
            code_hash: B256::ZERO,
            code: Some(code.clone()),
        };
        info.code_hash = code.hash_slow();
        self.accounts.insert(address, info);
    }

    /// 设置存储槽。
    pub fn set_storage(&mut self, address: Address, slot: U256, value: U256) {
        self.storage.insert((address, slot), value);
    }

    /// 设置区块哈希。
    pub fn set_block_hash(&mut self, number: u64, hash: B256) {
        self.block_hashes.insert(number, hash);
    }

    /// 查询账户余额。
    pub fn balance(&self, address: Address) -> U256 {
        self.accounts
            .get(&address)
            .map(|info| info.balance)
            .unwrap_or_default()
    }

    /// 查询账户 nonce。
    pub fn nonce(&self, address: Address) -> u64 {
        self.accounts
            .get(&address)
            .map(|info| info.nonce)
            .unwrap_or_default()
    }

    /// 查询合约字节码。
    pub fn code(&self, address: Address) -> Vec<u8> {
        self.accounts
            .get(&address)
            .and_then(|info| info.code.as_ref())
            .map(|c| c.original_bytes().to_vec())
            .unwrap_or_default()
    }
}

impl Database for InMemoryEvmState {
    type Error = ChainforgeError;

    fn basic(&mut self, address: Address) -> Result<Option<AccountInfo>, Self::Error> {
        Ok(self.accounts.get(&address).cloned())
    }

    fn code_by_hash(&mut self, _code_hash: B256) -> Result<Bytecode, Self::Error> {
        Ok(Bytecode::default())
    }

    fn storage(&mut self, address: Address, index: U256) -> Result<U256, Self::Error> {
        Ok(self
            .storage
            .get(&(address, index))
            .copied()
            .unwrap_or_default())
    }

    fn block_hash(&mut self, number: u64) -> Result<B256, Self::Error> {
        Ok(self.block_hashes.get(&number).copied().unwrap_or_default())
    }
}

impl DatabaseCommit for InMemoryEvmState {
    fn commit(&mut self, changes: revm::primitives::HashMap<Address, Account>) {
        for (addr, account) in changes {
            if account.is_empty() {
                self.accounts.remove(&addr);
            } else {
                self.accounts.insert(
                    addr,
                    AccountInfo {
                        balance: account.info.balance,
                        nonce: account.info.nonce,
                        code_hash: account.info.code_hash,
                        code: account.info.code,
                    },
                );
                for (slot, value) in account.storage {
                    self.storage.insert((addr, slot), value.present_value());
                }
            }
        }
    }
}

impl DatabaseRef for InMemoryEvmState {
    type Error = ChainforgeError;

    fn basic_ref(&self, address: Address) -> Result<Option<AccountInfo>, Self::Error> {
        Ok(self.accounts.get(&address).cloned())
    }

    fn code_by_hash_ref(&self, _code_hash: B256) -> Result<Bytecode, Self::Error> {
        Ok(Bytecode::default())
    }

    fn storage_ref(&self, address: Address, index: U256) -> Result<U256, Self::Error> {
        Ok(self
            .storage
            .get(&(address, index))
            .copied()
            .unwrap_or_default())
    }

    fn block_hash_ref(&self, number: u64) -> Result<B256, Self::Error> {
        Ok(self.block_hashes.get(&number).copied().unwrap_or_default())
    }
}
