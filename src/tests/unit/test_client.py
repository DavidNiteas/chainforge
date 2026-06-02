"""测试高层 Python API 封装。"""

import pytest

from kilnchain.client import open_db


@pytest.mark.asyncio
async def test_open_db_context_manager():
    async with open_db() as db:
        await db.put(b"key1", b"value1")
        result = await db.get(b"key1")
        assert result == b"value1"


@pytest.mark.asyncio
async def test_open_db_delete():
    async with open_db() as db:
        await db.put(b"key1", b"value1")
        await db.delete(b"key1")
        result = await db.get(b"key1")
        assert result is None
