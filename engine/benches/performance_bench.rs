use criterion::{black_box, criterion_group, criterion_main, Criterion};
use engine::OmniMind;
use cognition::knowledge::{KnowledgeGraph, ContradictionEngine};
use core_vsa::FactTriple;
use std::fs;
use std::path::Path;

fn bench_industrial_batch_ingestion(c: &mut Criterion) {
    let mut mind = OmniMind::new_forge("./bench_industrial_batch");

    // Setup 100 small text files for batch ingestion
    let dir = Path::new("bench_batch_industrial");
    fs::create_dir_all(dir).unwrap();
    for i in 0..100 {
        fs::write(dir.join(format!("file_{}.txt", i)),
            format!("Industrial Fact {}: The system reliability is {}% at load {}.", i, 90 + (i % 10), i * 10)).unwrap();
    }

    c.bench_function("batch_ingest_100_files", |b| {
        b.iter(|| {
            // We re-ingest the same folder to measure the BatchPipeline throughput
            mind.ingest_file(black_box("bench_batch_industrial")).unwrap();
        })
    });

    let _ = fs::remove_dir_all(dir);
    let _ = fs::remove_dir_all("./bench_industrial_batch");
}

fn bench_contradiction_engine_scan(c: &mut Criterion) {
    let mut kg = KnowledgeGraph::new();

    // Populate KG with a chain of transitive facts: A -> B -> C -> D -> E
    for i in 0..50 {
        let triple = FactTriple {
            subject: format!("node_{}", i),
            predicate: "taxonomy".to_string(),
            object: format!("node_{}", i + 1),
        };
        kg.add_fact(&format!("fact_{}", i), Some(triple), 1.0);
    }

    c.bench_function("contradiction_scan_50_chain", |b| {
        b.iter(|| {
            black_box(ContradictionEngine::scan_transitive_inconsistencies(black_box(&kg)))
        })
    });
}

criterion_group!(benches, bench_industrial_batch_ingestion, bench_contradiction_engine_scan);
criterion_main!(benches);
