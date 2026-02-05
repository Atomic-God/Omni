use criterion::{criterion_group, criterion_main, Criterion};
use engine::OmniMind;

fn bench_learning(c: &mut Criterion) {
    let mut mind = OmniMind::new();
    let text = "The quick brown fox jumps over the lazy dog";

    c.bench_function("learning", |b| b.iter(|| mind.learn(text)));
}

fn bench_query(c: &mut Criterion) {
    let mut mind = OmniMind::new();
    mind.learn("dog eats food");

    c.bench_function("query", |b| b.iter(|| mind.ask("What does dog eat?")));
}

fn bench_encoding(c: &mut Criterion) {
    // We can't access inner perception directly easily via public API for micro-benchmark
    // So we benchmark the 'learn' step which includes encoding.
    // Or we create a large text.
    let mut mind = OmniMind::new();
    let text =
        "This is a longer sentence to benchmark the encoding latency of the neuro-symbolic mind.";
    c.bench_function("encoding_full_pipeline", |b| b.iter(|| mind.learn(text)));
}

criterion_group!(benches, bench_learning, bench_query, bench_encoding);
criterion_main!(benches);
