"""高层 Pythonic API 封装。"""

from contextlib import asynccontextmanager
from typing import AsyncGenerator

from chainforge._internal import InMemoryStorage as _InMemoryStorage


@asynccontextmanager
async def open_db() -> AsyncGenerator[_InMemoryStorage, None]:
    """异步上下文管理器，创建并自动释放内存存储实例。

    Yields
    ------
    InMemoryStorage
        可异步读写的内存存储引擎。
    """
    db = _InMemoryStorage()
    try:
        yield db
    finally:
        pass
