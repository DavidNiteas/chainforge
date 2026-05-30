# Chainforge 用户指南

> [English Version](./README.en.md)

## 简介

**Chainforge** 是一个高性能区块链核心库，使用 **Rust** 实现底层密码学、共识原语与数据结构，并通过 **PyO3** 向 Python 提供绑定。目标是让数据科学家、量化研究员及需要快速原型验证的区块链开发者，能够用熟悉的 Python 接口调用高性能 Rust 底层。

## 核心特性

| 模块 | 功能 | 状态 |
|------|------|------|
| **密码学** | SHA-256、Keccak-256、RIPEMD-160、secp256k1 ECDSA | ✅ |
| **核心结构** | Transaction、BlockHeader、Block、MerkleTree、RLP 编解码 | ✅ |
| **存储** | 内存存储、RocksDB 后端、LRU 缓存 | ✅ |
| **P2P 网络** | Noise XX 加密握手、Kademlia 路由、Gossip 广播、区块同步 | ✅ |
| **交易池** | CRUD、Gas 优先级队列、Nonce 防重放、容量驱逐 | ✅ |
| **共识** | HotStuff BFT — BlockTree、QuorumCertificate、Safety/Liveness | ✅ |
| **EVM** | revm 集成 — 转账、合约部署、合约调用 | ✅ |
| **RPC** | 以太坊兼容 JSON-RPC（HTTP + WebSocket 订阅）| ✅ |
| **轻客户端** | 区块头同步、MerkleProof 验证、MPT 状态证明 | ✅ |
| **Python 绑定** | PyO3 暴露核心类型 + Pydantic 输入校验 | ✅ |

## 快速开始

### 环境要求

- Rust 1.70+（stable）
- Python 3.10+
- [Pixi](https://pixi.sh)（推荐）或 pip + virtualenv

### 安装

```bash
# 克隆仓库
git clone https://github.com/your-org/chainforge.git
cd chainforge

# 使用 Pixi 安装依赖（同时安装 Python + Rust 工具链）
pixi install

# 构建并安装 Python 可编辑包
pixi run dev-build
```

### 运行测试

```bash
# 全部测试（Rust + Python）
pixi run test

# 仅 Rust 测试
cargo test --workspace

# 仅 Python 测试
pixi run test-py

# 类型检查
pixi run typecheck
```

## 使用示例

### Python API

```python
from chainforge import Transaction, BlockHeader, MerkleTree, SecretKey

# 创建交易
tx = Transaction(nonce=0, gas_price=10, gas_limit=21000,
                 to=b'\xab' * 20, value=1000, data=b'')
print("tx hash:", tx.hash().hex())

# Merkle 树
leaves = [bytes([i] * 32) for i in range(4)]
tree = MerkleTree(leaves)
print("merkle root:", tree.root().hex())

# ECDSA 签名
sk = SecretKey.random()
pk = sk.public_key()
msg = b"hello chainforge"
sig = sk.sign(msg)
assert pk.verify(msg, sig)
```

### Rust API

```rust
use chainforge_core::{Transaction, BlockHeader, MerkleTree};
use chainforge_crypto::ecdsa::SecretKey;

fn main() {
    let tx = Transaction {
        nonce: 0, gas_price: 10, gas_limit: 21000,
        to: Some([0xabu8; 20]), value: 1000,
        data: vec![], v: 27, r: [0u8; 32], s: [0u8; 32],
    };
    println!("tx hash: {:x?}", tx.hash());

    let leaves: Vec<[u8; 32]> = (0..4).map(|i| [i as u8; 32]).collect();
    let tree = MerkleTree::new(leaves);
    println!("root: {:x?}", tree.root());

    let sk = SecretKey::random();
    let sig = sk.sign(b"hello").unwrap();
    assert!(sk.public_key().verify(b"hello", &sig));
}
```

## 架构概览

```
┌─────────────────────────────────────────────────────────────┐
│  Python Layer (src/chainforge)                              │
│  ├─ types.py        Pydantic 输入校验                        │
│  ├─ client.py       高层 API（open_db 上下文管理器）          │
│  └─ __init__.py     公共导出                                  │
├─────────────────────────────────────────────────────────────┤
│  PyO3 FFI Layer (crates/chainforge-py)                      │
│  ├─ PyTransaction / PyBlockHeader / PyMerkleTree            │
│  ├─ PySecretKey / PyPublicKey                               │
│  └─ PyStorageEngine (InMemory / RocksDB)                    │
├─────────────────────────────────────────────────────────────┤
│  Rust Core (crates/)                                        │
│  ├─ chainforge-core      交易、区块、Merkle、RLP            │
│  ├─ chainforge-crypto    哈希、签名                         │
│  ├─ chainforge-storage   KV 存储抽象                        │
│  ├─ chainforge-p2p       网络层                            │
│  ├─ chainforge-mempool   交易池                            │
│  ├─ chainforge-consensus HotStuff BFT                      │
│  ├─ chainforge-evm       revm 执行引擎                      │
│  ├─ chainforge-rpc       JSON-RPC 服务                     │
│  └─ chainforge-error     统一错误类型                       │
└─────────────────────────────────────────────────────────────┘
```

## 开发命令速查

| 命令 | 说明 |
|------|------|
| `pixi run dev-build` | 可编辑安装 Python 包 |
| `pixi run test` | 运行全部测试 |
| `pixi run test-rust` | `cargo test --workspace` |
| `pixi run test-py` | `pytest src/tests` |
| `pixi run typecheck` | `mypy src/chainforge` |
| `pixi run fmt` | `cargo fmt && ruff format` |
| `pixi run bench` | `cargo bench --workspace` |

## 基准测试参考

| 测试项 | 规模 | 耗时 |
|--------|------|------|
| Merkle root | 1,000 leaves | ~565 µs |
| Merkle root | 10,000 leaves | ~5.99 ms |
| ECDSA sign | — | ~52 µs |
| ECDSA verify | — | ~64 µs |

## 测试覆盖

- **Rust**: 101 个单元测试全部通过
- **Python**: 31 个单元测试全部通过
- **Clippy**: 零警告（`-D warnings`）
- **mypy**: 零类型错误

## 相关文档

- [架构设计文档](../design/framework/design.md)
- [AGENTS.md](../AGENTS.md) — 开发代理指南
- [英文版用户指南](./README.en.md)

## License

MIT OR Apache-2.0
