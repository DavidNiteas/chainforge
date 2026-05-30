# Phase 03: 密码学哈希原语

## 目标

在 `chainforge-crypto` 中实现基础哈希函数：SHA-256、Keccak-256、RIPEMD-160。本阶段**不暴露给 Python**，仅完成纯 Rust 实现 + 测试向量验证 + property test。

---

## 交付物清单

### 源码

| 文件 | 说明 |
|------|------|
| `crates/chainforge-crypto/src/lib.rs` | 模块导出 |
| `crates/chainforge-crypto/src/hash.rs` | `sha256`, `keccak256`, `ripemd160` 实现 |

### 测试

| 文件 | 说明 |
|------|------|
| `crates/chainforge-crypto/src/hash.rs` (内联 `#[cfg(test)]`) | 已知测试向量 + property test |

---

## 核心代码规格

### 函数签名

```rust
/// SHA-256 哈希
pub fn sha256(data: &[u8]) -> [u8; 32];

/// Keccak-256 哈希（Ethereum 标准）
pub fn keccak256(data: &[u8]) -> [u8; 32];

/// RIPEMD-160 哈希
pub fn ripemd160(data: &[u8]) -> [u8; 20];
```

### 依赖策略

| 算法 | 推荐 crate | 理由 |
|------|-----------|------|
| SHA-256 | `ring` | 经过审计，性能优秀 |
| Keccak-256 | `tiny-keccak` | 轻量，Ethereum 生态标准 |
| RIPEMD-160 | `ripemd` | 纯 Rust，无 unsafe |

### 已知测试向量（必须验证）

**SHA-256 空输入：**
```
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
```

**Keccak-256 空输入（Ethereum）：**
```
c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470
```

---

## 验收标准（必须全部通过）

- [ ] `cargo test -p chainforge-crypto` 全部通过
- [ ] 空输入 SHA-256 等于 NIST 标准值
- [ ] 空输入 Keccak-256 等于 Ethereum 标准值
- [ ] 任意输入的 SHA-256 输出长度恒为 32 字节（proptest 验证）
- [ ] 同一输入两次哈希结果完全一致（proptest 验证）
- [ ] 不同输入哈希碰撞概率在随机测试中未触发（10k 次随机输入）

---

## 预计工时

0.5 ~ 1 天

---

## 前置依赖

Phase 01: 最小可编译工程骨架

---

## 下一步

Phase 04: 数字签名原语
