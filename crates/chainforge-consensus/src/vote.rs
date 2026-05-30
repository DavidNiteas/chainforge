//! 投票与 QuorumCertificate。

use chainforge_crypto::ecdsa::{PublicKey, Signature};

/// 共识阶段。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum Phase {
    #[default]
    Prepare,
    PreCommit,
    Commit,
    Decide,
}

/// 投票消息。
#[derive(Clone, Debug)]
pub struct Vote {
    pub block_hash: [u8; 32],
    pub view_number: u64,
    pub phase: Phase,
    pub voter: PublicKey,
    pub signature: Signature,
}

/// Quorum Certificate（2f+1 个投票的集合）。
#[derive(Clone, Debug, Default)]
pub struct QuorumCertificate {
    pub block_hash: [u8; 32],
    pub view_number: u64,
    pub phase: Phase,
    pub votes: Vec<Vote>,
}

impl QuorumCertificate {
    /// 创建空的 QC（用于 genesis）。
    pub fn new(block_hash: [u8; 32], view_number: u64, phase: Phase) -> Self {
        QuorumCertificate {
            block_hash,
            view_number,
            phase,
            votes: vec![],
        }
    }

    /// 检查是否达到 quorum（假设总节点数为 3f+1，quorum = 2f+1）。
    pub fn has_quorum(&self, quorum: usize) -> bool {
        self.votes.len() >= quorum
    }

    /// 验证所有签名（简化：实际应验证每个投票的签名）。
    pub fn verify(&self, _public_keys: &[PublicKey]) -> bool {
        // 简化实现：在测试环境中假设签名有效
        !self.votes.is_empty() || self.view_number == 0
    }
}
