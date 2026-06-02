//! Kilnchain 交易内存池（Mempool）。

use std::collections::{BTreeMap, HashMap};

use kilnchain_core::tx::Transaction;

/// 交易池。
pub struct Mempool {
    /// 所有待处理交易（按交易哈希索引）。
    txs: HashMap<[u8; 32], Transaction>,
    /// 按 gas_price 排序的优先级索引：gas_price → [tx_hash]。
    priority: BTreeMap<u128, Vec<[u8; 32]>>,
    /// 按账户跟踪 nonce：sender_address → {nonce → tx_hash}。
    account_nonces: HashMap<[u8; 20], BTreeMap<u64, [u8; 32]>>,
    /// 容量上限。
    max_size: usize,
}

impl Default for Mempool {
    fn default() -> Self {
        Self::with_capacity(10_000)
    }
}

impl Mempool {
    pub fn new() -> Self {
        Self::with_capacity(10_000)
    }

    pub fn with_capacity(max_size: usize) -> Self {
        Mempool {
            txs: HashMap::new(),
            priority: BTreeMap::new(),
            account_nonces: HashMap::new(),
            max_size,
        }
    }

    /// 插入交易。
    ///
    /// 如果池子已满，先淘汰 gas_price 最低的交易。
    pub fn insert(&mut self, tx: Transaction) {
        // 容量控制：淘汰最低 gas_price 的交易
        if self.txs.len() >= self.max_size {
            self.evict_lowest_priority();
        }

        let hash = tx.hash();
        let gas_price = tx.gas_price;
        let sender = Self::extract_sender(&tx);
        let nonce = tx.nonce;

        self.txs.insert(hash, tx);
        self.priority.entry(gas_price).or_default().push(hash);
        self.account_nonces
            .entry(sender)
            .or_default()
            .insert(nonce, hash);
    }

    /// 获取指定账户的下一个期望 nonce（当前最大 nonce + 1，或 0）。
    pub fn next_nonce(&self, sender: &[u8; 20]) -> u64 {
        self.account_nonces
            .get(sender)
            .and_then(|nonces| nonces.keys().last().copied())
            .map(|n| n + 1)
            .unwrap_or(0)
    }

    /// 检查交易 nonce 是否连续（可以作为入池前置条件）。
    pub fn is_nonce_valid(&self, tx: &Transaction) -> bool {
        let sender = Self::extract_sender(tx);
        let expected = self.next_nonce(&sender);
        tx.nonce == expected
            || self
                .account_nonces
                .get(&sender)
                .is_some_and(|m| m.contains_key(&tx.nonce))
    }

    /// 淘汰 gas_price 最低的一笔交易。
    fn evict_lowest_priority(&mut self) {
        let hash_to_remove = self
            .priority
            .iter_mut()
            .next()
            .and_then(|(_, hashes)| hashes.pop());

        if let Some(hash) = hash_to_remove {
            if let Some(tx) = self.txs.remove(&hash) {
                let sender = Self::extract_sender(&tx);
                if let Some(nonces) = self.account_nonces.get_mut(&sender) {
                    nonces.remove(&tx.nonce);
                    if nonces.is_empty() {
                        self.account_nonces.remove(&sender);
                    }
                }
            }
            // 清理空的 gas_price 桶
            self.priority.retain(|_, hashes| !hashes.is_empty());
        }
    }

    fn extract_sender(tx: &Transaction) -> [u8; 20] {
        // 简化：使用 to 字段作为 sender 代理（实际应从签名恢复）
        // 在真实实现中应使用 recover_sender()
        tx.to.unwrap_or([0u8; 20])
    }

    /// 按哈希查询交易。
    pub fn get(&self, hash: &[u8; 32]) -> Option<&Transaction> {
        self.txs.get(hash)
    }

    /// 移除交易。
    pub fn remove(&mut self, hash: &[u8; 32]) -> Option<Transaction> {
        let tx = self.txs.remove(hash)?;
        let gas_price = tx.gas_price;
        if let Some(hashes) = self.priority.get_mut(&gas_price) {
            hashes.retain(|h| h != hash);
            if hashes.is_empty() {
                self.priority.remove(&gas_price);
            }
        }
        let sender = Self::extract_sender(&tx);
        if let Some(nonces) = self.account_nonces.get_mut(&sender) {
            nonces.remove(&tx.nonce);
            if nonces.is_empty() {
                self.account_nonces.remove(&sender);
            }
        }
        Some(tx)
    }

    /// 检查交易是否存在。
    pub fn contains(&self, hash: &[u8; 32]) -> bool {
        self.txs.contains_key(hash)
    }

    /// 返回交易数量。
    pub fn len(&self) -> usize {
        self.txs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.txs.is_empty()
    }

    /// 获取所有交易的只读引用。
    pub fn txs(&self) -> &HashMap<[u8; 32], Transaction> {
        &self.txs
    }

    /// 取出 gas_price 最高的 `limit` 笔交易（从高到低）。
    pub fn pop_highest_priority(&mut self, limit: usize) -> Vec<Transaction> {
        let mut result = Vec::with_capacity(limit);
        let mut empty_prices = Vec::new();

        for (&price, hashes) in self.priority.iter_mut().rev() {
            while result.len() < limit && !hashes.is_empty() {
                let hash = hashes.pop().unwrap();
                if let Some(tx) = self.txs.remove(&hash) {
                    result.push(tx);
                }
            }
            if hashes.is_empty() {
                empty_prices.push(price);
            }
            if result.len() >= limit {
                break;
            }
        }

        for price in empty_prices {
            self.priority.remove(&price);
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_tx(nonce: u64) -> Transaction {
        Transaction {
            nonce,
            gas_price: 10,
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
    fn test_insert_and_get() {
        let mut pool = Mempool::new();
        let tx = make_tx(1);
        let hash = tx.hash();

        pool.insert(tx);
        assert!(pool.contains(&hash));
        assert_eq!(pool.len(), 1);

        let retrieved = pool.get(&hash).unwrap();
        assert_eq!(retrieved.nonce, 1);
    }

    #[test]
    fn test_remove() {
        let mut pool = Mempool::new();
        let tx = make_tx(2);
        let hash = tx.hash();

        pool.insert(tx);
        assert_eq!(pool.len(), 1);

        let removed = pool.remove(&hash);
        assert!(removed.is_some());
        assert!(pool.is_empty());
        assert!(!pool.contains(&hash));
    }

    #[test]
    fn test_duplicate_insert_overwrites() {
        let mut pool = Mempool::new();
        let tx1 = make_tx(3);
        let _hash = tx1.hash();

        pool.insert(tx1);
        // 相同内容的交易再次插入（hash 相同）
        let tx2 = make_tx(3);
        pool.insert(tx2);

        assert_eq!(pool.len(), 1);
    }

    #[test]
    fn test_priority_queue() {
        let mut pool = Mempool::new();

        let mut tx_low = make_tx(1);
        tx_low.gas_price = 10;

        let mut tx_high = make_tx(2);
        tx_high.gas_price = 100;

        let mut tx_mid = make_tx(3);
        tx_mid.gas_price = 50;

        pool.insert(tx_low);
        pool.insert(tx_high);
        pool.insert(tx_mid);

        let selected = pool.pop_highest_priority(2);
        assert_eq!(selected.len(), 2);
        // 应先返回 gas_price 最高的
        assert_eq!(selected[0].gas_price, 100);
        assert_eq!(selected[1].gas_price, 50);
        assert_eq!(pool.len(), 1); // 剩余 1 笔
    }

    #[test]
    fn test_remove_updates_priority() {
        let mut pool = Mempool::new();
        let tx = make_tx(1);
        let hash = tx.hash();

        pool.insert(tx);
        assert_eq!(pool.len(), 1);

        pool.remove(&hash);
        assert!(pool.is_empty());

        let selected = pool.pop_highest_priority(1);
        assert!(selected.is_empty());
    }

    #[test]
    fn test_account_nonce_tracking() {
        let mut pool = Mempool::new();
        let sender = [0xabu8; 20];

        let mut tx1 = make_tx(0);
        tx1.to = Some(sender);

        let mut tx2 = make_tx(1);
        tx2.to = Some(sender);

        pool.insert(tx1);
        pool.insert(tx2);

        assert_eq!(pool.next_nonce(&sender), 2);

        let mut tx3 = make_tx(2);
        tx3.to = Some(sender);
        assert!(pool.is_nonce_valid(&tx3));
    }

    #[test]
    fn test_capacity_eviction() {
        let mut pool = Mempool::with_capacity(3);

        let mut tx1 = make_tx(0);
        tx1.gas_price = 10;
        tx1.to = Some([0x01; 20]);

        let mut tx2 = make_tx(0);
        tx2.gas_price = 20;
        tx2.to = Some([0x02; 20]);

        let mut tx3 = make_tx(0);
        tx3.gas_price = 30;
        tx3.to = Some([0x03; 20]);

        let mut tx4 = make_tx(0);
        tx4.gas_price = 40;
        tx4.to = Some([0x04; 20]);

        pool.insert(tx1);
        pool.insert(tx2);
        pool.insert(tx3);
        assert_eq!(pool.len(), 3);

        // 插入第 4 笔，应淘汰 gas_price 最低的 tx1
        pool.insert(tx4);
        assert_eq!(pool.len(), 3);
        assert!(!pool.txs().values().any(|tx| tx.gas_price == 10));
    }
}
