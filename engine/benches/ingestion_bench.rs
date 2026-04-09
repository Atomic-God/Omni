use criterion::{black_box, criterion_group, criterion_main, Criterion};
use engine::OmniMind;
use std::fs;
use std::path::Path;

fn bench_pdf_ingestion(c: &mut Criterion) {
    let mut mind = OmniMind::new_forge("./bench_data_pdf");

    // Create a dummy "high-entropy" file to simulate content
    let content = "Industrial Ingestion Benchmark. ".repeat(1000);
    fs::write("bench_test.txt", content).unwrap();

    c.bench_function("ingest_large_text", |b| {
        b.iter(|| {
            mind.ingest_file(black_box("bench_test.txt")).unwrap();
        })
    });

    let _ = fs::remove_file("bench_test.txt");
    let _ = fs::remove_dir_all("./bench_data_pdf");
}

fn bench_batch_ingestion(c: &mut Criterion) {
    let mut mind = OmniMind::new_forge("./bench_data_batch");

    // Create a directory with 50 small files
    let dir = Path::new("bench_batch");
    fs::create_dir_all(dir).unwrap();
    for i in 0..50 {
        fs::write(dir.join(format!("file_{}.txt", i)), format!("Fact number {} for industrial batch testing.", i)).unwrap();
    }

    c.bench_function("batch_ingest_50_files", |b| {
        b.iter(|| {
            mind.ingest_file(black_box("bench_batch")).unwrap();
        })
    });

    let _ = fs::remove_dir_all(dir);
    let _ = fs::remove_dir_all("./bench_data_batch");
}

criterion_group!(benches, bench_pdf_ingestion, bench_batch_ingestion);
criterion_main!(benches);
