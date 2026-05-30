# Mempool（交易池）设计

## 目标

实现高效的交易内存池，管理待确认交易，为区块生产者和共识层提供交易来源。

## 核心功能

- 交易去重（按 hash）
- nonce 连续性验证
- Gas price 优先级排序
- 账户级别 nonce 跟踪
- 容量限制与淘汰策略

## 核心结构

```rust
pub struct Mempool {
    /// 所有待处理交易（按 hash 索引）
    txs: HashMap<[u8; 32], Transaction>,
    /// 按账户 + nonce 排序的队列
    queues: BTreeMap<[u8; 20], BTreeMap<u64, [u8; 32]>>,
    /// 按 gas_price 排序的全局优先级队列
    priority: BTreeMap<u128, Vec<[u8; 32]>>,
    /// 容量上限
    max_size: usize,
}
```

## 迭代阶段

| 阶段 | 目标 |
|------|------|
| MEM-01 | 基础插入/查询/删除 |
| MEM-02 | Gas price 优先级队列 |
| MEM-03 | 账户 nonce 连续性验证 |
| MEM-04 | 容量限制与低优先级淘汰 |
| MEM-05 | 与 P2P 层集成（接收广播交易）|
