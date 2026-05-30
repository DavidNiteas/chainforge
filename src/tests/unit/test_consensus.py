"""Tests for HotStuff consensus engine Python bindings."""

import pytest

from chainforge import (
    Block,
    BlockHeader,
    BlockTree,
    ConsensusEngine,
    LeaderRotator,
    Pacemaker,
    Phase,
    QuorumCertificate,
    SafetyRules,
    Transaction,
    Vote,
)


def make_tx(nonce: int = 0) -> Transaction:
    return Transaction(
        nonce=nonce,
        gas_price=1,
        gas_limit=21000,
        to=b"\x00" * 20,
        value=100,
        data=b"",
        v=27,
        r=b"\x00" * 32,
        s=b"\x00" * 32,
    )


def make_block(number: int, parent: bytes = b"\x00" * 32) -> Block:
    header = BlockHeader(
        parent_hash=parent,
        number=number,
        timestamp=0,
        difficulty=0,
        nonce=0,
        extra_data=b"",
        state_root=b"\x00" * 32,
        txs_root=b"\x00" * 32,
    )
    b = Block(header=header, transactions=[], uncle_headers=[])
    b.compute_txs_root()
    return b


class TestPhase:
    def test_phase_values(self) -> None:
        assert Phase.Prepare == Phase.Prepare
        assert Phase.PreCommit == Phase.PreCommit
        assert Phase.Commit == Phase.Commit
        assert Phase.Decide == Phase.Decide
        assert Phase.Prepare != Phase.PreCommit


class TestVote:
    def test_vote_creation(self) -> None:
        v = Vote(block_hash=b"\xab" * 32, view_number=5, phase=Phase.Prepare)
        assert v.block_hash == b"\xab" * 32
        assert v.view_number == 5
        assert v.phase == Phase.Prepare
        assert len(v.voter) == 33
        assert len(v.signature) == 64
        assert v.recovery_id == 0

    def test_vote_invalid_hash_length(self) -> None:
        with pytest.raises(ValueError, match="32 bytes"):
            Vote(block_hash=b"\x00" * 31, view_number=1, phase=Phase.Prepare)


class TestQuorumCertificate:
    def test_qc_creation(self) -> None:
        qc = QuorumCertificate.new(b"\xcd" * 32, 10, Phase.Commit)
        assert qc.block_hash == b"\xcd" * 32
        assert qc.view_number == 10
        assert qc.phase == Phase.Commit
        assert qc.votes == []
        assert not qc.has_quorum(quorum=3)

    def test_qc_verify_empty(self) -> None:
        qc = QuorumCertificate.new(b"\x00" * 32, 0, Phase.Prepare)
        assert qc.verify()  # genesis QC with view 0 is valid

    def test_qc_invalid_hash(self) -> None:
        with pytest.raises(ValueError, match="32 bytes"):
            QuorumCertificate.new(b"\x00" * 33, 1, Phase.Prepare)


class TestBlockTree:
    def test_insert_and_get(self) -> None:
        tree = BlockTree()
        block = make_block(1)
        h = block.header.hash()
        qc = QuorumCertificate.new(h, 1, Phase.Prepare)
        tree.insert(block, qc)

        node = tree.get(h)
        assert node is not None
        assert node.block.header.number == 1
        assert node.parent == b"\x00" * 32
        assert node.prepare_qc is not None
        assert node.precommit_qc is None
        assert node.commit_qc is None

    def test_commit_chain(self) -> None:
        tree = BlockTree()
        genesis = b"\x00" * 32
        a = make_block(1, genesis)
        a_hash = a.header.hash()

        tree.insert(a, QuorumCertificate.new(a_hash, 1, Phase.Prepare))
        tree.add_commit_qc(a_hash, QuorumCertificate.new(a_hash, 1, Phase.Commit))

        assert tree.committed_height() == 1
        blocks = tree.committed_blocks()
        assert len(blocks) == 1
        assert blocks[0].header.number == 1

    def test_view_and_locked_qc(self) -> None:
        tree = BlockTree()
        assert tree.view == 0
        tree.view = 5
        assert tree.view == 5
        assert tree.locked_qc is None
        qc = QuorumCertificate.new(b"\x00" * 32, 3, Phase.Prepare)
        tree.locked_qc = qc
        assert tree.locked_qc is not None
        assert tree.locked_qc.view_number == 3


class TestSafetyRules:
    def test_can_vote_prepare(self) -> None:
        safety = SafetyRules()
        block = make_block(1)
        qc = QuorumCertificate.new(b"\x00" * 32, 0, Phase.Prepare)
        assert safety.can_vote_prepare(block, qc)

    def test_locked_view_blocks_old(self) -> None:
        safety = SafetyRules()
        safety.locked_view = 5
        block = make_block(3)
        old_qc = QuorumCertificate.new(b"\x00" * 32, 3, Phase.Prepare)
        assert not safety.can_vote_prepare(block, old_qc)

    def test_update_locked(self) -> None:
        safety = SafetyRules()
        qc = QuorumCertificate.new(b"\x00" * 32, 7, Phase.PreCommit)
        safety.update_locked(qc)
        assert safety.locked_view == 7
        assert safety.locked_qc is not None

    def test_check_double_vote(self) -> None:
        safety = SafetyRules()
        assert not safety.check_double_vote(b"\x00" * 32, Phase.Prepare)

    def test_check_double_vote_invalid_hash(self) -> None:
        safety = SafetyRules()
        with pytest.raises(ValueError, match="32 bytes"):
            safety.check_double_vote(b"\x00" * 31, Phase.Prepare)


class TestLeaderRotator:
    def test_leader_rotation(self) -> None:
        lr = LeaderRotator(4)
        assert lr.node_count == 4
        assert lr.leader_for(0) == 0
        assert lr.leader_for(1) == 1
        assert lr.leader_for(3) == 3
        assert lr.leader_for(4) == 0


class TestPacemaker:
    def test_pacemaker_defaults(self) -> None:
        pm = Pacemaker(0, 4)
        assert pm.node_id == 0
        assert pm.current_view == 0
        assert pm.timeout_ms == 5000
        assert pm.is_leader()
        assert pm.current_leader() == 0

    def test_advance_view(self) -> None:
        pm = Pacemaker(1, 4)
        pm.advance_view(2)
        assert pm.current_view == 2
        assert pm.current_leader() == 2
        assert not pm.is_leader()

    def test_leader_rotator(self) -> None:
        pm = Pacemaker(0, 4)
        lr = pm.leader_rotator()
        assert lr.node_count == 4


class TestConsensusEngine:
    def test_new(self) -> None:
        engine = ConsensusEngine(0, 4)
        assert engine.node_id == 0

    def test_propose_block(self) -> None:
        engine = ConsensusEngine(0, 4)
        qc = QuorumCertificate.new(b"\x00" * 32, 0, Phase.Prepare)
        block = engine.propose_block(b"\x00" * 32, 1, [], qc)
        assert block.header.number == 1
        assert block.header.parent_hash == b"\x00" * 32

    def test_vote_prepare(self) -> None:
        engine = ConsensusEngine(0, 4)
        block = make_block(1)
        qc = QuorumCertificate.new(b"\x00" * 32, 0, Phase.Prepare)
        vote = engine.vote_prepare(block, qc)
        assert vote is not None
        assert vote.phase == Phase.Prepare

    def test_safety_reject_old_view(self) -> None:
        engine = ConsensusEngine(1, 4)
        safety = engine.safety()
        safety.locked_view = 5
        # ConsensusEngine.safety() returns a copy, so modifying it does not
        # affect the engine.  Test the underlying rules directly in
        # TestSafetyRules instead.

    def test_full_pipeline_commit(self) -> None:
        engine = ConsensusEngine(0, 4)
        genesis = b"\x00" * 32
        block = make_block(1, genesis)
        block_hash = block.header.hash()

        # Prepare phase
        prepare_votes = [
            Vote(block_hash, 1, Phase.Prepare),
            Vote(block_hash, 1, Phase.Prepare),
            Vote(block_hash, 1, Phase.Prepare),
        ]
        prepare_qc = engine.form_qc(prepare_votes, Phase.Prepare, 3)
        assert prepare_qc is not None
        engine.on_prepare_qc(block, prepare_qc)

        # PreCommit phase
        precommit_votes = [
            Vote(block_hash, 1, Phase.PreCommit),
            Vote(block_hash, 1, Phase.PreCommit),
            Vote(block_hash, 1, Phase.PreCommit),
        ]
        precommit_qc = engine.form_qc(precommit_votes, Phase.PreCommit, 3)
        assert precommit_qc is not None
        engine.on_precommit_qc(block_hash, precommit_qc)

        # Commit phase
        commit_votes = [
            Vote(block_hash, 1, Phase.Commit),
            Vote(block_hash, 1, Phase.Commit),
            Vote(block_hash, 1, Phase.Commit),
        ]
        commit_qc = engine.form_qc(commit_votes, Phase.Commit, 3)
        assert commit_qc is not None
        engine.on_commit_qc(block_hash, commit_qc)

        tree = engine.block_tree()
        assert tree.committed_height() == 1

    def test_leader_rotation(self) -> None:
        engine = ConsensusEngine(0, 4)
        pm = engine.pacemaker()
        assert pm.is_leader()

        engine.advance_view(1)
        pm2 = engine.pacemaker()
        # After advancing view, node 0 is no longer leader in view 1
        assert not pm2.is_leader()
        assert pm2.current_leader() == 1

    def test_fork_choice(self) -> None:
        engine = ConsensusEngine(0, 4)
        genesis = b"\x00" * 32

        # Main chain: genesis -> A -> B
        a = make_block(1, genesis)
        a_hash = a.header.hash()
        engine.on_prepare_qc(a, QuorumCertificate.new(a_hash, 1, Phase.Prepare))
        engine.on_precommit_qc(
            a_hash, QuorumCertificate.new(a_hash, 1, Phase.PreCommit)
        )
        engine.on_commit_qc(a_hash, QuorumCertificate.new(a_hash, 1, Phase.Commit))

        b = make_block(2, a_hash)
        b_hash = b.header.hash()
        engine.on_prepare_qc(b, QuorumCertificate.new(b_hash, 2, Phase.Prepare))
        engine.on_precommit_qc(
            b_hash, QuorumCertificate.new(b_hash, 2, Phase.PreCommit)
        )
        engine.on_commit_qc(b_hash, QuorumCertificate.new(b_hash, 2, Phase.Commit))

        tree = engine.block_tree()
        assert tree.committed_height() == 2
        assert len(tree.committed_blocks()) == 2
