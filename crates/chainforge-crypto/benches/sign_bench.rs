use chainforge_crypto::ecdsa::SecretKey;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_sign(c: &mut Criterion) {
    let sk = SecretKey::random();
    let msg = b"benchmark message";

    c.bench_function("ecdsa_sign", |b| {
        b.iter(|| sk.sign(black_box(msg)))
    });

    let sig = sk.sign(msg).unwrap();
    let pk = sk.public_key();

    c.bench_function("ecdsa_verify", |b| {
        b.iter(|| pk.verify(black_box(msg), black_box(&sig)))
    });
}

criterion_group!(benches, bench_sign);
criterion_main!(benches);
