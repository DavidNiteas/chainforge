"""测试 PyO3 Block / BlockBuilder / produce_block 绑定。"""

import pytest


def test_block_builder():
    from chainforge._internal import BlockBuilder, Transaction

    txs = [
        Transaction(nonce=0, gas_price=10, gas_limit=21000, to=b"\xab" * 20, value=100),
        Transaction(nonce=1, gas_price=20, gas_limit=21000, to=b"\xcb" * 20, value=200),
    ]
    block = (
        BlockBuilder(parent_hash=b"\x00" * 32, number=1)
        .timestamp(1234567890)
        .extra_data(b"py")
        .transactions(txs)
        .build()
    )
    assert block.header.number == 1
    assert block.header.timestamp == 1234567890
    assert len(block.transactions) == 2
    h = block.hash()
    assert isinstance(h, bytes)
    assert len(h) == 32


def test_block_rlp_roundtrip():
    from chainforge._internal import Block, BlockHeader, Transaction

    tx = Transaction(nonce=0, gas_price=1, gas_limit=21000, to=b"\xab" * 20, value=100)
    header = BlockHeader(
        parent_hash=b"\x00" * 32,
        number=1,
        timestamp=0,
        difficulty=0,
        nonce=0,
        extra_data=b"",
        state_root=b"\x00" * 32,
        txs_root=b"\x00" * 32,
    )
    block = Block(header=header, transactions=[tx], uncle_headers=[])
    encoded = block.to_rlp()
    decoded = Block.from_rlp(encoded)
    assert decoded.header.number == 1
    assert len(decoded.transactions) == 1


def test_mempool_produce_block():
    from chainforge._internal import Mempool, Transaction

    pool = Mempool(capacity=100)
    pool.insert(
        Transaction(nonce=0, gas_price=100, gas_limit=21000, to=b"\x01" * 20, value=10)
    )
    pool.insert(
        Transaction(nonce=0, gas_price=50, gas_limit=21000, to=b"\x02" * 20, value=10)
    )
    pool.insert(
        Transaction(nonce=0, gas_price=200, gas_limit=21000, to=b"\x03" * 20, value=10)
    )

    block = pool.produce_block(
        parent_hash=b"\x00" * 32,
        number=1,
        timestamp=1000,
        max_txs=2,
    )
    assert len(block.transactions) == 2
    assert block.transactions[0].gas_price >= block.transactions[1].gas_price
    assert pool.len() == 1
