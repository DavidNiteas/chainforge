# Phase 05: Merkle 树与属性测试

## 目标

在 `kilnchain-core` 中实现二叉 SHA-256 Merkle Tree，支持根哈希计算、证明生成与验证。以 property test（proptest）为主要验证手段，确保算法正确性。

---

## 交付物清单

### 源码

| 文件 | 说明 |
|------|------|
| `crates/kilnchain-core/src/lib.rs` | 导出 `merkle` 模块 |
| `crates/kilnchain-core/src/merkle.rs` | `MerkleTree`, `MerkleProof` |

### 测试

| 文件 | 说明 |
|------|------|
| `crates/kilnchain-core/src/merkle.rs` (内联 `#[cfg(test)]`) | 单元测试 + proptest |

---

## 核心代码规格

### 结构定义

```rust
pub struct MerkleTree {
    leaves: Vec<[u8; 32]>,
    layers: Vec<Vec<[u8; 32]>>,
}

pub struct MerkleProof {
    pub siblings: Vec<[u8; 32]>,
    pub indices: Vec<bool>, // true = 当前节点是右子节点，false = 左子节点
}
```

### 方法清单

| 方法 | 签名 | 说明 |
|------|------|------|
| `new` | `fn new(leaves: Vec<[u8; 32]>) -> Self` | 构建树，奇数时复制最后一个叶子 |
| `root` | `fn root(&self) -> [u8; 32]` | 返回根哈希；空树返回 `EMPTY_ROOT` |
| `proof` | `fn proof(&self, index: usize) -> Option<MerkleProof>` | 为指定索引的叶子生成证明 |
| `verify` | `fn verify(root: &[u8; 32], leaf: &[u8; 32], proof: &MerkleProof) -> bool` | 静态方法，验证证明 |

### 哈希规则

- 叶子节点：`leaf_hash = SHA-256(leaf_data)`（输入已预哈希为 `[u8; 32]`，不再二次哈希）
- 内部节点：`node_hash = SHA-256(left || right)`（拼接 64 字节后哈希）
- 奇数处理：复制最后一个叶子使其变为偶数

### 空树根常量

```rust
pub const EMPTY_ROOT: [u8; 32] = [0u8; 32];
```

（注：若需兼容 Ethereum，应使用 Ethereum 的空 trie root，此处先用 0 数组占位。）

---

## 验收标准（必须全部通过）

- [ ] `cargo test -p kilnchain-core` 全部通过（含 proptest）
- [ ] `test_empty_merkle_root`：空树返回 `EMPTY_ROOT`
- [ ] `test_single_leaf`：单叶子树根等于该叶子值
- [ ] `test_proof_roundtrip`：对 100 个叶子的树，随机选 10 个索引生成证明并验证通过
- [ ] `test_tampered_proof_fails`：篡改 proof 中的任意 sibling，验证返回 false
- [ ] proptest `merkle_root_deterministic`：相同输入永远产生相同根
- [ ] proptest `merkle_proof_verifies`：随机 1~1000 个叶子 + 随机索引，证明验证通过
- [ ] proptest `tampered_leaf_fails`：随机叶子被篡改后，原证明验证失败

---

## 预计工时

1 ~ 2 天

---

## 前置依赖

Phase 01: 最小可编译工程骨架
Phase 03: 密码学哈希原语（使用 `sha256`）

---

## 下一步

Phase 06: 交易与区块核心结构
