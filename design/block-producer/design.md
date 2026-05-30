# 区块生产者（Block Producer）设计

## 目标

实现从 mempool 收集交易、构建区块、计算状态变更、并将区块提交给共识层的完整流程。在 PoA（权威证明）模式下由固定授权节点出块，在 BFT 模式下由共识选出的领导者出块。

## 核心流程

```
1. 从 mempool 按 gas_price 优先级取出交易（不超过 gas_limit 上限）
2. 按顺序执行每笔交易（EVM 执行或简单转账）
3. 收集状态变更、计算 receipts
4. 构建 BlockHeader（parent_hash、number、timestamp、txs_root、state_root、receipts_root）
5. 打包为 Block
6. 提交给共识层（或直接在 PoA 模式下广播）
```

## 迭代阶段

| 阶段 | 目标 |
|------|------|
| BP-01 | 基础区块构建（从固定交易列表）|
| BP-02 | 与 mempool 集成 |
| BP-03 | Gas 限制与交易选择优化 |
| BP-04 | PoA 出块定时器 |
| BP-05 | 与共识层集成（BFT 领导者出块）|
