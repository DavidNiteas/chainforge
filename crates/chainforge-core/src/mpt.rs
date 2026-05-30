//! Merkle Patricia Trie（MPT）证明验证。
//!
//! 轻客户端仅验证证明，不构建完整 Trie。

use chainforge_crypto::keccak256;
use chainforge_error::ChainforgeError;

use crate::rlp::RlpDecoder;

/// MPT 节点类型（验证视角）。
#[derive(Debug, Clone, PartialEq)]
enum MptNode {
    /// 空节点
    Null,
    /// 叶子节点：(path, value)
    Leaf(Vec<u8>, Vec<u8>),
    /// 扩展节点：(path, next_hash_or_data)
    Extension(Vec<u8>, Vec<u8>),
    /// 分支节点：16 个分支 + 可选 value
    Branch(Box<[Option<Vec<u8>>; 16]>, Option<Vec<u8>>),
}

/// 将 key 转为 nibble 数组（每字节拆成高/低 4 位）。
fn key_to_nibbles(key: &[u8]) -> Vec<u8> {
    let mut nibbles = Vec::with_capacity(key.len() * 2);
    for b in key {
        nibbles.push(b >> 4);
        nibbles.push(b & 0x0f);
    }
    nibbles
}

/// 解码路径前缀，返回 (is_leaf, 剩余 nibbles)。
fn decode_path_prefix(encoded: &[u8]) -> (bool, Vec<u8>) {
    if encoded.is_empty() {
        return (false, vec![]);
    }
    let prefix = encoded[0];
    let is_leaf = (prefix & 0x20) != 0;
    let is_odd = (prefix & 0x10) != 0;

    let mut nibbles = Vec::new();
    if is_odd {
        nibbles.push(prefix & 0x0f);
    }
    for b in &encoded[1..] {
        nibbles.push(b >> 4);
        nibbles.push(b & 0x0f);
    }
    (is_leaf, nibbles)
}

/// 从 RLP 编码解析 MPT 节点。
fn decode_node(data: &[u8]) -> Result<MptNode, ChainforgeError> {
    if data.is_empty() || data == [0x80] {
        return Ok(MptNode::Null);
    }

    let mut dec = RlpDecoder::new(data);
    let items: Vec<Vec<u8>> = dec.decode_list(|d| Ok(d.decode_bytes()?.to_vec()))?;

    match items.len() {
        2 => {
            let (is_leaf, path) = decode_path_prefix(&items[0]);
            if is_leaf {
                Ok(MptNode::Leaf(path, items[1].clone()))
            } else {
                Ok(MptNode::Extension(path, items[1].clone()))
            }
        }
        17 => {
            let mut branches: [Option<Vec<u8>>; 16] = Default::default();
            for i in 0..16 {
                branches[i] = if items[i].is_empty() {
                    None
                } else {
                    Some(items[i].clone())
                };
            }
            let value = if items[16].is_empty() {
                None
            } else {
                Some(items[16].clone())
            };
            Ok(MptNode::Branch(Box::new(branches), value))
        }
        n => Err(ChainforgeError::Serialization(format!(
            "invalid MPT node item count: {}",
            n
        ))),
    }
}

/// 计算节点哈希（RLP 长度 < 32 时返回 RLP 本身，否则 Keccak-256）。
fn node_hash(data: &[u8]) -> [u8; 32] {
    keccak256(data)
}

/// 验证 MPT 证明。
///
/// # 参数
/// - `root`: 状态根哈希
/// - `key`: 查询键（如账户地址）
/// - `proof_nodes`: 从根到叶子的 RLP 编码节点列表
///
/// # 返回
/// 若证明有效，返回该键对应的值；否则返回 `None`。
pub fn verify_proof(
    root: &[u8; 32],
    key: &[u8],
    proof_nodes: &[Vec<u8>],
) -> Result<Option<Vec<u8>>, ChainforgeError> {
    if proof_nodes.is_empty() {
        return Ok(None);
    }

    let mut expected_hash = *root;
    let nibbles = key_to_nibbles(key);
    let mut path_idx = 0usize;

    for node_data in proof_nodes {
        // 验证当前节点哈希是否与期望值匹配
        let actual_hash = node_hash(node_data);
        if actual_hash != expected_hash {
            return Ok(None);
        }

        let node = decode_node(node_data)?;
        match node {
            MptNode::Null => return Ok(None),
            MptNode::Leaf(path, value) => {
                if path_idx + path.len() == nibbles.len()
                    && nibbles[path_idx..] == path[..]
                {
                    return Ok(Some(value));
                } else {
                    return Ok(None);
                }
            }
            MptNode::Extension(path, next) => {
                if nibbles[path_idx..].starts_with(&path) {
                    path_idx += path.len();
                    expected_hash = if next.len() == 32 {
                        let mut arr = [0u8; 32];
                        arr.copy_from_slice(&next);
                        arr
                    } else {
                        node_hash(&next)
                    };
                } else {
                    return Ok(None);
                }
            }
            MptNode::Branch(branches, value) => {
                if path_idx >= nibbles.len() {
                    // 路径结束，返回分支节点的 value
                    return Ok(value);
                }
                let nibble = nibbles[path_idx] as usize;
                path_idx += 1;
                match &branches[nibble] {
                    Some(next) => {
                        expected_hash = if next.len() == 32 {
                            let mut arr = [0u8; 32];
                            arr.copy_from_slice(next);
                            arr
                        } else {
                            node_hash(next)
                        };
                    }
                    None => return Ok(None),
                }
            }
        }
    }

    Ok(None)
}

/// MPT 证明验证的轻量封装。
#[derive(Debug, Clone, PartialEq)]
pub struct MptProof {
    pub key: Vec<u8>,
    pub proof_nodes: Vec<Vec<u8>>,
}

impl MptProof {
    /// 验证证明并返回值。
    pub fn verify(&self, root: &[u8; 32]) -> Result<Option<Vec<u8>>, ChainforgeError> {
        verify_proof(root, &self.key, &self.proof_nodes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rlp::RlpEncoder;

    #[test]
    fn test_decode_path_prefix_leaf_even() {
        // 叶子，偶数长度 2: 前缀 0x20 + 0xab
        let encoded = vec![0x20, 0xab];
        let (is_leaf, path) = decode_path_prefix(&encoded);
        assert!(is_leaf);
        assert_eq!(path, vec![0xa, 0xb]);
    }

    #[test]
    fn test_decode_path_prefix_leaf_odd() {
        // 叶子，奇数长度 1: 前缀 0x3a（0x30 | 0x0a）
        let encoded = vec![0x3a];
        let (is_leaf, path) = decode_path_prefix(&encoded);
        assert!(is_leaf);
        assert_eq!(path, vec![0xa]);
    }

    #[test]
    fn test_decode_path_prefix_extension_even() {
        // 扩展，偶数长度 2: 前缀 0x00 + 0xab
        let encoded = vec![0x00, 0xab];
        let (is_leaf, path) = decode_path_prefix(&encoded);
        assert!(!is_leaf);
        assert_eq!(path, vec![0xa, 0xb]);
    }

    #[test]
    fn test_decode_path_prefix_extension_odd() {
        // 扩展，奇数长度 1: 前缀 0x1a（0x10 | 0x0a）
        let encoded = vec![0x1a];
        let (is_leaf, path) = decode_path_prefix(&encoded);
        assert!(!is_leaf);
        assert_eq!(path, vec![0xa]);
    }

    #[test]
    fn test_key_to_nibbles() {
        assert_eq!(key_to_nibbles(&[0xab, 0xcd]), vec![0xa, 0xb, 0xc, 0xd]);
    }

    #[test]
    fn test_verify_leaf_only_proof() {
        // 构建一个最简单的 MPT：只有一个叶子节点
        // Leaf([0xa, 0xb], b"hello") → even leaf → [0x20, 0xab]
        let mut enc = RlpEncoder::new();
        enc.encode_list(|e| {
            e.encode_bytes(&[0x20, 0xab]);
            e.encode_bytes(b"hello");
        });
        let leaf_rlp = enc.finish();
        let root = node_hash(&leaf_rlp);

        let key = [0xab];
        let proof = vec![leaf_rlp];
        let result = verify_proof(&root, &key, &proof).unwrap();
        assert_eq!(result, Some(b"hello".to_vec()));
    }

    #[test]
    fn test_verify_leaf_wrong_key() {
        let mut enc = RlpEncoder::new();
        enc.encode_list(|e| {
            e.encode_bytes(&[0x20, 0xab]);
            e.encode_bytes(b"hello");
        });
        let leaf_rlp = enc.finish();
        let root = node_hash(&leaf_rlp);

        let key = [0xac]; // wrong key
        let proof = vec![leaf_rlp];
        let result = verify_proof(&root, &key, &proof).unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_verify_extension_then_leaf() {
        // Extension(path=[0xa], next=leaf_hash) -> Leaf(path=[0xb], value="world")
        let mut leaf_enc = RlpEncoder::new();
        leaf_enc.encode_list(|e| {
            e.encode_bytes(&[0x3b]); // leaf, odd, path=[b]
            e.encode_bytes(b"world");
        });
        let leaf_rlp = leaf_enc.finish();
        let leaf_hash = node_hash(&leaf_rlp);

        let mut ext_enc = RlpEncoder::new();
        ext_enc.encode_list(|e| {
            e.encode_bytes(&[0x1a]); // extension, odd, path=[a]
            e.encode_bytes(&leaf_hash);
        });
        let ext_rlp = ext_enc.finish();
        let root = node_hash(&ext_rlp);

        let key = [0xab];
        let proof = vec![ext_rlp, leaf_rlp];
        let result = verify_proof(&root, &key, &proof).unwrap();
        assert_eq!(result, Some(b"world".to_vec()));
    }

    #[test]
    fn test_verify_branch() {
        // 两个 key: [0x05] 和 [0x15]
        // Leaf for [0x05]: path=[5], odd leaf → 0x35
        let mut leaf0_enc = RlpEncoder::new();
        leaf0_enc.encode_list(|e| {
            e.encode_bytes(&[0x35]); // leaf, odd, path=[5]
            e.encode_bytes(b"val0");
        });
        let leaf0_rlp = leaf0_enc.finish();
        let leaf0_hash = node_hash(&leaf0_rlp);

        // Leaf for [0x15]: path=[5], odd leaf → 0x35
        let mut leaf1_enc = RlpEncoder::new();
        leaf1_enc.encode_list(|e| {
            e.encode_bytes(&[0x35]);
            e.encode_bytes(b"val1");
        });
        let leaf1_rlp = leaf1_enc.finish();
        let leaf1_hash = node_hash(&leaf1_rlp);

        let mut branch_enc = RlpEncoder::new();
        branch_enc.encode_list(|e| {
            e.encode_bytes(&leaf0_hash); // branch[0]
            e.encode_bytes(&leaf1_hash); // branch[1]
            for _ in 2..16 {
                e.encode_bytes(&[]);
            }
            e.encode_bytes(&[]); // no branch value
        });
        let branch_rlp = branch_enc.finish();
        let root = node_hash(&branch_rlp);

        let key = [0x05];
        let proof = vec![branch_rlp.clone(), leaf0_rlp];
        let result = verify_proof(&root, &key, &proof).unwrap();
        assert_eq!(result, Some(b"val0".to_vec()));

        // 验证另一个 key
        let key1 = [0x15];
        let proof1 = vec![branch_rlp, leaf1_rlp];
        let result1 = verify_proof(&root, &key1, &proof1).unwrap();
        assert_eq!(result1, Some(b"val1".to_vec()));
    }

    #[test]
    fn test_verify_proof_struct() {
        let mut enc = RlpEncoder::new();
        enc.encode_list(|e| {
            e.encode_bytes(&[0x20, 0xab]);
            e.encode_bytes(b"stored");
        });
        let leaf_rlp = enc.finish();
        let root = node_hash(&leaf_rlp);

        let proof = MptProof {
            key: vec![0xab],
            proof_nodes: vec![leaf_rlp],
        };
        assert_eq!(proof.verify(&root).unwrap(), Some(b"stored".to_vec()));
    }
}
