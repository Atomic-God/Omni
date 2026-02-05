use criterion::{criterion_group, criterion_main, Criterion};
use engine::OmniMind;

fn bench_learning(c: &mut Criterion) {
    let mut mind = OmniMind::new();
    let text = "The quick brown fox jumps over the lazy dog";

    c.bench_function("learning", |b| {
        b.iter(|| mind.learn(text))
    });
}

fn bench_query(c: &mut Criterion) {
    let mut mind = OmniMind::new();
    mind.learn("dog eats food");

    c.bench_function("query", |b| {
        b.iter(|| mind.ask("What does dog eat?"))
    });
}

criterion_group!(benches, bench_learning, bench_query);
criterion_main!(benches);
