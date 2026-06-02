# JSON-RPC API 层设计

## 目标

为 Kilnchain 提供以太坊兼容的 JSON-RPC 接口，使现有工具（MetaMask、Hardhat、Ethers.js）能够直接连接和交互。

## 技术选型

| 组件 | 技术 | 理由 |
|------|------|------|
| HTTP 服务器 | `axum` | Tokio 生态，性能优秀，中间件丰富 |
| JSON 序列化 | `serde_json` | 已依赖，生态标准 |
| 接口标准 | Ethereum JSON-RPC | 兼容 eth_sendTransaction、eth_getBalance 等 |

## 核心方法

| 方法 | 说明 |
|------|------|
| `eth_sendRawTransaction` | 提交已签名交易 |
| `eth_getBalance` | 查询账户余额 |
| `eth_getTransactionCount` | 查询账户 nonce |
| `eth_getBlockByNumber` | 按高度查询区块 |
| `eth_getBlockByHash` | 按哈希查询区块 |
| `eth_call` | 只读合约调用 |
| `eth_estimateGas` | Gas 估算 |
| `eth_getCode` | 查询合约字节码 |
| `eth_chainId` | 返回链 ID |
| `net_version` | 网络版本 |

## 迭代阶段

| 阶段 | 目标 |
|------|------|
| RPC-01 | HTTP 服务器骨架 + 基础路由 |
| RPC-02 | `eth_sendRawTransaction` + `eth_getTransactionReceipt` |
| RPC-03 | 区块查询接口 + `eth_call` |
| RPC-04 | 订阅接口（WebSocket）newHeads、newPendingTransactions |
