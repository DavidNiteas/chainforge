//! 轻客户端：仅同步区块头，验证交易包含证明。

use crate::block::BlockHeader;
use crate::merkle::{MerkleProof, MerkleTree};
use chainforge_error::ChainforgeError;

/// 轻客户端状态。
#[derive(Debug, Clone)]
pub struct LightClient {
    /// 已验证的区块头链（按高度排序）
    headers: Vec<BlockHeader>,
    /// 信任的创世区块哈希（保留用于未来安全模型扩展）
    #[allow(dead_code)]
    trusted_genesis: [u8; 32],
}

impl LightClient {
    /// 从信任的创世区块头创建轻客户端。
    pub fn new(genesis: BlockHeader) -> Self {
        let hash = genesis.hash();
        LightClient {
            headers: vec![genesis],
            trusted_genesis: hash,
        }
    }

    /// 添加并验证新区块头。
    ///
    /// 验证规则：
    /// 1. 父哈希必须与当前链尾匹配
    /// 2. 高度必须连续（number = last.number + 1）
    pub fn add_header(&mut self, header: BlockHeader) -> Result<(), ChainforgeError> {
        let last = self.headers.last().unwrap();
        if header.parent_hash != last.hash() {
            return Err(ChainforgeError::InvalidParameter(
                "parent hash mismatch".to_string(),
            ));
        }
        if header.number != last.number + 1 {
            return Err(ChainforgeError::InvalidParameter(
                "block number not sequential".to_string(),
            ));
        }
        self.headers.push(header);
        Ok(())
    }

    /// 返回最新验证的区块头。
    pub fn latest_header(&self) -> &BlockHeader {
        self.headers.last().unwrap()
    }

    /// 按高度查询区块头。
    pub fn get_header_by_number(&self, number: u64) -> Option<&BlockHeader> {
        let genesis_number = self.headers.first()?.number;
        let idx = (number - genesis_number) as usize;
        self.headers.get(idx)
    }

    /// 返回当前链高度（相对 genesis）。
    pub fn len(&self) -> usize {
        self.headers.len()
    }

    /// 链是否为空（理论上永远不会，因为至少包含 genesis）。
    pub fn is_empty(&self) -> bool {
        self.headers.is_empty()
    }

    /// 验证一笔交易是否包含在指定区块中。
    ///
    /// # 参数
    /// - `block_number`: 目标区块高度
    /// - `tx_hash`: 交易哈希（作为 Merkle 叶子）
    /// - `proof`: Merkle 证明
    pub fn verify_transaction(
        &self,
        block_number: u64,
        tx_hash: &[u8; 32],
        proof: &MerkleProof,
    ) -> Result<bool, ChainforgeError> {
        let header = self
            .get_header_by_number(block_number)
            .ok_or_else(|| ChainforgeError::InvalidParameter("block not found".to_string()))?;
        Ok(MerkleTree::verify(&header.txs_root, tx_hash, proof))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block::BlockHeader;

    fn make_header(number: u64, parent_hash: [u8; 32]) -> BlockHeader {
        BlockHeader {
            parent_hash,
            number,
            timestamp: 0,
            difficulty: 0,
            nonce: 0,
            extra_data: vec![],
            state_root: [0u8; 32],
            txs_root: [0u8; 32],
        }
    }

    #[test]
    fn test_light_client_genesis() {
        let genesis = make_header(0, [0u8; 32]);
        let lc = LightClient::new(genesis.clone());
        assert_eq!(lc.len(), 1);
        assert_eq!(lc.latest_header().number, 0);
    }

    #[test]
    fn test_add_valid_header() {
        let genesis = make_header(0, [0u8; 32]);
        let mut lc = LightClient::new(genesis);
        let h1 = make_header(1, lc.latest_header().hash());
        assert!(lc.add_header(h1).is_ok());
        assert_eq!(lc.len(), 2);
    }

    #[test]
    fn test_add_header_parent_mismatch() {
        let genesis = make_header(0, [0u8; 32]);
        let mut lc = LightClient::new(genesis);
        let bad = make_header(1, [0xffu8; 32]);
        assert!(lc.add_header(bad).is_err());
    }

    #[test]
    fn test_add_header_number_skip() {
        let genesis = make_header(0, [0u8; 32]);
        let mut lc = LightClient::new(genesis);
        let h1 = make_header(1, lc.latest_header().hash());
        lc.add_header(h1).unwrap();
        let h3 = make_header(3, lc.latest_header().hash());
        assert!(lc.add_header(h3).is_err());
    }

    #[test]
    fn test_verify_transaction() {
        // 构造一个包含 4 笔交易的区块，计算 txs_root
        let tx_hashes: Vec<[u8; 32]> = (0..4).map(|i| [i as u8; 32]).collect();
        let tree = MerkleTree::new(tx_hashes.clone());
        let root = tree.root();

        let genesis = make_header(0, [0u8; 32]);
        let mut lc = LightClient::new(genesis);
        let mut h1 = make_header(1, lc.latest_header().hash());
        h1.txs_root = root;
        lc.add_header(h1).unwrap();

        let proof = tree.proof(2).unwrap();
        assert!(lc.verify_transaction(1, &tx_hashes[2], &proof).unwrap());

        // 错误的交易哈希应失败
        let bad_hash = [0xffu8; 32];
        assert!(!lc.verify_transaction(1, &bad_hash, &proof).unwrap());
    }
}
