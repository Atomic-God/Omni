use core_vsa::HyperVector;
use std::time::Instant;
use memory::LSHIndex;

pub struct VSABenchmark;

impl VSABenchmark {
    pub fn run() -> String {
        let mut report = String::new();
        report.push_str("VSA Core Benchmark\n");
        report.push_str("==================\n");

        // 1. Bind Latency
        let a = HyperVector::random();
        let b = HyperVector::random();
        let start = Instant::now();
        for _ in 0..10_000 {
            let _ = a.bind(&b);
        }
        let dur = start.elapsed();
        report.push_str(&format!("Bind Latency (10k ops): {:.3} ms\n", dur.as_secs_f64() * 1000.0));

        // 2. Bundle Latency
        let start = Instant::now();
        for _ in 0..10_000 {
            let _ = a.bundle(&b);
        }
        let dur = start.elapsed();
        report.push_str(&format!("Bundle Latency (10k ops): {:.3} ms\n", dur.as_secs_f64() * 1000.0));

        // 3. Similarity Latency
        let start = Instant::now();
        for _ in 0..10_000 {
            let _ = a.similarity(&b);
        }
        let dur = start.elapsed();
        report.push_str(&format!("Similarity Latency (10k ops): {:.3} ms\n", dur.as_secs_f64() * 1000.0));

        // 4. Memory Scaling (LSH)
        let mut index = LSHIndex::new();
        let start = Instant::now();
        for i in 0..10_000 {
            index.insert(&format!("item_{}", i), HyperVector::random());
        }
        let dur = start.elapsed();
        report.push_str(&format!("LSH Insert (10k items): {:.3} ms\n", dur.as_secs_f64() * 1000.0));

        // Query
        let start = Instant::now();
        let _res = index.query(&a, 10);
        let dur = start.elapsed();
        report.push_str(&format!("LSH Query (10k items): {:.3} ms\n", dur.as_secs_f64() * 1000.0));

        report
    }
}
