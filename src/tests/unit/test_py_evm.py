"""测试 PyO3 EVM 绑定。"""

import pytest


ALICE = b"\xaa" * 20
BOB = b"\xbb" * 20


def test_evm_transfer():
    from chainforge._internal import EvmExecutor, EvmState

    state = EvmState()
    state.set_balance(ALICE, 1000)

    executor = EvmExecutor(state)
    result = executor.transfer(ALICE, BOB, 300)
    assert result.is_success()
    assert result.gas_used() > 0

    assert executor.balance(ALICE) == 700
    assert executor.balance(BOB) == 300


def test_evm_transfer_insufficient_balance():
    from chainforge._internal import EvmExecutor, EvmState

    state = EvmState()
    state.set_balance(ALICE, 1000)

    executor = EvmExecutor(state)
    # revm 在余额不足时会在 transact() 阶段抛出 Transaction 错误，映射为 ValueError
    with pytest.raises(ValueError):
        executor.transfer(ALICE, BOB, 2000)


def test_evm_deploy():
    from chainforge._internal import EvmExecutor, EvmState

    # 极简 counter 合约 bytecode:
    # PUSH1 0x00 CALLDATALOAD PUSH1 0x01 ADD PUSH1 0x00 MSTORE PUSH1 0x20 PUSH1 0x00 RETURN
    code = bytes(
        [
            0x60,
            0x00,
            0x35,
            0x60,
            0x01,
            0x01,
            0x60,
            0x00,
            0x52,
            0x60,
            0x20,
            0x60,
            0x00,
            0xF3,
        ]
    )

    state = EvmState()
    state.set_balance(ALICE, 10_000)

    executor = EvmExecutor(state)
    result = executor.deploy(ALICE, code, 0)
    assert result.is_success()
    assert result.gas_used() > 0
    assert executor.nonce(ALICE) == 1


def test_evm_state_methods():
    from chainforge._internal import EvmState

    state = EvmState()
    # 注意：set_code 会替换整个 AccountInfo（balance 归零），所以先 set_code 再 set_balance
    state.set_code(ALICE, b"\x60\x00\x00")
    state.set_balance(ALICE, 10**18)
    state.set_nonce(ALICE, 5)
    state.set_storage(ALICE, 0, 42)

    assert state.balance(ALICE) == 10**18
    assert state.nonce(ALICE) == 5
    assert state.code(ALICE) == b"\x60\x00\x00"


def test_evm_execution_result_properties():
    from chainforge._internal import EvmExecutor, EvmState

    state = EvmState()
    state.set_balance(ALICE, 1000)

    executor = EvmExecutor(state)
    result = executor.transfer(ALICE, BOB, 100)

    assert result.is_success()
    assert not result.is_revert()
    assert not result.is_halt()
    assert result.gas_used() > 0
    assert result.output() is None or isinstance(result.output(), bytes)
    assert result.contract_address() is None
