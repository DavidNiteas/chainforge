//! P2P 网络消息定义。

use chainforge_core::block::BlockHeader;
use serde::{Deserialize, Serialize};

use crate::peer::PeerInfo;

/// 轻客户端 MPT 证明响应。
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ProofResponse {
    pub key: Vec<u8>,
    pub value: Option<Vec<u8>>,
    pub proof_nodes: Vec<Vec<u8>>,
}

/// 网络消息枚举。
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum Message {
    /// 心跳请求。
    Ping,
    /// 心跳响应。
    Pong,
    /// 广播新交易。
    Transaction(Vec<u8>),
    /// 广播新区块。
    Block(Vec<u8>),
    /// 请求区块范围。
    BlockRequest { from: u64, to: u64 },
    /// 返回区块列表。
    BlockResponse(Vec<Vec<u8>>),
    /// 节点发现：交换已知节点列表。
    PeerDiscovery(Vec<PeerInfo>),
    /// Kademlia 查找节点请求。
    FindNode { target: crate::peer::PeerId },
    /// Kademlia 查找节点响应。
    FindNodeResponse(Vec<PeerInfo>),
    /// 轻客户端：请求区块头。
    GetBlockHeaders { start: u64, count: u64 },
    /// 轻客户端：返回区块头列表。
    BlockHeaders(Vec<BlockHeader>),
    /// 轻客户端：请求 MPT 证明。
    GetProof { key: Vec<u8> },
    /// 轻客户端：返回 MPT 证明。
    Proof(ProofResponse),
}

impl Message {
    /// 将消息编码为字节串（bincode）。
    pub fn encode(&self) -> Vec<u8> {
        bincode::serialize(self).expect("bincode serialization should not fail")
    }

    /// 从字节串解码消息。
    pub fn decode(bytes: &[u8]) -> Result<Self, bincode::Error> {
        bincode::deserialize(bytes)
    }
}
