#[cfg(feature = "rocksdb-backend")]
mod rocksdb_bench {
    use chainforge_storage::rocksdb::RocksDBEngine;
    use criterion::{black_box, criterion_group, criterion_main, Criterion};
    use std::path::PathBuf;

    fn bench_rocksdb_random_read(c: &mut Criterion) {
        let tmp = tempfile::tempdir().unwrap();
        let db = RocksDBEngine::open(tmp.path()).unwrap();

        // Pre-populate 1000 records
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            for i in 0..1000 {
                let key = format!("key{:08}", i);
                let value = vec![0u8; 1024];
                db.put(key.as_bytes(), &value).await.unwrap();
            }
        });

        c.bench_function("rocksdb_random_get", |b| {
            b.iter(|| {
                let key = format!("key{:08}", black_box(500));
                rt.block_on(async {
                    db.get(key.as_bytes()).await.unwrap();
                });
            })
        });
    }

    criterion_group!(benches, bench_rocksdb_random_read);
    criterion_main!(benches);
}

#[cfg(not(feature = "rocksdb-backend"))]
fn main() {
    println!("rocksdb-backend feature not enabled, skipping bench");
}
