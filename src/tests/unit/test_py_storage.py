"""测试 PyO3 存储绑定（异步）。"""

import pytest


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


@pytest.mark.asyncio
async def test_cached_storage_put_get():
    from chainforge._internal import CachedStorage
    db = CachedStorage(capacity=10)
    await db.put(b"key1", b"value1")
    result = await db.get(b"key1")
    assert result == b"value1"
    # 再次读取（缓存命中）
    result2 = await db.get(b"key1")
    assert result2 == b"value1"


@pytest.mark.asyncio
async def test_cached_storage_delete():
    from chainforge._internal import CachedStorage
    db = CachedStorage(capacity=10)
    await db.put(b"key1", b"value1")
    await db.delete(b"key1")
    result = await db.get(b"key1")
    assert result is None
