# P2P 网络层设计

## 目标

为 Kilnchain 构建去中心化点对点网络层，实现节点发现、消息广播、区块与交易同步。作为迭代开发阶段的第一个扩展方向，P2P 层是共识算法和全节点同步的基础设施。

## 技术选型

| 组件 | 技术 | 理由 |
|------|------|------|
| 传输层 | TCP + Noise 协议 | 轻量、可控，避免 libp2p 的复杂依赖 |
| 序列化 | `bincode` + `serde` | 高效二进制编码，兼容现有 Rust 生态 |
| 节点发现 | Kademlia DHT（简化版） | 去中心化节点发现，无需中心化 bootstrap |
| 消息广播 | Gossip 协议 | 高效传播交易和区块，容错性强 |
| 并发模型 | Tokio | 与现有存储层一致，async/await 统一 |

## 模块划分

### `kilnchain-p2p` crate（新增）

```
crates/kilnchain-p2p/
├── Cargo.toml
└── src/
    ├── lib.rs          # 模块导出
    ├── peer.rs         # Peer 连接管理（PeerId、PeerInfo）
    ├── transport.rs    # TCP + Noise 加密传输
    ├── discovery.rs    # Kademlia 简化节点发现
    ├── gossip.rs       # Gossip 消息广播
    ├── message.rs      # 网络消息定义（Message 枚举）
    ├── sync.rs         # 区块/交易同步协议
    └── node.rs         # Node 主结构，整合所有子系统
```

### 核心结构

```rust
pub struct Node {
    local_id: PeerId,
    swarm: Swarm,
    discovery: Discovery,
    gossip: Gossip,
    sync: SyncManager,
}

pub enum Message {
    Transaction(Transaction),
    Block(Block),
    BlockRequest { from: u64, to: u64 },
    BlockResponse(Vec<Block>),
    PeerDiscovery(Vec<PeerInfo>),
    Ping,
    Pong,
}
```

## 消息协议规格

| 消息类型 | 方向 | 说明 |
|----------|------|------|
| `Transaction` | 广播 | 新交易进入 mempool，Gossip 传播 |
| `Block` | 广播 | 新区块生成后广播给所有 peers |
| `BlockRequest` | 点对点 | 同步时请求缺失区块范围 |
| `BlockResponse` | 点对点 | 返回请求的区块列表 |
| `PeerDiscovery` | 广播 | 周期性交换已知节点列表 |
| `Ping/Pong` | 点对点 | 心跳检测，维护活跃连接 |

## 与现有模块的交互

```
Node (p2p)
  ├── 接收 Transaction → 验证 → 提交到 mempool (kilnchain-core)
  ├── 接收 Block → 验证 → 写入存储 (kilnchain-storage)
  ├── 发送 BlockRequest → 从 peers 拉取缺失区块
  └── 本地生成 Block → 广播到所有 peers
```

## 迭代阶段划分

| 阶段 | 目标 | 交付物 |
|------|------|--------|
| P2P-01 | 基础 TCP 连接 + Noise 握手 | PeerId、加密连接、心跳 |
| P2P-02 | 消息编解码 + 基础广播 | Message 枚举、send/receive API |
| P2P-03 | Kademlia 节点发现 | 简化 DHT、Peer 路由表 |
| P2P-04 | Gossip 协议 | 高效广播、消息去重 |
| P2P-05 | 区块/交易同步 | SyncManager、缺失区块拉取 |
| P2P-06 | 集成测试 + 多节点模拟 | 3~5 节点本地网络测试 |

## 验收标准

- [ ] 两个节点能建立加密连接并交换 Ping/Pong
- [ ] 单个 Transaction 能在 3 节点网络中 1 秒内传播到全部节点
- [ ] 区块同步：落后 100 个区块的节点能在 10 秒内追平
- [ ] 节点掉线后重连能自动恢复同步
