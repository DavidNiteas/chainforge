"""测试 PyO3 密码学绑定。"""

import pytest


def test_secret_key_generation():
    from kilnchain._internal import SecretKey

    sk = SecretKey()
    pk_bytes = sk.public_key()
    assert isinstance(pk_bytes, bytes)
    assert len(pk_bytes) == 33


def test_sign_and_verify():
    from kilnchain._internal import SecretKey, PublicKey

    sk = SecretKey()
    pk = PublicKey.from_bytes(sk.public_key())
    msg = b"hello world"
    sig = sk.sign(msg)
    assert isinstance(sig, bytes)
    assert len(sig) == 65
    assert pk.verify(msg, sig)


def test_verify_rejects_wrong_message():
    from kilnchain._internal import SecretKey, PublicKey

    sk = SecretKey()
    pk = PublicKey.from_bytes(sk.public_key())
    sig = sk.sign(b"correct message")
    assert not pk.verify(b"wrong message", sig)


def test_sha256():
    from kilnchain._internal import sha256

    result = sha256(b"hello")
    assert isinstance(result, bytes)
    assert len(result) == 32


def test_ripemd160():
    from kilnchain._internal import ripemd160

    result = ripemd160(b"hello")
    assert isinstance(result, bytes)
    assert len(result) == 20
