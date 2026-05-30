//! 简化 Kademlia DHT 节点发现。

use std::collections::VecDeque;

use crate::peer::{PeerId, PeerInfo};

/// Kademlia 桶容量。
const K: usize = 20;

/// 计算两个 PeerId 的 XOR 距离。
pub fn xor_distance(a: &PeerId, b: &PeerId) -> [u8; 32] {
    let mut dist = [0u8; 32];
    for (i, d) in dist.iter_mut().enumerate() {
        *d = a.0[i] ^ b.0[i];
    }
    dist
}

/// 获取距离的最高位索引（0~255），用于确定 bucket 位置。
pub fn distance_bucket_index(dist: &[u8; 32]) -> usize {
    for i in (0..32).rev() {
        if dist[i] != 0 {
            return (31 - i) * 8 + (7 - dist[i].leading_zeros() as usize);
        }
    }
    0 // 相同 ID（理论上不会发生）
}

/// K-bucket：存储最多 K 个 peers，按最近访问时间排序。
pub struct KBucket {
    peers: VecDeque<PeerInfo>,
}

impl Default for KBucket {
    fn default() -> Self {
        Self::new()
    }
}

impl KBucket {
    pub fn new() -> Self {
        KBucket {
            peers: VecDeque::with_capacity(K),
        }
    }

    /// 添加或更新 peer 位置（移到队尾表示最新）。
    pub fn update(&mut self, info: PeerInfo) {
        if let Some(pos) = self.peers.iter().position(|p| p.id == info.id) {
            self.peers.remove(pos);
        }
        if self.peers.len() >= K {
            self.peers.pop_front(); // 淘汰最旧的
        }
        self.peers.push_back(info);
    }

    /// 查找指定 peer 是否存在。
    pub fn contains(&self, id: &PeerId) -> bool {
        self.peers.iter().any(|p| &p.id == id)
    }

    /// 获取 bucket 中所有 peers。
    pub fn peers(&self) -> &VecDeque<PeerInfo> {
        &self.peers
    }

    /// 移除指定 peer。
    pub fn remove(&mut self, id: &PeerId) {
        if let Some(pos) = self.peers.iter().position(|p| &p.id == id) {
            self.peers.remove(pos);
        }
    }
}

/// Kademlia 路由表：256 个 K-bucket。
pub struct RoutingTable {
    local_id: PeerId,
    buckets: Vec<KBucket>,
}

impl RoutingTable {
    pub fn new(local_id: PeerId) -> Self {
        let mut buckets = Vec::with_capacity(256);
        for _ in 0..256 {
            buckets.push(KBucket::new());
        }
        RoutingTable { local_id, buckets }
    }

    /// 根据目标 ID 的 XOR 距离确定 bucket 索引。
    fn bucket_index(&self, target: &PeerId) -> usize {
        let dist = xor_distance(&self.local_id, target);
        distance_bucket_index(&dist)
    }

    /// 添加/更新 peer。
    pub fn update(&mut self, info: PeerInfo) {
        let idx = self.bucket_index(&info.id);
        self.buckets[idx].update(info);
    }

    /// 查找距离目标最近的 K 个 peers。
    pub fn find_closest(&self, target: &PeerId, k: usize) -> Vec<PeerInfo> {
        let mut all: Vec<_> = self
            .buckets
            .iter()
            .flat_map(|b| b.peers().iter().cloned())
            .collect();

        all.sort_by_key(|p| xor_distance(&p.id, target));
        all.truncate(k);
        all
    }

    /// 返回路由表中所有 peers 的数量。
    pub fn len(&self) -> usize {
        self.buckets.iter().map(|b| b.peers().len()).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
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
            id: PeerId(id),
            addr: SocketAddr::from(([127, 0, 0, 1], port)),
        }
    }

    #[test]
    fn test_xor_distance() {
        let a = PeerId([0u8; 32]);
        let mut b = [0u8; 32];
        b[31] = 0x0F;
        let b = PeerId(b);
        let dist = xor_distance(&a, &b);
        assert_eq!(dist[31], 0x0F);
        assert!(dist[..31].iter().all(|&x| x == 0));
    }

    #[test]
    fn test_distance_bucket_index() {
        let mut dist = [0u8; 32];
        dist[31] = 0x01;
        assert_eq!(distance_bucket_index(&dist), 0);

        dist[31] = 0x80;
        assert_eq!(distance_bucket_index(&dist), 7);

        dist[31] = 0;
        dist[30] = 0x01;
        assert_eq!(distance_bucket_index(&dist), 8);

        // 最高位 byte 测试
        dist[30] = 0;
        dist[0] = 0x80;
        assert_eq!(distance_bucket_index(&dist), 255);
    }

    #[test]
    fn test_routing_table_find_closest() {
        let local = PeerId([0u8; 32]);
        let mut table = RoutingTable::new(local);

        for i in 1..=10 {
            table.update(make_peer(i, 1000 + i as u16));
        }

        let mut target = [0u8; 32];
        target[31] = 5;
        let closest = table.find_closest(&PeerId(target), 3);

        assert_eq!(closest.len(), 3);
        // 最接近 5 的应该是 4, 5, 6（如果存在）或 3, 4, 6 等
        assert!(closest.iter().any(|p| p.id.0[31] == 4));
        assert!(closest.iter().any(|p| p.id.0[31] == 5 || p.id.0[31] == 6));
    }

    #[test]
    fn test_kbucket_capacity() {
        let mut bucket = KBucket::new();
        for i in 0..25 {
            bucket.update(make_peer(i, 1000 + i as u16));
        }
        assert_eq!(bucket.peers().len(), K); // 最多 20 个
    }
}
