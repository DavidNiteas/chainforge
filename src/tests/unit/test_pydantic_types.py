"""测试 Pydantic 输入校验模型。"""

import pytest
from pydantic import ValidationError

from chainforge.types import BlockInput, TxInput


def test_txinput_valid():
    tx = TxInput(nonce=1, gas_price=10, gas_limit=21000, value=100)
    assert tx.nonce == 1
    assert tx.gas_price == 10
    assert tx.gas_limit == 21000
    assert tx.value == 100
    assert tx.to is None
    assert tx.data == b""


def test_txinput_defaults():
    tx = TxInput(nonce=0, gas_price=0)
    assert tx.gas_limit == 21_000
    assert tx.value == 0
    assert tx.data == b""


def test_txinput_negative_nonce():
    with pytest.raises(ValidationError):
        TxInput(nonce=-1, gas_price=0)


def test_txinput_gas_limit_too_low():
    with pytest.raises(ValidationError):
        TxInput(nonce=0, gas_price=0, gas_limit=20_999)


def test_txinput_to_max_length():
    with pytest.raises(ValidationError):
        TxInput(nonce=0, gas_price=0, to=b"\x00" * 21)


def test_blockinput_valid():
    parent = b"\x00" * 32
    block = BlockInput(parent_hash=parent, number=1, timestamp=1234567890)
    assert block.number == 1
    assert block.timestamp == 1234567890
    assert block.extra_data == b""


def test_blockinput_extra_data_too_long():
    parent = b"\x00" * 32
    with pytest.raises(ValidationError):
        BlockInput(parent_hash=parent, number=0, timestamp=0, extra_data=b"\x00" * 33)


def test_blockinput_parent_hash_wrong_length():
    with pytest.raises(ValidationError):
        BlockInput(parent_hash=b"\x00" * 31, number=0, timestamp=0)
