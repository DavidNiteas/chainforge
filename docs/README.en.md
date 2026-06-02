# Kilnchain User Guide

> [中文版](./README.zh.md)

## Introduction

**Kilnchain** is a high-performance blockchain core library implemented in **Rust**, with **PyO3** bindings for Python. It targets data scientists, quantitative researchers, and blockchain developers who need rapid prototyping — offering familiar Python APIs backed by high-performance Rust primitives.

## Core Features

| Module | Functionality | Status |
|--------|--------------|--------|
| **Cryptography** | SHA-256, Keccak-256, RIPEMD-160, secp256k1 ECDSA | ✅ |
| **Core Types** | Transaction, BlockHeader, Block, MerkleTree, RLP codec | ✅ |
| **Storage** | In-memory backend, RocksDB, LRU cache | ✅ |
| **P2P Network** | Noise XX handshake, Kademlia routing, Gossip broadcast, block sync | ✅ |
| **Mempool** | CRUD, Gas priority queue, nonce replay protection, capacity eviction | ✅ |
| **Consensus** | HotStuff BFT — BlockTree, QuorumCertificate, Safety/Liveness | ✅ |
| **EVM** | revm integration — transfer, contract deploy, contract call | ✅ |
| **RPC** | Ethereum-compatible JSON-RPC (HTTP + WebSocket subscriptions) | ✅ |
| **Light Client** | Header sync, MerkleProof verification, MPT state proofs | ✅ |
| **Python Bindings** | PyO3 core types + Pydantic input validation | ✅ |

## Quick Start

### Requirements

- Rust 1.70+ (stable)
- Python 3.10+
- [Pixi](https://pixi.sh) (recommended) or pip + virtualenv

### Installation

```bash
# Clone the repository
git clone https://github.com/DavidNiteas/kilnchain.git
cd kilnchain

# Install dependencies with Pixi (handles both Python + Rust toolchains)
pixi install

# Build and install the Python editable package
pixi run dev-build
```

### Running Tests

```bash
# Full test suite (Rust + Python)
pixi run test

# Rust only
cargo test --workspace

# Python only
pixi run test-py

# Type checking
pixi run typecheck
```

## Usage Examples

### Python API

```python
from kilnchain import Transaction, BlockHeader, MerkleTree, SecretKey

# Create a transaction
tx = Transaction(nonce=0, gas_price=10, gas_limit=21000,
                 to=b'\xab' * 20, value=1000, data=b'')
print("tx hash:", tx.hash().hex())

# Merkle tree
leaves = [bytes([i] * 32) for i in range(4)]
tree = MerkleTree(leaves)
print("merkle root:", tree.root().hex())

# ECDSA signing
sk = SecretKey.random()
pk = sk.public_key()
msg = b"hello kilnchain"
sig = sk.sign(msg)
assert pk.verify(msg, sig)
```

### Rust API

```rust
use kilnchain_core::{Transaction, BlockHeader, MerkleTree};
use kilnchain_crypto::ecdsa::SecretKey;

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

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│  Python Layer (src/kilnchain)                              │
│  ├─ types.py        Pydantic input validation               │
│  ├─ client.py       High-level API (open_db context mgr)    │
│  └─ __init__.py     Public re-exports                       │
├─────────────────────────────────────────────────────────────┤
│  PyO3 FFI Layer (crates/kilnchain-py)                      │
│  ├─ PyTransaction / PyBlockHeader / PyMerkleTree            │
│  ├─ PySecretKey / PyPublicKey                               │
│  └─ PyStorageEngine (InMemory / RocksDB)                    │
├─────────────────────────────────────────────────────────────┤
│  Rust Core (crates/)                                        │
│  ├─ kilnchain-core      Transactions, blocks, Merkle, RLP  │
│  ├─ kilnchain-crypto    Hashing, signatures                │
│  ├─ kilnchain-storage   KV storage abstraction             │
│  ├─ kilnchain-p2p       Networking layer                   │
│  ├─ kilnchain-mempool   Transaction pool                   │
│  ├─ kilnchain-consensus HotStuff BFT                       │
│  ├─ kilnchain-evm       revm execution engine              │
│  ├─ kilnchain-rpc       JSON-RPC service                   │
│  └─ kilnchain-error     Unified error types                │
└─────────────────────────────────────────────────────────────┘
```

## Development Commands

| Command | Description |
|---------|-------------|
| `pixi run dev-build` | Editable install of Python package |
| `pixi run test` | Run full test suite |
| `pixi run test-rust` | `cargo test --workspace` |
| `pixi run test-py` | `pytest src/tests` |
| `pixi run typecheck` | `mypy src/kilnchain` |
| `pixi run fmt` | `cargo fmt && ruff format` |
| `pixi run bench` | `cargo bench --workspace` |

## Benchmarks

| Benchmark | Scale | Time |
|-----------|-------|------|
| Merkle root | 1,000 leaves | ~565 µs |
| Merkle root | 10,000 leaves | ~5.99 ms |
| ECDSA sign | — | ~52 µs |
| ECDSA verify | — | ~64 µs |

## Test Coverage

- **Rust**: 101 unit tests passing
- **Python**: 31 unit tests passing
- **Clippy**: zero warnings (`-D warnings`)
- **mypy**: zero type errors

## Related Documentation

- [Architecture Design Docs](../design/framework/design.md)
- [AGENTS.md](../AGENTS.md) — Developer agent guide
- [Chinese User Guide](./README.zh.md)

## License

MIT OR Apache-2.0
