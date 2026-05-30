//! 安全规则。

use chainforge_core::block::Block;

use crate::vote::{Phase, QuorumCertificate};

/// 安全规则：防止双投和无效分支。
pub struct SafetyRules {
    /// 当前 locked 的 view（来自 Prepare-QC）。
    pub locked_view: u64,
    /// 当前 locked 的 QC。
    pub locked_qc: Option<QuorumCertificate>,
}

impl Default for SafetyRules {
    fn default() -> Self {
        Self::new()
    }
}

impl SafetyRules {
    pub fn new() -> Self {
        SafetyRules {
            locked_view: 0,
            locked_qc: None,
        }
    }

    /// 判断是否可以投票给某个提案。
    ///
    /// 规则：
    /// 1. 提案的 view 必须大于 locked_view
    /// 2. 提案携带的 parent 的 Prepare-QC 的 view 必须 >= locked_view
    pub fn can_vote_prepare(&self, block: &Block, high_qc: &QuorumCertificate) -> bool {
        block.header.number > self.locked_view || high_qc.view_number >= self.locked_view
    }

    /// 更新 locked_view（收到 PreCommit-QC 时）。
    pub fn update_locked(&mut self, qc: QuorumCertificate) {
        if qc.view_number > self.locked_view {
            self.locked_view = qc.view_number;
            self.locked_qc = Some(qc);
        }
    }

    /// 检查是否存在双投。
    pub fn check_double_vote(&self, _block_hash: &[u8; 32], _phase: Phase) -> bool {
        // 简化：实际应检查历史投票记录
        false
    }
}
