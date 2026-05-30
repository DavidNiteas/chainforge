//! P2P 节点主结构，整合所有子系统。

use std::sync::Arc;
use tokio::sync::RwLock;

use chainforge_mempool::Mempool;

use crate::discovery::RoutingTable;
use crate::gossip::Gossip;
use crate::message::Message;
use crate::peer::{PeerId, PeerInfo};
use crate::sync::SyncManager;

/// P2P 节点配置。
#[derive(Clone, Debug)]
pub struct NodeConfig {
    pub local_id: PeerId,
    pub static_key: [u8; 32],
    pub gossip_fanout: usize,
    pub gossip_ttl_secs: u64,
}

impl NodeConfig {
    pub fn new(static_key: [u8; 32]) -> Self {
        let local_id = PeerId::from_public_key(&static_key);
        NodeConfig {
            local_id,
            static_key,
            gossip_fanout: 3,
            gossip_ttl_secs: 60,
        }
    }
}

/// P2P 节点。
pub struct Node {
    pub config: NodeConfig,
    pub routing_table: Arc<RwLock<RoutingTable>>,
    pub gossip: Arc<RwLock<Gossip>>,
    pub sync: Arc<RwLock<SyncManager>>,
    pub mempool: Arc<RwLock<Mempool>>,
}

impl Node {
    pub fn new(config: NodeConfig) -> Self {
        Node {
            routing_table: Arc::new(RwLock::new(RoutingTable::new(config.local_id))),
            gossip: Arc::new(RwLock::new(Gossip::new(
                config.gossip_fanout,
                std::time::Duration::from_secs(config.gossip_ttl_secs),
            ))),
            sync: Arc::new(RwLock::new(SyncManager::new())),
            mempool: Arc::new(RwLock::new(Mempool::new())),
            config,
        }
    }

    /// 处理收到的消息，返回需要转发的消息列表。
    pub async fn handle_message(&self, msg: &Message) -> Vec<Message> {
        let mut to_forward = Vec::new();

        match msg {
            Message::Ping => {
                to_forward.push(Message::Pong);
            }
            Message::Pong => {
                // 心跳响应，无需转发
            }
            Message::Transaction(tx_bytes) => {
                let msg_id = Gossip::message_id(msg);
                {
                    let mut gossip = self.gossip.write().await;
                    if !gossip.is_seen(&msg_id) {
                        gossip.mark_seen(msg_id);
                        to_forward.push(msg.clone());
                    }
                }
                // 尝试解码并入池
                if let Ok(tx) = chainforge_core::tx::Transaction::decode_rlp(tx_bytes) {
                    let mut mempool = self.mempool.write().await;
                    if mempool.is_nonce_valid(&tx) {
                        mempool.insert(tx);
                    }
                }
            }
            Message::Block(_) => {
                let msg_id = Gossip::message_id(msg);
                {
                    let mut gossip = self.gossip.write().await;
                    if !gossip.is_seen(&msg_id) {
                        gossip.mark_seen(msg_id);
                        to_forward.push(msg.clone());
                    }
                }
            }
            Message::BlockRequest { from, to } => {
                // 简化：本地不存储历史区块，直接忽略
                let _ = (*from, *to);
            }
            Message::BlockResponse(_blocks) => {
                // Block 解码由上层 SyncManager 处理
                // 简化：P2P-06 中仅做消息转发
            }
            Message::PeerDiscovery(peers) | Message::FindNodeResponse(peers) => {
                let mut rt = self.routing_table.write().await;
                for peer in peers {
                    rt.update(peer.clone());
                }
            }
            Message::FindNode { target } => {
                let rt = self.routing_table.read().await;
                let closest = rt.find_closest(target, 20);
                to_forward.push(Message::FindNodeResponse(closest));
            }
            Message::GetBlockHeaders { .. }
            | Message::BlockHeaders(_)
            | Message::GetProof { .. }
            | Message::Proof(_) => {
                // 轻客户端消息：暂不做特殊处理，直接忽略
            }
        }

        to_forward
    }

    /// 获取需要传播该消息的邻居列表。
    pub async fn gossip_targets(&self, _msg: &Message) -> Vec<PeerInfo> {
        let rt = self.routing_table.read().await;
        let all: Vec<_> = rt
            .find_closest(&self.config.local_id, 100)
            .into_iter()
            .filter(|p| p.id != self.config.local_id)
            .collect();

        let gossip = self.gossip.read().await;
        gossip.select_targets(&all)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::SocketAddr;

    fn make_config(id_byte: u8) -> NodeConfig {
        let mut key = [0u8; 32];
        key[31] = id_byte;
        NodeConfig::new(key)
    }

    fn make_peer(id_byte: u8, port: u16) -> PeerInfo {
        let mut id = [0u8; 32];
        id[31] = id_byte;
        PeerInfo {
            id: PeerId(id),
            addr: SocketAddr::from(([127, 0, 0, 1], port)),
        }
    }

    #[tokio::test]
    async fn test_node_handle_ping() {
        let node = Node::new(make_config(1));
        let response = node.handle_message(&Message::Ping).await;
        assert_eq!(response, vec![Message::Pong]);
    }

    #[tokio::test]
    async fn test_node_gossip_dedup() {
        let node = Node::new(make_config(1));

        // 预先填充路由表
        {
            let mut rt = node.routing_table.write().await;
            for i in 2..10 {
                rt.update(make_peer(i, 1000 + i as u16));
            }
        }

        let msg = Message::Transaction(vec![1, 2, 3]);

        // 第一次收到，应该转发
        let forward1 = node.handle_message(&msg).await;
        assert_eq!(forward1.len(), 1);

        // 第二次收到相同消息，应该去重
        let forward2 = node.handle_message(&msg).await;
        assert!(forward2.is_empty());
    }

    #[tokio::test]
    async fn test_node_peer_discovery() {
        let node = Node::new(make_config(1));
        let peers = vec![make_peer(2, 2000), make_peer(3, 3000)];

        node.handle_message(&Message::PeerDiscovery(peers)).await;

        let rt = node.routing_table.read().await;
        assert_eq!(rt.len(), 2);
    }
}
