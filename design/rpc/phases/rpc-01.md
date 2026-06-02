# RPC-01：HTTP 服务器骨架

## 目标

创建 `kilnchain-rpc` crate，搭建 axum HTTP 服务器骨架，实现基础 JSON-RPC 路由。

## 交付物

- `crates/kilnchain-rpc/` crate
- `src/types.rs` — RpcRequest / RpcResponse / RpcError 类型
- `src/server.rs` — axum Router + rpc_handler
- 支持方法：`eth_chainId`、`net_version`、`eth_blockNumber`

## 验收标准

- [x] `cargo test -p kilnchain-rpc` 通过
- [x] `cargo clippy -p kilnchain-rpc -- -D warnings` 零警告
