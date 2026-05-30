//! Pacemaker：视图超时与领导者轮换。

/// 领导者选择器。
pub struct LeaderRotator {
    pub node_count: usize,
}

impl LeaderRotator {
    pub fn new(node_count: usize) -> Self {
        LeaderRotator { node_count }
    }

    /// 根据 view_number 轮询选择领导者。
    pub fn leader_for(&self, view: u64) -> usize {
        (view as usize) % self.node_count
    }
}

/// Pacemaker 状态。
pub struct Pacemaker {
    pub current_view: u64,
    pub node_id: usize,
    pub leader_rotator: LeaderRotator,
    pub timeout_ms: u64,
}

impl Pacemaker {
    pub fn new(node_id: usize, node_count: usize, timeout_ms: u64) -> Self {
        Pacemaker {
            current_view: 0,
            node_id,
            leader_rotator: LeaderRotator::new(node_count),
            timeout_ms,
        }
    }

    /// 当前节点是否是领导者。
    pub fn is_leader(&self) -> bool {
        self.leader_rotator.leader_for(self.current_view) == self.node_id
    }

    /// 进入下一个 view。
    pub fn advance_view(&mut self, view: u64) {
        if view > self.current_view {
            self.current_view = view;
        }
    }

    /// 获取当前 view 的领导者 ID。
    pub fn current_leader(&self) -> usize {
        self.leader_rotator.leader_for(self.current_view)
    }
}
