# Kilnchain

[![Rust Tests](https://img.shields.io/badge/Rust%20Tests-102%20passing-success)](./docs/README.en.md)
[![Python Tests](https://img.shields.io/badge/Python%20Tests-97%20passing-success)](./docs/README.en.md)
[![License](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue)](./LICENSE-MIT)

High-performance blockchain core library with Python bindings.

**[📖 English User Guide](./docs/README.en.md) | [📖 中文用户指南](./docs/README.zh.md)**

---

## What is Kilnchain?

Kilnchain is a blockchain infrastructure toolkit built in **Rust** and exposed to **Python** via PyO3. It provides the building blocks for a full EVM-compatible chain — cryptography, consensus, networking, execution, and RPC — all accessible through ergonomic Python APIs.

Whether you're building a custom L2, prototyping a new consensus mechanism, or running chain analytics, Kilnchain gives you production-grade Rust performance without leaving the Python ecosystem.

## Quick Start

```bash
# Install with Pixi (recommended)
pixi install
pixi run dev-build

# Run everything
pixi run test
```

See the [User Guide](./docs/README.en.md) for detailed installation instructions, API examples, and architecture overview.

## Core Modules

| Module | Description |
|--------|-------------|
| **kilnchain-crypto** | SHA-256, Keccak-256, secp256k1 ECDSA |
| **kilnchain-core** | Transactions, blocks, Merkle trees, RLP |
| **kilnchain-storage** | KV storage with in-memory and RocksDB backends |
| **kilnchain-p2p** | Noise-encrypted P2P with Kademlia + Gossip |
| **kilnchain-mempool** | Gas-prioritized transaction pool |
| **kilnchain-consensus** | HotStuff BFT consensus engine |
| **kilnchain-evm** | revm-based EVM execution |
| **kilnchain-rpc** | Ethereum-compatible JSON-RPC over HTTP & WebSocket |
| **kilnchain-py** | PyO3 bindings for all core types |

## Documentation

- **[English User Guide](./docs/README.en.md)** — Installation, API examples, architecture, benchmarks
- **[中文用户指南](./docs/README.zh.md)** — 安装、API 示例、架构、基准测试
- **[Architecture Design](./design/framework/design.md)** — Low-level design documents (Chinese)
- **[AGENTS.md](./AGENTS.md)** — Guide for AI coding agents

## Test Status

```
cargo test --workspace   # 102 Rust tests passing
pixi run test-py         # 97 Python tests passing
pixi run typecheck       # mypy zero errors
```

## License

Kilnchain is dual-licensed under either:

- **[MIT License](./LICENSE-MIT)** — short and permissive
- **[Apache License, Version 2.0](./LICENSE-APACHE)** — includes explicit patent grant

at your option. This dual-licensing approach is the de facto standard in the Rust ecosystem (used by Rust itself, PyO3, Tokio, and Serde) and maximizes compatibility with downstream projects.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in Kilnchain by you, as defined in the Apache-2.0 license, shall be dual-licensed as above, without any additional terms or conditions.
