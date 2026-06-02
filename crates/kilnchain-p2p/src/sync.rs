//! 区块/交易同步管理器。

use std::collections::HashMap;

use kilnchain_core::block::Block;
use kilnchain_core::tx::Transaction;

/// 同步状态。
#[derive(Clone, Debug, Default)]
pub struct SyncState {
    /// 当前本地链高度。
    pub local_height: u64,
    /// 已知的最高远程高度。
    pub remote_height: u64,
    /// 正在请求的区块范围。
    pub pending_requests: Vec<(u64, u64)>,
}

/// 区块同步管理器。
pub struct SyncManager {
    state: SyncState,
    /// 缓存已接收但未连接的区块（按高度索引）。
    pending_blocks: HashMap<u64, Block>,
    /// 缓存已接收的交易。
    pending_txs: Vec<Transaction>,
}

impl Default for SyncManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SyncManager {
    pub fn new() -> Self {
        SyncManager {
            state: SyncState::default(),
            pending_blocks: HashMap::new(),
            pending_txs: Vec::new(),
        }
    }

    pub fn state(&self) -> &SyncState {
        &self.state
    }

    /// 接收到新区块广播时的处理逻辑。
    ///
    /// 返回 `Some((from, to))` 表示需要请求缺失的区块范围。
    pub fn on_new_block(&mut self, block: &Block) -> Option<(u64, u64)> {
        let block_num = block.header.number;

        if block_num <= self.state.local_height {
            // 已知的旧区块，忽略
            return None;
        }

        if block_num == self.state.local_height + 1 {
            // 下一个预期区块（简化：假设父哈希总是匹配）
            self.state.local_height = block_num;
            self.try_connect_pending();
            return None;
        }

        // 存在缺失区块，缓存此区块并请求缺失范围
        self.pending_blocks.insert(block_num, block.clone());
        self.state.remote_height = self.state.remote_height.max(block_num);

        let from = self.state.local_height + 1;
        let to = block_num.saturating_sub(1);
        if from <= to && !self.is_already_requesting(from, to) {
            self.state.pending_requests.push((from, to));
            return Some((from, to));
        }
        None
    }

    /// 接收到批量区块响应时的处理。
    pub fn on_block_response(&mut self, blocks: Vec<Block>) {
        for block in blocks {
            let num = block.header.number;
            if num > self.state.local_height {
                self.pending_blocks.insert(num, block);
            }
        }
        self.try_connect_pending();
    }

    /// 接收到新交易广播时的处理。
    pub fn on_new_transaction(&mut self, tx: Transaction) {
        self.pending_txs.push(tx);
    }

    /// 尝试将缓存的 pending 区块连接到主链。
    fn try_connect_pending(&mut self) {
        loop {
            let next = self.state.local_height + 1;
            if let Some(block) = self.pending_blocks.remove(&next) {
                self.state.local_height = block.header.number;
            } else {
                break;
            }
        }
    }

    /// 检查是否已经在请求指定范围。
    fn is_already_requesting(&self, from: u64, to: u64) -> bool {
        self.state
            .pending_requests
            .iter()
            .any(|(f, t)| *f == from && *t == to)
    }

    /// 获取待处理的交易（供上层批量处理）。
    pub fn drain_pending_txs(&mut self) -> Vec<Transaction> {
        std::mem::take(&mut self.pending_txs)
    }

    /// 获取缓存中尚未连接的区块数量。
    pub fn pending_block_count(&self) -> usize {
        self.pending_blocks.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kilnchain_core::block::BlockHeader;

    fn make_block(number: u64, parent_hash: [u8; 32]) -> Block {
        Block {
            header: BlockHeader {
                parent_hash,
                number,
                timestamp: 0,
                difficulty: 0,
                nonce: 0,
                extra_data: vec![],
                state_root: [0u8; 32],
                txs_root: [0u8; 32],
            },
            transactions: vec![],
            uncle_headers: vec![],
        }
    }

    #[test]
    fn test_sync_continuous_blocks() {
        let mut sync = SyncManager::new();
        let genesis = [0u8; 32];

        let block1 = make_block(1, genesis);
        let block2 = make_block(2, block1.header.hash());

        assert!(sync.on_new_block(&block1).is_none());
        assert_eq!(sync.state().local_height, 1);

        assert!(sync.on_new_block(&block2).is_none());
        assert_eq!(sync.state().local_height, 2);
    }

    #[test]
    fn test_sync_missing_blocks() {
        let mut sync = SyncManager::new();
        let genesis = [0u8; 32];

        // 直接收到 block 5，缺失 1~4
        let block5 = make_block(5, genesis);
        let request = sync.on_new_block(&block5);

        assert_eq!(request, Some((1, 4)));
        assert_eq!(sync.pending_block_count(), 1);
    }

    #[test]
    fn test_sync_backfill() {
        let mut sync = SyncManager::new();
        let genesis = [0u8; 32];

        // 先收到 block 3，请求 1~2
        let block3 = make_block(3, genesis);
        sync.on_new_block(&block3);

        // 收到 backfill 响应
        let block1 = make_block(1, genesis);
        let block2 = make_block(2, block1.header.hash());
        sync.on_block_response(vec![block1, block2]);

        assert_eq!(sync.state().local_height, 3);
        assert_eq!(sync.pending_block_count(), 0);
    }

    #[test]
    fn test_ignore_old_block() {
        let mut sync = SyncManager::new();
        let genesis = [0u8; 32];

        let block1 = make_block(1, genesis);
        sync.on_new_block(&block1);
        assert_eq!(sync.state().local_height, 1);

        // 再次收到 block 1
        assert!(sync.on_new_block(&block1).is_none());
        assert_eq!(sync.state().local_height, 1);
    }
}
