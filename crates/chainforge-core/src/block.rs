use crate::merkle::MerkleTree;
use crate::rlp::{RlpDecoder, RlpEncoder};
use crate::tx::Transaction;
use chainforge_crypto::keccak256;
use chainforge_error::ChainforgeError;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BlockHeader {
    pub parent_hash: [u8; 32],
    pub number: u64,
    pub timestamp: u64,
    pub difficulty: u64,
    pub nonce: u64,
    pub extra_data: Vec<u8>,
    pub state_root: [u8; 32],
    pub txs_root: [u8; 32],
}

impl BlockHeader {
    pub fn encode_rlp(&self) -> Vec<u8> {
        let mut enc = RlpEncoder::new();
        enc.encode_list(|e| {
            e.encode_bytes(&self.parent_hash);
            e.encode_u64(self.number);
            e.encode_u64(self.timestamp);
            e.encode_u64(self.difficulty);
            e.encode_u64(self.nonce);
            e.encode_bytes(&self.extra_data);
            e.encode_bytes(&self.state_root);
            e.encode_bytes(&self.txs_root);
        });
        enc.finish()
    }

    pub fn decode_rlp(data: &[u8]) -> Result<Self, ChainforgeError> {
        let mut dec = RlpDecoder::new(data);
        let items: Vec<Vec<u8>> = dec.decode_list(|d| Ok(d.decode_bytes()?.to_vec()))?;
        if items.len() != 8 {
            return Err(ChainforgeError::Serialization(format!(
                "expected 8 RLP items for BlockHeader, got {}",
                items.len()
            )));
        }

        if items[5].len() > 32 {
            return Err(ChainforgeError::InvalidParameter(
                "extra_data exceeds 32 bytes".to_string(),
            ));
        }

        let mut parent_hash = [0u8; 32];
        parent_hash.copy_from_slice(&items[0]);
        let mut state_root = [0u8; 32];
        state_root.copy_from_slice(&items[6]);
        let mut txs_root = [0u8; 32];
        txs_root.copy_from_slice(&items[7]);

        fn bytes_to_u64(bytes: &[u8]) -> u64 {
            if bytes.is_empty() {
                0
            } else {
                let mut arr = [0u8; 8];
                arr[8 - bytes.len()..].copy_from_slice(bytes);
                u64::from_be_bytes(arr)
            }
        }

        Ok(BlockHeader {
            parent_hash,
            number: bytes_to_u64(&items[1]),
            timestamp: bytes_to_u64(&items[2]),
            difficulty: bytes_to_u64(&items[3]),
            nonce: bytes_to_u64(&items[4]),
            extra_data: items[5].clone(),
            state_root,
            txs_root,
        })
    }

    pub fn hash(&self) -> [u8; 32] {
        keccak256(&self.encode_rlp())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Block {
    pub header: BlockHeader,
    pub transactions: Vec<Transaction>,
    pub uncle_headers: Vec<BlockHeader>,
}

impl Block {
    /// 计算交易列表的 Merkle 树根并赋值给 header.txs_root
    pub fn compute_txs_root(&mut self) {
        let leaves: Vec<[u8; 32]> = self.transactions.iter().map(|tx| tx.hash()).collect();
        self.header.txs_root = MerkleTree::new(leaves).root();
    }

    /// RLP 编码区块
    pub fn encode_rlp(&self) -> Vec<u8> {
        use crate::rlp::RlpEncoder;
        let mut enc = RlpEncoder::new();
        enc.encode_list(|e| {
            e.encode_bytes(&self.header.encode_rlp());
            e.encode_list(|e2| {
                for tx in &self.transactions {
                    e2.encode_bytes(&tx.encode_rlp());
                }
            });
            e.encode_list(|e2| {
                for uncle in &self.uncle_headers {
                    e2.encode_bytes(&uncle.encode_rlp());
                }
            });
        });
        enc.finish()
    }

    /// RLP 解码区块
    pub fn decode_rlp(data: &[u8]) -> Result<Self, ChainforgeError> {
        use crate::rlp::RlpDecoder;
        let mut dec = RlpDecoder::new(data);
        let items: Vec<Vec<u8>> = dec.decode_list(|d| Ok(d.decode_bytes()?.to_vec()))?;
        if items.len() != 3 {
            return Err(ChainforgeError::Serialization(format!(
                "expected 3 RLP items for Block, got {}",
                items.len()
            )));
        }
        let header = BlockHeader::decode_rlp(&items[0])?;

        let mut tx_dec = RlpDecoder::new(&items[1]);
        let tx_bytes: Vec<Vec<u8>> = tx_dec.decode_list(|d| Ok(d.decode_bytes()?.to_vec()))?;
        let transactions: Vec<Transaction> = tx_bytes
            .iter()
            .map(|b| Transaction::decode_rlp(b))
            .collect::<Result<Vec<_>, _>>()?;

        let mut uncle_dec = RlpDecoder::new(&items[2]);
        let uncle_bytes: Vec<Vec<u8>> = uncle_dec.decode_list(|d| Ok(d.decode_bytes()?.to_vec()))?;
        let uncle_headers: Vec<BlockHeader> = uncle_bytes
            .iter()
            .map(|b| BlockHeader::decode_rlp(b))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Block {
            header,
            transactions,
            uncle_headers,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blockheader_rlp_roundtrip() {
        let header = BlockHeader {
            parent_hash: [1u8; 32],
            number: 1,
            timestamp: 1234567890,
            difficulty: 1000,
            nonce: 0,
            extra_data: vec![0xde, 0xad],
            state_root: [2u8; 32],
            txs_root: [3u8; 32],
        };
        let encoded = header.encode_rlp();
        let decoded = BlockHeader::decode_rlp(&encoded).unwrap();
        assert_eq!(header, decoded);
    }

    #[test]
    fn test_extra_data_too_long() {
        let header = BlockHeader {
            parent_hash: [0u8; 32],
            number: 0,
            timestamp: 0,
            difficulty: 0,
            nonce: 0,
            extra_data: vec![0u8; 33],
            state_root: [0u8; 32],
            txs_root: [0u8; 32],
        };
        let encoded = header.encode_rlp();
        let result = BlockHeader::decode_rlp(&encoded);
        assert!(matches!(result, Err(ChainforgeError::InvalidParameter(_))));
    }

    #[test]
    fn test_block_txs_root() {
        let tx1 = Transaction {
            nonce: 0,
            gas_price: 1,
            gas_limit: 21000,
            to: Some([0xabu8; 20]),
            value: 100,
            data: vec![],
            v: 27,
            r: [0u8; 32],
            s: [0u8; 32],
        };
        let tx2 = Transaction {
            nonce: 1,
            gas_price: 1,
            gas_limit: 21000,
            to: Some([0xcbu8; 20]),
            value: 200,
            data: vec![],
            v: 28,
            r: [1u8; 32],
            s: [1u8; 32],
        };
        let mut block = Block {
            header: BlockHeader {
                parent_hash: [0u8; 32],
                number: 1,
                timestamp: 0,
                difficulty: 0,
                nonce: 0,
                extra_data: vec![],
                state_root: [0u8; 32],
                txs_root: [0u8; 32],
            },
            transactions: vec![tx1.clone(), tx2.clone()],
            uncle_headers: vec![],
        };
        block.compute_txs_root();
        let expected_root = MerkleTree::new(vec![tx1.hash(), tx2.hash()]).root();
        assert_eq!(block.header.txs_root, expected_root);
    }
}
