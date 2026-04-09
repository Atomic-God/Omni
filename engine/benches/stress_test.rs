use criterion::{black_box, criterion_group, criterion_main, Criterion};
use engine::OmniMind;

fn stress_test_ingestion(c: &mut Criterion) {
    let mut mind = OmniMind::new_forge("./stress_data");
    let text = "Industrial knowledge should be stable and scalable. ".repeat(100);

    c.bench_function("learn_100_sentences", |b| {
        b.iter(|| mind.learn(black_box(&text)))
    });

    let _ = std::fs::remove_dir_all("./stress_data");
}

fn stress_test_inference(c: &mut Criterion) {
    let mut mind = OmniMind::new_forge("./stress_data_inf");
    for i in 0..100 {
        mind.learn(&format!("Fact {} is verified.", i));
    }

    c.bench_function("ask_with_100_facts", |b| {
        b.iter(|| mind.ask(black_box("What is Fact 50?")))
    });

    let _ = std::fs::remove_dir_all("./stress_data_inf");
}

criterion_group!(benches, stress_test_ingestion, stress_test_inference);
criterion_main!(benches);
