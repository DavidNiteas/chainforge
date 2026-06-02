"""Tests for P2P network layer Python bindings."""

import pytest

from kilnchain import (
    Message,
    Node,
    NodeConfig,
    PeerId,
    PeerInfo,
    RoutingTable,
)


class TestPeerId:
    def test_peer_id_creation(self) -> None:
        pk = b"\xab" * 32
        pid = PeerId(pk)
        assert pid.to_bytes() == pk

    def test_peer_id_from_public_key(self) -> None:
        pk = b"\xcd" * 32
        pid = PeerId.from_public_key(pk)
        assert len(pid.to_bytes()) == 32

    def test_peer_id_invalid_length(self) -> None:
        with pytest.raises(ValueError, match="32 bytes"):
            PeerId(b"\x00" * 31)


class TestPeerInfo:
    def test_peer_info_creation(self) -> None:
        pid = PeerId(b"\x00" * 32)
        info = PeerInfo(pid, "127.0.0.1:30303")
        assert info.addr == "127.0.0.1:30303"
        assert info.id.to_bytes() == b"\x00" * 32

    def test_peer_info_invalid_addr(self) -> None:
        pid = PeerId(b"\x00" * 32)
        with pytest.raises(ValueError):
            PeerInfo(pid, "not-an-address")


class TestMessage:
    def test_message_ping(self) -> None:
        msg = Message.ping()
        assert msg.is_ping
        assert not msg.is_pong
        assert not msg.is_transaction
        assert not msg.is_block

    def test_message_pong(self) -> None:
        msg = Message.pong()
        assert msg.is_pong
        assert not msg.is_ping

    def test_message_transaction(self) -> None:
        msg = Message.transaction(b"\x01\x02\x03")
        assert msg.is_transaction
        assert not msg.is_ping

    def test_message_block(self) -> None:
        msg = Message.block(b"\x04\x05\x06")
        assert msg.is_block

    def test_message_encode_decode(self) -> None:
        msg = Message.transaction(b"hello")
        encoded = msg.encode()
        assert isinstance(encoded, bytes)
        decoded = Message.decode(encoded)
        assert decoded.is_transaction

    def test_message_decode_transaction(self) -> None:
        from kilnchain import Transaction

        tx = Transaction(
            nonce=1,
            gas_price=1,
            gas_limit=21000,
            to=b"\x00" * 20,
            value=100,
            data=b"",
        )
        tx_bytes = tx.encode_rlp()
        msg = Message.transaction(tx_bytes)
        decoded_tx = msg.decode_transaction()
        assert decoded_tx is not None
        assert decoded_tx.nonce == 1

    def test_message_decode_transaction_on_non_tx(self) -> None:
        msg = Message.ping()
        assert msg.decode_transaction() is None


class TestRoutingTable:
    def test_routing_table_basic(self) -> None:
        local = PeerId(b"\x00" * 32)
        table = RoutingTable(local)
        assert table.len() == 0
        assert table.is_empty()

    def test_routing_table_update_and_find(self) -> None:
        local = PeerId(b"\x00" * 32)
        table = RoutingTable(local)

        peer = PeerInfo(PeerId(b"\x01" * 32), "127.0.0.1:30303")
        table.update(peer)
        assert table.len() == 1

        closest = table.find_closest(PeerId(b"\x01" * 32), 1)
        assert len(closest) == 1
        assert closest[0].addr == "127.0.0.1:30303"


class TestNodeConfig:
    def test_node_config_defaults(self) -> None:
        config = NodeConfig(b"\x00" * 32)
        assert config.gossip_fanout == 3
        assert config.gossip_ttl_secs == 60
        assert len(config.local_id.to_bytes()) == 32

    def test_node_config_setters(self) -> None:
        config = NodeConfig(b"\x00" * 32)
        config.gossip_fanout = 5
        config.gossip_ttl_secs = 120
        assert config.gossip_fanout == 5
        assert config.gossip_ttl_secs == 120


@pytest.mark.asyncio
class TestNode:
    async def test_node_handle_ping(self) -> None:
        config = NodeConfig(b"\x00" * 32)
        node = Node(config)
        result = await node.handle_message(Message.ping())
        assert len(result) == 1
        assert result[0].is_pong

    async def test_node_gossip_dedup(self) -> None:
        config = NodeConfig(b"\x00" * 32)
        node = Node(config)

        # Pre-populate routing table via internal method
        rt = await node.routing_table()
        for i in range(2, 10):
            rt.update(PeerInfo(PeerId(bytes([0] * 31 + [i])), f"127.0.0.1:{1000 + i}"))

        msg = Message.transaction(b"\x01\x02\x03")

        # First time: should forward
        forward1 = await node.handle_message(msg)
        assert len(forward1) == 1

        # Second time: deduplicated
        forward2 = await node.handle_message(msg)
        assert len(forward2) == 0

    async def test_node_drain_inbox(self) -> None:
        config = NodeConfig(b"\x00" * 32)
        node = Node(config)

        # Empty inbox
        msgs = await node.drain_inbox(100)
        assert msgs == []

    async def test_node_routing_table(self) -> None:
        config = NodeConfig(b"\x00" * 32)
        node = Node(config)
        rt = await node.routing_table()
        assert rt.is_empty()
