# Phase 04: 数字签名原语

## 目标

实现 Secp256k1 ECDSA 的完整功能：密钥生成、签名、验签、公钥恢复。Ed25519 仅预留接口，不实现。本阶段仍**不暴露给 Python**。

---

## 交付物清单

### 源码

| 文件 | 说明 |
|------|------|
| `crates/chainforge-crypto/src/lib.rs` | 导出 `ecdsa` 模块 |
| `crates/chainforge-crypto/src/ecdsa.rs` | `SecretKey`, `PublicKey`, `Signature` |

### 测试

| 文件 | 说明 |
|------|------|
| `crates/chainforge-crypto/src/ecdsa.rs` (内联 `#[cfg(test)]`) | 往返测试、错误消息测试 |
| `crates/chainforge-crypto/benches/sign_bench.rs` | Criterion 签名/验签吞吐量基准 |

---

## 核心代码规格

### 结构定义

```rust
pub struct SecretKey([u8; 32]);
pub struct PublicKey([u8; 33]);  // 压缩格式
pub struct Signature([u8; 64]);
```

### 方法清单

| 结构 | 方法 | 说明 |
|------|------|------|
| `SecretKey` | `random() -> Self` | 密码学安全随机生成 |
| `SecretKey` | `public_key(&self) -> PublicKey` | 派生公钥 |
| `SecretKey` | `sign(&self, msg: &[u8]) -> Result<Signature, ChainforgeError>` | ECDSA 签名 |
| `PublicKey` | `verify(&self, msg: &[u8], sig: &Signature) -> Result<bool, ChainforgeError>` | 验签 |
| `PublicKey` | `recover_from_msg(msg: &[u8], sig: &Signature) -> Result<Self, ChainforgeError>` | 从签名恢复公钥 |
| `Signature` | `to_bytes(&self) -> [u8; 64]` | 序列化 |
| `Signature` | `from_bytes(bytes: &[u8]) -> Result<Self, ChainforgeError>` | 反序列化（严格长度检查） |

### 依赖

- `secp256k1` crate (rust-bitcoin 维护版本)
- feature flag: `recovery`（用于公钥恢复）

---

## 验收标准（必须全部通过）

- [ ] `cargo test -p chainforge-crypto` 全部通过
- [ ] 签名 → 验签往返：`verify(msg, sign(msg)) == true`
- [ ] 错误消息拒绝：`verify(b"wrong", sign(msg)) == false`
- [ ] 公钥恢复：`recover(sign(msg)) == public_key()`
- [ ] 无效私钥长度（非 32 字节）返回 `ChainforgeError::Crypto`
- [ ] Criterion bench 运行成功，输出签名吞吐量基线（保存 `target/criterion/` 报告）

---

## 预计工时

1 ~ 2 天

---

## 前置依赖

Phase 02: 跨语言错误体系（使用 `ChainforgeError`）
Phase 03: 密码学哈希原语（签名内部可能使用哈希）

---

## 下一步

Phase 05: Merkle 树与属性测试
