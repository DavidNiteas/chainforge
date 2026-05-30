# RPC-04：WebSocket 订阅

## 目标

实现 WebSocket JSON-RPC 订阅接口。

## 交付物

- `/ws` 路由 — axum WebSocket upgrade
- `eth_subscribe` — 支持 `newHeads`、`newPendingTransactions`
- `eth_unsubscribe` — 取消订阅
- EventBus — broadcast channel 事件广播

## 验收标准

- [x] WebSocket handler 编译通过
- [x] 订阅/取消订阅逻辑测试通过
- [x] 全 workspace 测试通过
