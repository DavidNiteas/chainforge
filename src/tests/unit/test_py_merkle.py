"""测试 PyO3 MerkleTree 绑定。"""

import pytest


def test_merkle_tree_root_returns_bytes():
    from kilnchain._internal import MerkleTree

    leaves = [b"a", b"b", b"c"]
    tree = MerkleTree(leaves)
    root = tree.root()
    assert isinstance(root, bytes)
    assert len(root) == 32


def test_merkle_tree_proof_and_verify():
    from kilnchain._internal import MerkleTree, keccak256

    leaves = [b"a", b"b", b"c", b"d"]
    tree = MerkleTree(leaves)
    root = tree.root()

    for i, leaf in enumerate(leaves):
        proof = tree.proof(i)
        assert proof is not None
        assert "siblings" in proof
        assert "indices" in proof
        leaf_hash = keccak256(leaf)
        assert MerkleTree.verify(root, leaf_hash, proof)
