"""测试 PyO3 Transaction / BlockHeader 绑定。"""

import pytest


def test_transaction_construct():
    from chainforge._internal import Transaction
    tx = Transaction(nonce=1, gas_price=10, gas_limit=21000, value=100)
    assert tx.nonce == 1
    assert tx.gas_price == 10
    assert tx.gas_limit == 21000
    assert tx.value == 100


def test_transaction_hash():
    from chainforge._internal import Transaction
    tx = Transaction()
    h = tx.hash()
    assert isinstance(h, bytes)
    assert len(h) == 32


def test_transaction_rlp_roundtrip():
    from chainforge._internal import Transaction
    tx = Transaction(nonce=5, gas_price=20, value=50)
    encoded = tx.encode_rlp()
    decoded = Transaction.decode_rlp(encoded)
    assert decoded.nonce == 5
    assert decoded.gas_price == 20
    assert decoded.value == 50


def test_blockheader_construct():
    from chainforge._internal import BlockHeader
    parent = b"\x00" * 32
    state_root = b"\x01" * 32
    txs_root = b"\x02" * 32
    header = BlockHeader(
        parent_hash=parent,
        number=1,
        timestamp=1234567890,
        difficulty=1000,
        nonce=0,
        extra_data=b"\xde\xad",
        state_root=state_root,
        txs_root=txs_root,
    )
    assert header.number == 1
    h = header.hash()
    assert isinstance(h, bytes)
    assert len(h) == 32


def test_blockheader_extra_data_too_long():
    from chainforge._internal import BlockHeader
    with pytest.raises(ValueError):
        BlockHeader(
            parent_hash=b"\x00" * 32,
            number=0,
            timestamp=0,
            difficulty=0,
            nonce=0,
            extra_data=b"\x00" * 33,
            state_root=b"\x00" * 32,
            txs_root=b"\x00" * 32,
        )
