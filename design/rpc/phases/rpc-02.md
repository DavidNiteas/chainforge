# RPC-02：交易提交与查询

## 目标

实现 `eth_sendRawTransaction` 和账户相关查询接口。

## 交付物

- `eth_sendRawTransaction` — RLP 解码交易 → 验证 nonce → 入 mempool
- `eth_getBalance` — 返回账户余额（stub / 实装）
- `eth_getTransactionCount` — 返回账户 nonce

## 验收标准

- [x] 单元测试覆盖发送交易和查询流程
- [x] 全 workspace 测试通过
