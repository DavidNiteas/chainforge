//! Gossip 消息广播协议。

use std::collections::HashSet;
use std::time::Duration;

use crate::message::Message;
use crate::peer::PeerInfo;

/// Gossip 广播器。
pub struct Gossip {
    /// 已传播的消息 ID 集合（Keccak-256 哈希）。
    seen: HashSet<[u8; 32]>,
    ///  Seen 集合清理间隔。
    _ttl: Duration,
    /// 每次传播选择的邻居数。
    fanout: usize,
}

impl Gossip {
    pub fn new(fanout: usize, ttl: Duration) -> Self {
        Gossip {
            seen: HashSet::new(),
            _ttl: ttl,
            fanout,
        }
    }

    /// 计算消息 ID（对 Message 的 bincode 编码取 Keccak-256）。
    pub fn message_id(msg: &Message) -> [u8; 32] {
        chainforge_crypto::keccak256(&msg.encode())
    }

    /// 判断是否已处理过该消息。
    pub fn is_seen(&self, msg_id: &[u8; 32]) -> bool {
        self.seen.contains(msg_id)
    }

    /// 标记消息为已见。
    pub fn mark_seen(&mut self, msg_id: [u8; 32]) {
        self.seen.insert(msg_id);
    }

    /// 清理过期的 seen 记录（简化实现：直接清空，实际可用 LRU）。
    pub fn prune(&mut self) {
        // 简化：当集合过大时清空
        if self.seen.len() > 100_000 {
            self.seen.clear();
        }
    }

    /// 选择传播目标：从候选 peers 中随机选择 fanout 个。
    pub fn select_targets(&self, candidates: &[PeerInfo]) -> Vec<PeerInfo> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        if candidates.len() <= self.fanout {
            return candidates.to_vec();
        }

        // 简单策略：按 hash 排序后取前 fanout 个（确定性，便于测试）
        let mut indexed: Vec<_> = candidates
            .iter()
            .map(|p| {
                let mut hasher = DefaultHasher::new();
                p.id.0.hash(&mut hasher);
                (hasher.finish(), p.clone())
            })
            .collect();
        indexed.sort_by_key(|(h, _)| *h);
        indexed.into_iter().take(self.fanout).map(|(_, p)| p).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::SocketAddr;

    fn make_peer(id_byte: u8, port: u16) -> PeerInfo {
        let mut id = [0u8; 32];
        id[31] = id_byte;
        PeerInfo {
            id: crate::peer::PeerId(id),
            addr: SocketAddr::from(([127, 0, 0, 1], port)),
        }
    }

    #[test]
    fn test_message_deduplication() {
        let mut gossip = Gossip::new(3, Duration::from_secs(60));
        let msg = Message::Ping;
        let id = Gossip::message_id(&msg);

        assert!(!gossip.is_seen(&id));
        gossip.mark_seen(id);
        assert!(gossip.is_seen(&id));
    }

    #[test]
    fn test_select_targets() {
        let gossip = Gossip::new(3, Duration::from_secs(60));
        let peers: Vec<_> = (0..10).map(|i| make_peer(i, 1000 + i as u16)).collect();

        let targets = gossip.select_targets(&peers);
        assert_eq!(targets.len(), 3);
        // 确保选择是确定性的
        let targets2 = gossip.select_targets(&peers);
        assert_eq!(targets, targets2);
    }

    #[test]
    fn test_select_targets_when_fewer_than_fanout() {
        let gossip = Gossip::new(10, Duration::from_secs(60));
        let peers: Vec<_> = (0..3).map(|i| make_peer(i, 1000 + i as u16)).collect();

        let targets = gossip.select_targets(&peers);
        assert_eq!(targets.len(), 3);
    }
}
