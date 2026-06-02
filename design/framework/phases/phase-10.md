# Phase 10: Python API 与类型校验层

## 目标

在 Rust 绑定之上，提供 Pythonic 的高层 API。使用 Pydantic v2 做输入校验，使用上下文管理器做资源生命周期管理，确保类型提示完整并通过 mypy 检查。

---

## 交付物清单

### Python 源码

| 文件 | 说明 |
|------|------|
| `src/kilnchain/__init__.py` | 统一导出公共 API，定义 `__all__` |
| `src/kilnchain/types.py` | Pydantic 输入模型：`TxInput`, `BlockInput` 等 |
| `src/kilnchain/client.py` | 高层封装：`open_db()`, `create_transaction()` 等 |
| `src/kilnchain/py.typed` | PEP 561 标记文件 |

### 配置变更

| 文件 | 变更 |
|------|------|
| `pyproject.toml` | 增加 `[tool.mypy]` 配置段 |

### 测试

| 文件 | 说明 |
|------|------|
| `src/tests/unit/test_types.py` | Pydantic 校验边界测试 |
| `src/tests/unit/test_client.py` | `open_db()` 上下文管理器测试 |
| `src/tests/integration/test_e2e.py` | 端到端：创建交易 → 签名 → 计算 Merkle 根 |

---

## 核心代码规格

### __init__.py 导出规范

```python
from kilnchain._internal import (
    Transaction as _Transaction,
    BlockHeader as _BlockHeader,
    MerkleTree as _MerkleTree,
    Secp256k1 as _Secp256k1,
    RocksDB as _RocksDB,
    KilnchainError,
)

__all__ = [
    "TxInput", "BlockInput",
    "Transaction", "BlockHeader", "MerkleTree",
    "Secp256k1", "open_db", "KilnchainError",
]
```

### Pydantic 模型

```python
from pydantic import BaseModel, Field
from typing import Optional

class TxInput(BaseModel):
    nonce: int = Field(ge=0, le=2**64 - 1)
    gas_price: int = Field(ge=0)
    gas_limit: int = Field(default=21_000, ge=21_000)
    to: Optional[bytes] = Field(default=None, max_length=20)
    value: int = Field(default=0, ge=0)
    data: bytes = b""

class BlockInput(BaseModel):
    parent_hash: bytes = Field(min_length=32, max_length=32)
    number: int = Field(ge=0)
    timestamp: int = Field(ge=0)
    extra_data: bytes = Field(default=b"", max_length=32)
```

### 上下文管理器

```python
from contextlib import contextmanager
from typing import Generator
from kilnchain._internal import RocksDB as _RocksDB

@contextmanager
def open_db(path: str) -> Generator[_RocksDB, None, None]:
    db = _RocksDB(path)
    try:
        yield db
    finally:
        db.close()
```

### mypy 配置

```toml
[tool.mypy]
python_version = "3.10"
strict = true
warn_return_any = true
warn_unused_configs = true
ignore_missing_imports = true
```

---

## 验收标准（必须全部通过）

- [ ] `pixi run typecheck` 零错误
- [ ] `pytest src/tests/unit/test_types.py` 通过
- [ ] Pydantic 对 `nonce=-1` 抛出 `ValidationError`
- [ ] Pydantic 对 `to=b'\x00' * 21` 抛出 `ValidationError`
- [ ] `open_db()` 上下文管理器在退出时调用 `db.close()`（可用 mock 验证）
- [ ] NumPy 风格 docstring 覆盖所有公共类和方法（可抽样检查）

---

## 预计工时

1 ~ 2 天

---

## 前置依赖

Phase 09: PyO3 完整绑定层（提供 `_internal` 模块）

---

## 下一步

Phase 11: CI/CD 与基准测试
