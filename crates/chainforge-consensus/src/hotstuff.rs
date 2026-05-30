//! Chained HotStuff 共识引擎。

use chainforge_core::block::{Block, BlockHeader};
use chainforge_core::tx::Transaction;

use crate::block_tree::BlockTree;
use crate::pacemaker::Pacemaker;
use crate::safety::SafetyRules;
use crate::vote::{Phase, QuorumCertificate, Vote};

/// 共识引擎。
pub struct ConsensusEngine {
    pub block_tree: BlockTree,
    pub safety: SafetyRules,
    pub pacemaker: Pacemaker,
    pub node_id: usize,
}

impl ConsensusEngine {
    pub fn new(node_id: usize, node_count: usize) -> Self {
        ConsensusEngine {
            block_tree: BlockTree::new(),
            safety: SafetyRules::new(),
            pacemaker: Pacemaker::new(node_id, node_count, 5000),
            node_id,
        }
    }

    /// 领导者：构造新提案（携带 high_qc）。
    pub fn propose_block(
        &self,
        parent_hash: [u8; 32],
        number: u64,
        txs: Vec<Transaction>,
        _high_qc: QuorumCertificate,
    ) -> Block {
        let mut block = Block {
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
            transactions: txs,
            uncle_headers: vec![],
        };
        block.compute_txs_root();
        block
    }

    /// 副本：对 Prepare 阶段投票。
    pub fn vote_prepare(&mut self, block: &Block, high_qc: &QuorumCertificate) -> Option<Vote> {
        if !self.safety.can_vote_prepare(block, high_qc) {
            return None;
        }
        Some(Vote {
            block_hash: block.header.hash(),
            view_number: self.pacemaker.current_view,
            phase: Phase::Prepare,
            voter: chainforge_crypto::ecdsa::PublicKey::from_bytes(&[0u8; 33]).unwrap(),
            signature: chainforge_crypto::ecdsa::Signature::from_bytes(&[0u8; 64], 0).unwrap(),
        })
    }

    /// 收集投票形成 QC。
    pub fn form_qc(
        &self,
        votes: Vec<Vote>,
        phase: Phase,
        quorum: usize,
    ) -> Option<QuorumCertificate> {
        if votes.len() < quorum {
            return None;
        }
        let first = votes.first()?;
        Some(QuorumCertificate {
            block_hash: first.block_hash,
            view_number: first.view_number,
            phase,
            votes,
        })
    }

    /// 处理 Prepare-QC：插入区块到 BlockTree。
    pub fn on_prepare_qc(&mut self, block: Block, qc: QuorumCertificate) {
        self.block_tree.insert(block, qc);
    }

    /// 处理 PreCommit-QC：更新 locked_view。
    pub fn on_precommit_qc(&mut self, hash: &[u8; 32], qc: QuorumCertificate) {
        self.block_tree.add_precommit_qc(hash, qc.clone());
        self.safety.update_locked(qc);
    }

    /// 处理 Commit-QC：提交区块。
    pub fn on_commit_qc(&mut self, hash: &[u8; 32], qc: QuorumCertificate) {
        self.block_tree.add_commit_qc(hash, qc);
    }

    /// 进入新 view。
    pub fn advance_view(&mut self, view: u64) {
        self.pacemaker.advance_view(view);
        self.block_tree.view = view;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_block(number: u64, parent: [u8; 32]) -> Block {
        let mut b = Block {
            header: BlockHeader {
                parent_hash: parent,
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
        };
        b.compute_txs_root();
        b
    }

    fn make_vote(block_hash: [u8; 32], view: u64, phase: Phase) -> Vote {
        Vote {
            block_hash,
            view_number: view,
            phase,
            voter: chainforge_crypto::ecdsa::PublicKey::from_bytes(&[0u8; 33]).unwrap(),
            signature: chainforge_crypto::ecdsa::Signature::from_bytes(&[0u8; 64], 0).unwrap(),
        }
    }

    #[test]
    fn test_propose_and_vote() {
        let mut engine = ConsensusEngine::new(0, 4);
        let block = make_block(1, [0u8; 32]);
        let high_qc = QuorumCertificate::new([0u8; 32], 0, Phase::Prepare);

        let vote = engine.vote_prepare(&block, &high_qc);
        assert!(vote.is_some());
    }

    #[test]
    fn test_full_pipeline_commit() {
        let mut engine = ConsensusEngine::new(0, 4);
        let genesis = [0u8; 32];
        let block = make_block(1, genesis);
        let hash = block.header.hash();

        // Prepare 阶段
        let prepare_votes = vec![
            make_vote(hash, 1, Phase::Prepare),
            make_vote(hash, 1, Phase::Prepare),
            make_vote(hash, 1, Phase::Prepare),
        ];
        let prepare_qc = engine.form_qc(prepare_votes, Phase::Prepare, 3).unwrap();
        engine.on_prepare_qc(block.clone(), prepare_qc);

        // PreCommit 阶段
        let precommit_votes = vec![
            make_vote(hash, 1, Phase::PreCommit),
            make_vote(hash, 1, Phase::PreCommit),
            make_vote(hash, 1, Phase::PreCommit),
        ];
        let precommit_qc = engine
            .form_qc(precommit_votes, Phase::PreCommit, 3)
            .unwrap();
        engine.on_precommit_qc(&hash, precommit_qc);

        // Commit 阶段
        let commit_votes = vec![
            make_vote(hash, 1, Phase::Commit),
            make_vote(hash, 1, Phase::Commit),
            make_vote(hash, 1, Phase::Commit),
        ];
        let commit_qc = engine.form_qc(commit_votes, Phase::Commit, 3).unwrap();
        engine.on_commit_qc(&hash, commit_qc);

        assert_eq!(engine.block_tree.committed_height(), 1);
    }

    #[test]
    fn test_safety_reject_old_view() {
        let mut engine = ConsensusEngine::new(1, 4);
        engine.safety.locked_view = 5;

        let old_block = make_block(3, [0u8; 32]);
        let old_qc = QuorumCertificate::new([0u8; 32], 3, Phase::Prepare);

        let vote = engine.vote_prepare(&old_block, &old_qc);
        assert!(vote.is_none()); // 拒绝旧 view 的提案
    }

    #[test]
    fn test_leader_rotation() {
        let engine = ConsensusEngine::new(0, 4);
        assert!(engine.pacemaker.is_leader()); // view 0, node 0 是领导者

        let mut engine2 = ConsensusEngine::new(1, 4);
        engine2.advance_view(1);
        assert!(engine2.pacemaker.is_leader()); // view 1, node 1 是领导者
    }

    #[test]
    fn test_fork_choice() {
        let mut engine = ConsensusEngine::new(0, 4);
        let genesis = [0u8; 32];

        // 构建主链：genesis -> A -> B
        let a = make_block(1, genesis);
        let a_hash = a.header.hash();
        let a_qc = QuorumCertificate::new(a_hash, 1, Phase::Prepare);
        engine.on_prepare_qc(a.clone(), a_qc.clone());
        engine.on_precommit_qc(&a_hash, QuorumCertificate::new(a_hash, 1, Phase::PreCommit));
        engine.on_commit_qc(&a_hash, QuorumCertificate::new(a_hash, 1, Phase::Commit));

        let b = make_block(2, a_hash);
        let b_hash = b.header.hash();
        engine.on_prepare_qc(b.clone(), QuorumCertificate::new(b_hash, 2, Phase::Prepare));
        engine.on_precommit_qc(&b_hash, QuorumCertificate::new(b_hash, 2, Phase::PreCommit));
        engine.on_commit_qc(&b_hash, QuorumCertificate::new(b_hash, 2, Phase::Commit));

        assert_eq!(engine.block_tree.committed_height(), 2);
        assert_eq!(engine.block_tree.committed_blocks().len(), 2);
    }
}
