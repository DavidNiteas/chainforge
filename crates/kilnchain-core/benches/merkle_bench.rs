use criterion::{black_box, criterion_group, criterion_main, Criterion};
use kilnchain_core::merkle::MerkleTree;

fn bench_merkle_root(c: &mut Criterion) {
    for size in [1, 10, 100, 1_000, 10_000] {
        let leaves: Vec<[u8; 32]> = (0..size)
            .map(|i: u64| {
                let mut buf = [0u8; 32];
                buf[..8].copy_from_slice(&i.to_be_bytes());
                buf
            })
            .collect();

        c.bench_function(&format!("merkle_root_{}", size), |b| {
            b.iter(|| {
                let tree = MerkleTree::new(black_box(leaves.clone()));
                black_box(tree.root())
            })
        });
    }
}

criterion_group!(benches, bench_merkle_root);
criterion_main!(benches);
