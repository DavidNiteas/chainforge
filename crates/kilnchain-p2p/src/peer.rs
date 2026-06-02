//! Peer 标识与信息。

use kilnchain_crypto::keccak256;
use serde::{Deserialize, Serialize};

/// 基于公钥哈希的节点唯一标识。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PeerId(pub [u8; 32]);

impl PeerId {
    /// 从公钥字节派生 PeerId（Keccak-256 哈希）。
    pub fn from_public_key(pk: &[u8]) -> Self {
        PeerId(keccak256(pk))
    }
}

/// 节点连接信息。
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PeerInfo {
    pub id: PeerId,
    pub addr: std::net::SocketAddr,
}
