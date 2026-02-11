use criterion::{criterion_group, criterion_main, Criterion};
use engine::ForgeMind;

fn benchmark_learning(c: &mut Criterion) {
    c.bench_function("learning_step", |b| b.iter(|| {
        let mut mind = ForgeMind::new();
        mind.learn("The quick brown fox jumps over the lazy dog.");
    }));
}

fn benchmark_inference(c: &mut Criterion) {
    let mut mind = ForgeMind::new();
    mind.learn("A implies B.");
    mind.learn("B implies C.");

    c.bench_function("inference_chain", |b| b.iter(|| {
        mind.ask("Does A imply C?");
    }));
}

criterion_group!(benches, benchmark_learning, benchmark_inference);
criterion_main!(benches);
