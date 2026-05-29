"""测试 PyO3 密码学绑定。"""

import pytest


def test_secret_key_generation():
    from chainforge._internal import SecretKey
    sk = SecretKey()
    pk_bytes = sk.public_key()
    assert isinstance(pk_bytes, bytes)
    assert len(pk_bytes) == 33


def test_sign_and_verify():
    from chainforge._internal import SecretKey, PublicKey
    sk = SecretKey()
    pk = PublicKey.from_bytes(sk.public_key())
    msg = b"hello world"
    sig = sk.sign(msg)
    assert isinstance(sig, bytes)
    assert len(sig) == 65
    assert pk.verify(msg, sig)


def test_verify_rejects_wrong_message():
    from chainforge._internal import SecretKey, PublicKey
    sk = SecretKey()
    pk = PublicKey.from_bytes(sk.public_key())
    sig = sk.sign(b"correct message")
    assert not pk.verify(b"wrong message", sig)
