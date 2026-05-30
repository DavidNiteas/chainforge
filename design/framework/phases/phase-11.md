# Phase 11: CI/CD 与基准测试

## 目标

建立 GitHub Actions 持续集成流水线，配置 Criterion 基准测试，编写快速开始教程，完成项目的「可发布」状态。

---

## 交付物清单

### CI/CD

| 文件 | 说明 |
|------|------|
| `.github/workflows/ci.yml` | 三阶段流水线：rust-checks, python-checks, build-wheels |

### 基准测试

| 文件 | 说明 |
|------|------|
| `crates/chainforge-crypto/benches/sign_bench.rs` | Secp256k1 签名/验签吞吐量 |
| `crates/chainforge-core/benches/merkle_bench.rs` | Merkle 根计算延迟（1/10/100/1000/10000 叶子） |
| `crates/chainforge-storage/benches/rocksdb_bench.rs` | 随机读写 IOPS |

### 文档

| 文件 | 说明 |
|------|------|
| `docs/tutorials/01-quickstart.md` | 创建私钥 → 签名交易 → 计算 Merkle 根 |
| `docs/adr/001-why-rocksdb.md` | 架构决策记录：为何选择 RocksDB |
| `CHANGELOG.md` | 初始版本记录（遵循 Keep a Changelog） |

---

## CI 工作流规格

### Job 1: rust-checks

```yaml
rust-checks:
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@stable
      with:
        components: rustfmt, clippy
    - run: cargo fmt -- --check
    - run: cargo clippy --workspace -- -D warnings
    - run: cargo test --workspace --all-features
    - run: cargo bench --workspace -- --no-run
```

### Job 2: python-checks

```yaml
python-checks:
  runs-on: ${{ matrix.os }}
  strategy:
    matrix:
      os: [ubuntu-latest, macos-latest, windows-latest]
      python-version: ["3.10", "3.11", "3.12"]
  steps:
    - uses: actions/checkout@v4
    - uses: prefix-dev/setup-pixi@v0.8.0
      with:
        pixi-version: v0.25.0
        cache: true
    - run: pixi run dev-build
    - run: pixi run test-py
    - run: pixi run typecheck
```

### Job 3: build-wheels

```yaml
build-wheels:
  needs: [rust-checks, python-checks]
  runs-on: ${{ matrix.os }}
  strategy:
    matrix:
      os: [ubuntu-latest, macos-13, macos-14, windows-latest]
  steps:
    - uses: actions/checkout@v4
    - uses: PyO3/maturin-action@v1
      with:
        target: ${{ matrix.target }}
        args: --release --out dist
        sccache: 'true'
    - uses: actions/upload-artifact@v4
      with:
        name: wheels-${{ matrix.os }}
        path: dist
```

---

## 基准测试规格

### sign_bench.rs

测量：1. 生成签名；2. 验证签名。
输入规模：32 字节消息，批量 100/1000/10000 次。

### merkle_bench.rs

测量：构建树 + 计算根的时间。
叶子规模：1, 10, 100, 1_000, 10_000。
叶子数据：随机 32 字节。

### rocksdb_bench.rs

测量：随机键（32 字节）的 put/get 吞吐量。
数据集：1000 条记录，每条 value 1KB。

---

## 验收标准（必须全部通过）

- [ ] 推送代码后 GitHub Actions 全部绿灯
- [ ] `cargo bench --workspace` 成功运行，生成 HTML 报告
- [ ] `cargo clippy --workspace -- -D warnings` 零警告
- [ ] `docs/tutorials/01-quickstart.md` 中的代码可在干净环境中复制粘贴运行
- [ ] 版本号在三处同步：`Cargo.toml` workspace, `pyproject.toml`, `pixi.toml`

---

## 预计工时

1 ~ 2 天

---

## 前置依赖

Phase 01 ~ Phase 10 全部完成

---

## 里程碑

至此，Chainforge v0.1.0 完成，具备：
- 完整的 Rust 核心库（密码学、数据结构、存储）
- Python 绑定与高层 API
- 跨平台 CI/CD 与自动化测试
- 性能基准基线

可进入迭代开发阶段：根据实际需求扩展功能（P2P 网络、共识算法、EVM 兼容层等）。
