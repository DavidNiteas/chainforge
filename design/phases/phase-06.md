# Phase 06: 交易与区块核心结构

## 目标

实现区块链最核心的数据结构：`Transaction`、`BlockHeader`、`Block`，以及 RLP 编解码和关键业务方法（哈希计算、发送方地址恢复）。

---

## 交付物清单

### 源码

| 文件 | 说明 |
|------|------|
| `crates/chainforge-core/src/lib.rs` | 导出 `tx`, `block` 模块 |
| `crates/chainforge-core/src/tx.rs` | `Transaction` 结构及方法 |
| `crates/chainforge-core/src/block.rs` | `BlockHeader`, `Block` 结构及方法 |
| `crates/chainforge-core/src/rlp.rs` | RLP 编码器/解码器（基础实现） |

### 测试

| 文件 | 说明 |
|------|------|
| `crates/chainforge-core/src/tx.rs` (内联测试) | Transaction 往返、哈希、恢复发送方 |
| `crates/chainforge-core/src/block.rs` (内联测试) | Block 往返、header 哈希 |
| `crates/chainforge-core/src/rlp.rs` (内联测试) | RLP 基础类型编解码 |

---

## 核心代码规格

### Transaction

```rust
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Transaction {
    pub nonce: u64,
    pub gas_price: u128,
    pub gas_limit: u64,
    pub to: Option<[u8; 20]>,   // None = 合约创建
    pub value: u128,
    pub data: Vec<u8>,
    pub v: u64,                  // 恢复 ID + 链 ID
    pub r: [u8; 32],
    pub s: [u8; 32],
}
```

### Transaction 方法

| 方法 | 签名 | 说明 |
|------|------|------|
| `hash` | `fn hash(&self) -> [u8; 32]` | RLP 编码后 Keccak-256 |
| `recover_sender` | `fn recover_sender(&self) -> Result<[u8; 20], ChainforgeError>` | ECDSA 公钥恢复 → 取后 20 字节 |
| `encode_rlp` | `fn encode_rlp(&self) -> Vec<u8>` | 完整 RLP 编码 |
| `decode_rlp` | `fn decode_rlp(data: &[u8]) -> Result<Self, ChainforgeError>` | RLP 解码 |

### BlockHeader

```rust
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BlockHeader {
    pub parent_hash: [u8; 32],
    pub number: u64,
    pub timestamp: u64,
    pub difficulty: u64,
    pub nonce: u64,
    pub extra_data: Vec<u8>,    // 最大 32 字节
    pub state_root: [u8; 32],
    pub txs_root: [u8; 32],
}
```

### Block

```rust
#[derive(Clone, Debug, PartialEq)]
pub struct Block {
    pub header: BlockHeader,
    pub transactions: Vec<Transaction>,
    pub uncle_headers: Vec<BlockHeader>,
}
```

### RLP 编解码范围

本阶段只需支持以下类型的 RLP：
- `u64` / `u128` → 大端字节串
- `Vec<u8>` / `&[u8]` → 字节串
- `Option<[u8; 20]>` → 有值时编码为字节串，None 编码为空串
- 列表（嵌套结构）→ 递归编码

不需要实现流式解码，先以完整 `Vec<u8>` 为基础。

---

## 验收标准（必须全部通过）

- [ ] `cargo test -p chainforge-core` 全部通过
- [ ] Transaction RLP 往返：`decode(encode(tx)) == tx`
- [ ] `hash()` 返回 32 字节
- [ ] `recover_sender` 对有效签名返回正确的 20 字节地址（使用已知私钥构造交易验证）
- [ ] `extra_data` 超过 32 字节时构造/编码返回 `InvalidParameter`
- [ ] BlockHeader RLP 往返一致
- [ ] Block 的 `txs_root` 等于其交易列表构建的 Merkle 树根

---

## 预计工时

2 ~ 3 天

---

## 前置依赖

Phase 02: 跨语言错误体系
Phase 03: 密码学哈希原语（Keccak-256）
Phase 04: 数字签名原语（公钥恢复）
Phase 05: Merkle 树（txs_root 计算）

---

## 下一步

Phase 07: 存储层 Trait + 内存后端
