//! 区块树（分叉选择）。

use std::collections::HashMap;

use chainforge_core::block::Block;

use crate::vote::QuorumCertificate;

/// 区块节点。
#[derive(Clone, Debug)]
pub struct BlockNode {
    pub block: Block,
    pub parent: Option<[u8; 32]>,
    pub prepare_qc: Option<QuorumCertificate>,
    pub precommit_qc: Option<QuorumCertificate>,
    pub commit_qc: Option<QuorumCertificate>,
}

/// 区块树。
#[derive(Clone)]
pub struct BlockTree {
    nodes: HashMap<[u8; 32], BlockNode>,
    /// 已提交的区块哈希列表。
    committed: Vec<[u8; 32]>,
    /// 当前 locked 的 QC（Prepare 阶段）。
    pub locked_qc: Option<QuorumCertificate>,
    /// 当前 view  number。
    pub view: u64,
}

impl Default for BlockTree {
    fn default() -> Self {
        Self::new()
    }
}

impl BlockTree {
    pub fn new() -> Self {
        BlockTree {
            nodes: HashMap::new(),
            committed: vec![],
            locked_qc: None,
            view: 0,
        }
    }

    /// 插入新区块（携带其 Prepare-QC）。
    pub fn insert(&mut self, block: Block, prepare_qc: QuorumCertificate) {
        let hash = block.header.hash();
        let parent = if block.header.number == 0 {
            None
        } else {
            Some(block.header.parent_hash)
        };

        self.nodes.insert(
            hash,
            BlockNode {
                block,
                parent,
                prepare_qc: Some(prepare_qc),
                precommit_qc: None,
                commit_qc: None,
            },
        );
    }

    /// 为指定区块添加 PreCommit-QC。
    pub fn add_precommit_qc(&mut self, hash: &[u8; 32], qc: QuorumCertificate) {
        if let Some(node) = self.nodes.get_mut(hash) {
            node.precommit_qc = Some(qc);
        }
    }

    /// 为指定区块添加 Commit-QC（触发提交）。
    pub fn add_commit_qc(&mut self, hash: &[u8; 32], qc: QuorumCertificate) {
        if let Some(node) = self.nodes.get_mut(hash) {
            node.commit_qc = Some(qc.clone());
        }
        // 提交该区块及其所有祖先
        self.commit_chain(hash);
    }

    /// 从指定区块回溯到 genesis，提交路径上所有未提交的区块。
    fn commit_chain(&mut self, hash: &[u8; 32]) {
        let mut to_commit = vec![];
        let mut current = *hash;

        while let Some(node) = self.nodes.get(&current) {
            if self.committed.contains(&current) {
                break;
            }
            to_commit.push(current);
            match node.parent {
                Some(p) => current = p,
                None => break,
            }
        }

        to_commit.reverse();
        for h in to_commit {
            self.committed.push(h);
        }
    }

    /// 获取已提交的区块列表。
    pub fn committed_blocks(&self) -> Vec<&Block> {
        self.committed
            .iter()
            .filter_map(|h| self.nodes.get(h).map(|n| &n.block))
            .collect()
    }

    /// 获取指定区块。
    pub fn get(&self, hash: &[u8; 32]) -> Option<&BlockNode> {
        self.nodes.get(hash)
    }

    /// 获取最高已提交区块的高度。
    pub fn committed_height(&self) -> u64 {
        self.committed_blocks()
            .last()
            .map(|b| b.header.number)
            .unwrap_or(0)
    }
}
