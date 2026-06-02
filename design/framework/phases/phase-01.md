# Phase 01: 最小可编译工程骨架

## 目标

搭建 Monorepo 目录结构，配置好 Pixi + Rust workspace + Maturin，使所有 crate 能空编译通过。此阶段不实现任何业务逻辑，只确保「工程机器」运转正常。

---

## 交付物清单

### 配置文件

| 文件 | 说明 |
|------|------|
| `Cargo.toml` | Workspace 根，定义 members 和统一依赖版本 |
| `pixi.toml` | Pixi 项目配置，含 Python 依赖与 task 定义 |
| `pyproject.toml` | Python 包元数据 + maturin 配置 + pytest/mypy 设置 |
| `rust-toolchain.toml` | 锁定 stable 工具链及组件 |

### 空 Crate 结构

| Crate | 路径 | 说明 |
|-------|------|------|
| `kilnchain-core` | `crates/kilnchain-core/` | 纯 Rust 核心：区块、交易、Merkle 树 |
| `kilnchain-crypto` | `crates/kilnchain-crypto/` | 密码学原语 |
| `kilnchain-storage` | `crates/kilnchain-storage/` | KV 存储抽象 |
| `kilnchain-py` | `crates/kilnchain-py/` | 唯一依赖 pyo3 的绑定层 |

### Python 包结构

| 文件 | 说明 |
|------|------|
| `src/kilnchain/__init__.py` | 空文件（占位） |
| `src/kilnchain/py.typed` | PEP 561 类型标记 |
| `src/tests/conftest.py` | pytest 共享配置（空） |
| `src/tests/unit/.gitkeep` | 占位 |
| `src/tests/integration/.gitkeep` | 占位 |

---

## 关键配置参考

### Cargo.toml (workspace root)

```toml
[workspace]
members = [
    "crates/kilnchain-core",
    "crates/kilnchain-crypto",
    "crates/kilnchain-storage",
    "crates/kilnchain-py",
]
resolver = "2"

[workspace.package]
version = "0.1.0"
edition = "2021"
license = "MIT OR Apache-2.0"
repository = "https://github.com/your-org/kilnchain"

[workspace.dependencies]
serde = { version = "1.0", features = ["derive"] }
thiserror = "1.0"
anyhow = "1.0"
```

### crates/kilnchain-core/Cargo.toml

```toml
[package]
name = "kilnchain-core"
version.workspace = true
edition.workspace = true

[dependencies]
serde = { workspace = true }
thiserror = { workspace = true }
kilnchain-crypto = { path = "../kilnchain-crypto" }

[dev-dependencies]
proptest = "1.5"
```

### crates/kilnchain-py/Cargo.toml

```toml
[package]
name = "kilnchain-py"
version.workspace = true
edition.workspace = true
crate-type = ["cdylib"]

[dependencies]
pyo3 = { version = "0.23", features = ["extension-module"] }
kilnchain-core = { path = "../kilnchain-core" }
kilnchain-crypto = { path = "../kilnchain-crypto" }
kilnchain-storage = { path = "../kilnchain-storage" }
```

### pyproject.toml (关键段)

```toml
[build-system]
requires = ["maturin>=1.7.0"]
build-backend = "maturin"

[tool.maturin]
manifest-path = "crates/kilnchain-py/Cargo.toml"
module-name = "kilnchain._internal"
python-source = "src"
```

---

## 验收标准（必须全部通过）

- [ ] `cargo check --workspace` 零错误、零警告
- [ ] `pixi install` 成功完成环境创建
- [ ] `pixi run dev-build` 成功编译出 `_internal` 扩展模块
- [ ] `python -c "import kilnchain._internal; print('ok')"` 不报错
- [ ] 目录结构符合 design.md 中的规划

---

## 预计工时

0.5 ~ 1 天

---

## 下一步

Phase 02: 跨语言错误体系
