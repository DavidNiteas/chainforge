"""测试 Rust 错误到 Python 异常的映射。"""

import pytest


def test_chainforge_error_importable():
    """ChainforgeError 应能从 _internal 模块导入。"""
    from chainforge._internal import ChainforgeError

    assert ChainforgeError is not None


def test_invalid_parameter_is_value_error():
    from chainforge._internal import raise_invalid_parameter

    with pytest.raises(ValueError) as exc_info:
        raise_invalid_parameter("negative amount")
    assert "negative amount" in str(exc_info.value)


def test_serialization_is_value_error():
    from chainforge._internal import raise_serialization

    with pytest.raises(ValueError) as exc_info:
        raise_serialization("unexpected EOF")
    assert "unexpected EOF" in str(exc_info.value)


def test_crypto_is_runtime_error():
    from chainforge._internal import raise_crypto

    with pytest.raises(RuntimeError) as exc_info:
        raise_crypto("bad signature")
    assert "bad signature" in str(exc_info.value)


def test_storage_is_runtime_error():
    from chainforge._internal import raise_storage

    with pytest.raises(RuntimeError) as exc_info:
        raise_storage("disk full")
    assert "disk full" in str(exc_info.value)


def test_state_root_mismatch_contains_expected_and_actual():
    from chainforge._internal import raise_state_root_mismatch

    with pytest.raises(RuntimeError) as exc_info:
        raise_state_root_mismatch("0xabc", "0xdef")
    msg = str(exc_info.value)
    assert "expected 0xabc" in msg
    assert "got 0xdef" in msg
