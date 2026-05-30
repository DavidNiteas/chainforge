"""Pydantic 输入校验模型。"""

from typing import Optional

from pydantic import BaseModel, Field


class TxInput(BaseModel):
    """交易输入参数模型。"""

    nonce: int = Field(ge=0, le=2**64 - 1)
    gas_price: int = Field(ge=0)
    gas_limit: int = Field(default=21_000, ge=21_000)
    to: Optional[bytes] = Field(default=None, max_length=20)
    value: int = Field(default=0, ge=0)
    data: bytes = b""


class BlockInput(BaseModel):
    """区块输入参数模型。"""

    parent_hash: bytes = Field(min_length=32, max_length=32)
    number: int = Field(ge=0)
    timestamp: int = Field(ge=0)
    extra_data: bytes = Field(default=b"", max_length=32)
