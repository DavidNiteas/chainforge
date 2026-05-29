"""测试 PyO3 存储绑定（异步）。"""

import pytest
import asyncio


@pytest.mark.asyncio
async def test_inmemory_storage_put_get():
    from chainforge._internal import InMemoryStorage
    db = InMemoryStorage()
    await db.put(b"key1", b"value1")
    result = await db.get(b"key1")
    assert result == b"value1"


@pytest.mark.asyncio
async def test_inmemory_storage_delete():
    from chainforge._internal import InMemoryStorage
    db = InMemoryStorage()
    await db.put(b"key1", b"value1")
    await db.delete(b"key1")
    result = await db.get(b"key1")
    assert result is None
