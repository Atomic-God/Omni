use criterion::{black_box, criterion_group, criterion_main, Criterion};
use core_vsa::HyperVector;
use cognition::sequence::SequenceResonator;
use memory::lsh::LSHIndex;

fn bench_hypervector_ops(c: &mut Criterion) {
    let hv1 = HyperVector::random();
    let hv2 = HyperVector::random();

    c.bench_function("vsa_bind", |b| b.iter(|| {
        black_box(hv1.bind(&hv2))
    }));

    c.bench_function("vsa_bundle", |b| b.iter(|| {
        black_box(hv1.bundle(&hv2))
    }));

    c.bench_function("vsa_permute", |b| b.iter(|| {
        black_box(hv1.permute(1))
    }));
}

fn bench_sequence_folding(c: &mut Criterion) {
    let mut resonator = SequenceResonator::new(10);
    let vector = HyperVector::random();

    c.bench_function("sequence_add_token", |b| b.iter(|| {
        resonator.add(black_box(&vector))
    }));
}

fn bench_memory_query(c: &mut Criterion) {
    let mut index = LSHIndex::new();
    // Populate with 1000 items
    for i in 0..1000 {
        index.insert(&format!("item_{}", i), HyperVector::random());
    }

    let query = HyperVector::random();

    c.bench_function("lsh_query_1k", |b| b.iter(|| {
        black_box(index.query(&query, 5))
    }));
}

criterion_group!(benches, bench_hypervector_ops, bench_sequence_folding, bench_memory_query);
criterion_main!(benches);
