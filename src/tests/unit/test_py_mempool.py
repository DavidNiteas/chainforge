"""测试 PyO3 Mempool 绑定。"""

import pytest


def test_mempool_insert_and_get():
    from chainforge._internal import Mempool, Transaction

    pool = Mempool()
    tx = Transaction(nonce=1, gas_price=10, gas_limit=21000, to=b"\x01" * 20, value=100)
    pool.insert(tx)
    h = tx.hash()
    assert pool.contains(h)
    assert pool.len() == 1
    retrieved = pool.get(h)
    assert retrieved is not None
    assert retrieved.nonce == 1


def test_mempool_remove():
    from chainforge._internal import Mempool, Transaction

    pool = Mempool()
    tx = Transaction(nonce=2, gas_price=10, gas_limit=21000, to=b"\x01" * 20, value=100)
    h = tx.hash()
    pool.insert(tx)
    removed = pool.remove(h)
    assert removed is not None
    assert pool.is_empty()
    assert not pool.contains(h)


def test_mempool_priority_queue():
    from chainforge._internal import Mempool, Transaction

    pool = Mempool()
    pool.insert(Transaction(nonce=0, gas_price=10, gas_limit=21000, to=b"\x01" * 20, value=10))
    pool.insert(Transaction(nonce=0, gas_price=100, gas_limit=21000, to=b"\x02" * 20, value=10))
    pool.insert(Transaction(nonce=0, gas_price=50, gas_limit=21000, to=b"\x03" * 20, value=10))

    selected = pool.pop_highest_priority(2)
    assert len(selected) == 2
    assert selected[0].gas_price == 100
    assert selected[1].gas_price == 50
    assert pool.len() == 1


def test_mempool_nonce_tracking():
    from chainforge._internal import Mempool, Transaction

    pool = Mempool()
    sender = b"\xab" * 20
    tx1 = Transaction(nonce=0, gas_price=10, gas_limit=21000, to=sender, value=10)
    tx2 = Transaction(nonce=1, gas_price=10, gas_limit=21000, to=sender, value=10)
    pool.insert(tx1)
    pool.insert(tx2)

    assert pool.next_nonce(sender) == 2
    tx3 = Transaction(nonce=2, gas_price=10, gas_limit=21000, to=sender, value=10)
    assert pool.is_nonce_valid(tx3)


def test_mempool_capacity_eviction():
    from chainforge._internal import Mempool, Transaction

    pool = Mempool(capacity=3)
    pool.insert(Transaction(nonce=0, gas_price=10, gas_limit=21000, to=b"\x01" * 20, value=10))
    pool.insert(Transaction(nonce=0, gas_price=20, gas_limit=21000, to=b"\x02" * 20, value=10))
    pool.insert(Transaction(nonce=0, gas_price=30, gas_limit=21000, to=b"\x03" * 20, value=10))
    assert pool.len() == 3

    pool.insert(Transaction(nonce=0, gas_price=40, gas_limit=21000, to=b"\x04" * 20, value=10))
    assert pool.len() == 3
    # gas_price=10 的应被驱逐
    for h, tx in pool.txs().items():
        assert tx.gas_price != 10
