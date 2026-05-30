//! Chainforge 区块生产者。

use chainforge_core::block::{Block, BlockHeader};
use chainforge_core::tx::Transaction;

/// 区块构建器。
pub struct BlockBuilder {
    parent_hash: [u8; 32],
    number: u64,
    timestamp: u64,
    extra_data: Vec<u8>,
    state_root: [u8; 32],
    transactions: Vec<Transaction>,
    gas_limit: u64,
}

impl BlockBuilder {
    pub fn new(parent_hash: [u8; 32], number: u64) -> Self {
        BlockBuilder {
            parent_hash,
            number,
            timestamp: 0,
            extra_data: vec![],
            state_root: [0u8; 32],
            transactions: vec![],
            gas_limit: 30_000_000,
        }
    }

    pub fn timestamp(mut self, ts: u64) -> Self {
        self.timestamp = ts;
        self
    }

    pub fn extra_data(mut self, data: Vec<u8>) -> Self {
        self.extra_data = data;
        self
    }

    pub fn state_root(mut self, root: [u8; 32]) -> Self {
        self.state_root = root;
        self
    }

    pub fn transactions(mut self, txs: Vec<Transaction>) -> Self {
        self.transactions = txs;
        self
    }

    pub fn gas_limit(mut self, limit: u64) -> Self {
        self.gas_limit = limit;
        self
    }

    /// 构建区块（计算 txs_root）。
    pub fn build(self) -> Block {
        let mut block = Block {
            header: BlockHeader {
                parent_hash: self.parent_hash,
                number: self.number,
                timestamp: self.timestamp,
                difficulty: 0,
                nonce: 0,
                extra_data: self.extra_data,
                state_root: self.state_root,
                txs_root: [0u8; 32],
            },
            transactions: self.transactions,
            uncle_headers: vec![],
        };
        block.compute_txs_root();
        block
    }
}

/// 从 mempool 取交易并构建新区块。
pub fn produce_block(
    parent_hash: [u8; 32],
    number: u64,
    timestamp: u64,
    mempool: &mut chainforge_mempool::Mempool,
    max_txs: usize,
) -> Block {
    let txs = mempool.pop_highest_priority(max_txs);
    BlockBuilder::new(parent_hash, number)
        .timestamp(timestamp)
        .transactions(txs)
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chainforge_core::tx::Transaction;

    fn make_tx(nonce: u64, gas_price: u128) -> Transaction {
        Transaction {
            nonce,
            gas_price,
            gas_limit: 21000,
            to: Some([1u8; 20]),
            value: 100,
            data: vec![],
            v: 27,
            r: [0u8; 32],
            s: [0u8; 32],
        }
    }

    #[test]
    fn test_block_builder() {
        let txs = vec![make_tx(0, 10), make_tx(1, 20)];
        let block = BlockBuilder::new([0u8; 32], 1)
            .timestamp(1234567890)
            .transactions(txs)
            .build();

        assert_eq!(block.header.number, 1);
        assert_eq!(block.header.timestamp, 1234567890);
        assert_eq!(block.transactions.len(), 2);
        assert_ne!(block.header.txs_root, [0u8; 32]);
    }

    #[test]
    fn test_produce_block_from_mempool() {
        let mut mempool = chainforge_mempool::Mempool::new();
        mempool.insert(make_tx(0, 100));
        mempool.insert(make_tx(1, 50));
        mempool.insert(make_tx(2, 200));

        let block = produce_block([0u8; 32], 1, 1000, &mut mempool, 2);

        assert_eq!(block.transactions.len(), 2);
        // 应优先取 gas_price 最高的两笔
        assert!(block.transactions[0].gas_price >= block.transactions[1].gas_price);
        assert_eq!(mempool.len(), 1); // 剩余 1 笔
    }
}
