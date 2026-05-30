# RPC-03：区块查询与合约调用

## 目标

实现区块查询和只读合约调用接口。

## 交付物

- `eth_getBlockByNumber` / `eth_getBlockByHash` — 从 Storage 查询区块
- `eth_call` — 接入 EvmExecutor 执行只读调用
- `eth_getCode` — 查询合约字节码

## 验收标准

- [x] `eth_call` 测试通过
- [x] 区块查询返回正确格式
- [x] 全 workspace 测试通过
