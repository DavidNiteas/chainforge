"""测试 PyO3 LightClient / MptProof 绑定。"""

import pytest


def test_light_client_genesis():
    from kilnchain._internal import BlockHeader, LightClient

    genesis = BlockHeader(
        parent_hash=b"\x00" * 32,
        number=0,
        timestamp=0,
        difficulty=0,
        nonce=0,
        extra_data=b"",
        state_root=b"\x00" * 32,
        txs_root=b"\x00" * 32,
    )
    lc = LightClient(genesis)
    assert lc.len() == 1
    assert lc.latest_header().number == 0


def test_light_client_add_header():
    from kilnchain._internal import BlockHeader, LightClient

    genesis = BlockHeader(
        parent_hash=b"\x00" * 32,
        number=0,
        timestamp=0,
        difficulty=0,
        nonce=0,
        extra_data=b"",
        state_root=b"\x00" * 32,
        txs_root=b"\x00" * 32,
    )
    lc = LightClient(genesis)
    h1 = BlockHeader(
        parent_hash=lc.latest_header().hash(),
        number=1,
        timestamp=0,
        difficulty=0,
        nonce=0,
        extra_data=b"",
        state_root=b"\x00" * 32,
        txs_root=b"\x00" * 32,
    )
    lc.add_header(h1)
    assert lc.len() == 2
    assert lc.get_header_by_number(1).number == 1


def test_light_client_rejects_bad_parent():
    from kilnchain._internal import BlockHeader, LightClient

    genesis = BlockHeader(
        parent_hash=b"\x00" * 32,
        number=0,
        timestamp=0,
        difficulty=0,
        nonce=0,
        extra_data=b"",
        state_root=b"\x00" * 32,
        txs_root=b"\x00" * 32,
    )
    lc = LightClient(genesis)
    bad = BlockHeader(
        parent_hash=b"\xff" * 32,
        number=1,
        timestamp=0,
        difficulty=0,
        nonce=0,
        extra_data=b"",
        state_root=b"\x00" * 32,
        txs_root=b"\x00" * 32,
    )
    with pytest.raises(ValueError):
        lc.add_header(bad)


def test_mpt_proof_verify():
    from kilnchain._internal import MptProof

    # MptProof 的构造和属性访问测试
    proof = MptProof(key=b"\xab", proof_nodes=[b"some_rlp_data"])
    assert proof.key == b"\xab"
    assert len(proof.proof_nodes) == 1
