"""EVM 执行层高层封装（Layer 2/3）。"""

from typing import Optional

from chainforge._internal import EvmExecutor as _EvmExecutor
from chainforge._internal import EvmState as _EvmState
from chainforge._internal import ExecutionResult as _ExecutionResult


class EvmState:
    """内存中的 EVM 状态数据库。"""

    def __init__(self) -> None:
        self._inner = _EvmState()

    @property
    def _raw(self) -> _EvmState:
        return self._inner

    def set_balance(self, address: bytes, balance: int) -> "EvmState":
        self._inner.set_balance(address, balance)
        return self

    def set_nonce(self, address: bytes, nonce: int) -> "EvmState":
        self._inner.set_nonce(address, nonce)
        return self

    def set_code(self, address: bytes, code: bytes) -> "EvmState":
        self._inner.set_code(address, code)
        return self

    def set_storage(self, address: bytes, slot: int, value: int) -> "EvmState":
        self._inner.set_storage(address, slot, value)
        return self

    def balance(self, address: bytes) -> int:
        return self._inner.balance(address)  # type: ignore[no-any-return]

    def nonce(self, address: bytes) -> int:
        return self._inner.nonce(address)  # type: ignore[no-any-return]

    def code(self, address: bytes) -> bytes:
        return self._inner.code(address)  # type: ignore[no-any-return]


class EvmExecutor:
    """EVM 交易执行器。"""

    def __init__(self, state: EvmState) -> None:
        self._inner = _EvmExecutor(state._raw)

    def transfer(self, from_addr: bytes, to_addr: bytes, value: int) -> "ExecutionResult":
        raw = self._inner.transfer(from_addr, to_addr, value)
        return ExecutionResult(raw)

    def deploy(self, from_addr: bytes, code: bytes, value: int = 0) -> "ExecutionResult":
        raw = self._inner.deploy(from_addr, code, value)
        return ExecutionResult(raw)

    def call(self, from_addr: bytes, to_addr: bytes, data: bytes, value: int = 0) -> "ExecutionResult":
        raw = self._inner.call(from_addr, to_addr, data, value)
        return ExecutionResult(raw)

    def balance(self, address: bytes) -> int:
        return self._inner.balance(address)  # type: ignore[no-any-return]

    def nonce(self, address: bytes) -> int:
        return self._inner.nonce(address)  # type: ignore[no-any-return]


class ExecutionResult:
    """EVM 执行结果包装。"""

    def __init__(self, raw: _ExecutionResult) -> None:
        self._raw = raw

    def is_success(self) -> bool:
        return self._raw.is_success()  # type: ignore[no-any-return]

    def is_revert(self) -> bool:
        return self._raw.is_revert()  # type: ignore[no-any-return]

    def is_halt(self) -> bool:
        return self._raw.is_halt()  # type: ignore[no-any-return]

    @property
    def gas_used(self) -> int:
        return self._raw.gas_used()  # type: ignore[no-any-return]

    @property
    def output(self) -> Optional[bytes]:
        return self._raw.output()  # type: ignore[no-any-return]

    @property
    def contract_address(self) -> Optional[bytes]:
        return self._raw.contract_address()  # type: ignore[no-any-return]
