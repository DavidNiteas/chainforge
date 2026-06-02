use kilnchain_crypto::sha256;

/// 空 Merkle 树根常量
pub const EMPTY_ROOT: [u8; 32] = [0u8; 32];

/// Merkle 证明
#[derive(Debug, Clone, PartialEq)]
pub struct MerkleProof {
    pub siblings: Vec<[u8; 32]>,
    /// true = 当前节点是右子节点，false = 左子节点
    pub indices: Vec<bool>,
}

/// 二叉 SHA-256 Merkle 树
#[derive(Debug, Clone, PartialEq)]
pub struct MerkleTree {
    leaves: Vec<[u8; 32]>,
    layers: Vec<Vec<[u8; 32]>>,
}

impl MerkleTree {
    /// 从叶子哈希构建 Merkle 树
    pub fn new(leaves: Vec<[u8; 32]>) -> Self {
        if leaves.is_empty() {
            return MerkleTree {
                leaves: vec![],
                layers: vec![],
            };
        }

        let mut layers = vec![leaves.clone()];
        let mut current = leaves.clone();

        while current.len() > 1 {
            // 奇数时复制最后一个叶子
            if current.len() % 2 == 1 {
                current.push(current[current.len() - 1]);
            }
            let mut next = Vec::with_capacity(current.len() / 2);
            for pair in current.chunks_exact(2) {
                let mut concat = [0u8; 64];
                concat[..32].copy_from_slice(&pair[0]);
                concat[32..].copy_from_slice(&pair[1]);
                next.push(sha256(&concat));
            }
            layers.push(next.clone());
            current = next;
        }

        MerkleTree { leaves, layers }
    }

    /// 返回根哈希；空树返回 EMPTY_ROOT
    pub fn root(&self) -> [u8; 32] {
        if self.layers.is_empty() {
            return EMPTY_ROOT;
        }
        self.layers.last().unwrap()[0]
    }

    /// 为指定索引的叶子生成证明
    pub fn proof(&self, index: usize) -> Option<MerkleProof> {
        if index >= self.leaves.len() {
            return None;
        }

        let mut siblings = Vec::new();
        let mut indices = Vec::new();
        let mut idx = index;

        for layer in &self.layers {
            if layer.len() <= 1 {
                break;
            }
            // 确保当前层是偶数长度
            let len = layer.len();
            let _effective_len = if len % 2 == 1 { len + 1 } else { len };
            let is_right = idx % 2 == 1;
            indices.push(is_right);

            let sibling_idx = if is_right { idx - 1 } else { idx + 1 };
            if sibling_idx < len {
                siblings.push(layer[sibling_idx]);
            } else {
                // 奇数情况，sibling 是复制的最后一个
                siblings.push(layer[len - 1]);
            }

            idx /= 2;
        }

        Some(MerkleProof { siblings, indices })
    }

    /// 验证 Merkle 证明
    pub fn verify(root: &[u8; 32], leaf: &[u8; 32], proof: &MerkleProof) -> bool {
        let mut current = *leaf;
        for (sibling, is_right) in proof.siblings.iter().zip(proof.indices.iter()) {
            let mut concat = [0u8; 64];
            if *is_right {
                concat[..32].copy_from_slice(sibling);
                concat[32..].copy_from_slice(&current);
            } else {
                concat[..32].copy_from_slice(&current);
                concat[32..].copy_from_slice(sibling);
            }
            current = sha256(&concat);
        }
        &current == root
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn test_empty_merkle_root() {
        let tree = MerkleTree::new(vec![]);
        assert_eq!(tree.root(), EMPTY_ROOT);
    }

    #[test]
    fn test_single_leaf() {
        let leaf = [0xabu8; 32];
        let tree = MerkleTree::new(vec![leaf]);
        assert_eq!(tree.root(), leaf);
    }

    #[test]
    fn test_proof_roundtrip() {
        let leaves: Vec<[u8; 32]> = (0..100).map(|i| [i as u8; 32]).collect();
        let tree = MerkleTree::new(leaves.clone());
        let root = tree.root();

        for idx in [0, 10, 50, 99] {
            let proof = tree.proof(idx).unwrap();
            assert!(MerkleTree::verify(&root, &leaves[idx], &proof));
        }
    }

    #[test]
    fn test_tampered_proof_fails() {
        let leaves: Vec<[u8; 32]> = (0..10).map(|i| [i as u8; 32]).collect();
        let tree = MerkleTree::new(leaves.clone());
        let root = tree.root();

        let mut proof = tree.proof(3).unwrap();
        proof.siblings[0] = [0xffu8; 32]; // 篡改
        assert!(!MerkleTree::verify(&root, &leaves[3], &proof));
    }

    proptest! {
        #[test]
        fn merkle_root_deterministic(leaves in prop::collection::vec(any::<[u8; 32]>(), 0..100)) {
            let tree1 = MerkleTree::new(leaves.clone());
            let tree2 = MerkleTree::new(leaves.clone());
            assert_eq!(tree1.root(), tree2.root());
        }

        #[test]
        fn merkle_proof_verifies(
            leaves in prop::collection::vec(any::<[u8; 32]>(), 1..1000),
            idx in any::<prop::sample::Index>()
        ) {
            let tree = MerkleTree::new(leaves.clone());
            let root = tree.root();
            let index = idx.index(leaves.len());
            let proof = tree.proof(index).unwrap();
            assert!(MerkleTree::verify(&root, &leaves[index], &proof));
        }

        #[test]
        fn tampered_leaf_fails(
            leaves in prop::collection::vec(any::<[u8; 32]>(), 1..1000),
            idx in any::<prop::sample::Index>()
        ) {
            let tree = MerkleTree::new(leaves.clone());
            let root = tree.root();
            let index = idx.index(leaves.len());
            let proof = tree.proof(index).unwrap();
            let mut tampered_leaf = leaves[index];
            tampered_leaf[0] = tampered_leaf[0].wrapping_add(1);
            assert!(!MerkleTree::verify(&root, &tampered_leaf, &proof));
        }
    }
}
