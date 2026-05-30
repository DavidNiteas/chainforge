"""端到端集成测试：完整业务链路。"""

import pytest

from chainforge import (
    BlockHeader,
    MerkleTree,
    PublicKey,
    SecretKey,
    Transaction,
)


def test_transaction_sign_verify_roundtrip():
    sk = SecretKey()
    pk = PublicKey.from_bytes(sk.public_key())
    msg = b"hello chainforge"
    sig = sk.sign(msg)
    assert pk.verify(msg, sig)


def test_transaction_merkle_tree_roundtrip():
    leaves = [b"leaf1", b"leaf2", b"leaf3"]
    tree = MerkleTree(leaves)
    root = tree.root()
    assert isinstance(root, bytes)
    assert len(root) == 32

    from chainforge._internal import keccak256

    proof = tree.proof(0)
    leaf_hash = keccak256(leaves[0])
    assert MerkleTree.verify(root, leaf_hash, proof)


def test_blockheader_hash():
    header = BlockHeader(
        parent_hash=b"\x00" * 32,
        number=1,
        timestamp=1234567890,
        difficulty=1000,
        nonce=0,
        extra_data=b"",
        state_root=b"\x00" * 32,
        txs_root=b"\x00" * 32,
    )
    h = header.hash()
    assert isinstance(h, bytes)
    assert len(h) == 32
