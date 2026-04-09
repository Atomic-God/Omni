use engine::OmniMind;
use std::time::Instant;
use std::sync::{Arc, Mutex};
use std::thread;

#[test]
fn test_industrial_load_stress_10k() {
    let mut mind = OmniMind::new_forge("./test_stress_10k");

    println!("Starting Stress Test: Ingesting 10,000 facts...");
    let start = Instant::now();
    for i in 0..10_000 {
        mind.learn(&format!("Industrial sensor {} reports value {} at timestamp {}.", i, i * 3, i + 1000));
    }
    let elapsed = start.elapsed();
    println!("Ingested 10,000 facts in {:?}", elapsed);
    assert!(elapsed.as_secs() < 60, "Ingestion took too long: {:?}", elapsed);

    // Verify some knowledge
    let response = mind.ask("sensor 5000");
    println!("Response for sensor 5000: {}", response.answer);
    assert!(response.answer.contains("5000") || response.answer.contains("Logic:"));

    let _ = std::fs::remove_dir_all("./test_stress_10k");
}

#[test]
fn test_high_concurrency_100_threads() {
    let mind = Arc::new(Mutex::new(OmniMind::new_forge("./test_stress_threads")));
    let mut handles = Vec::new();

    println!("Starting Concurrency Stress: 100 threads...");
    let start = Instant::now();

    for t in 0..100 {
        let m = Arc::clone(&mind);
        let handle = thread::spawn(move || {
            for i in 0..20 {
                if i % 2 == 0 {
                    let mut lock = m.lock().unwrap();
                    lock.learn(&format!("Thread {} learning unit {}.", t, i));
                } else {
                    let mut lock = m.lock().unwrap();
                    let _ = lock.ask(&format!("What is thread {} doing?", t));
                }
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("100 threads completed in {:?}", start.elapsed());

    let lock = mind.lock().unwrap();
    println!("Final Memory Stats: {}", lock.memory_stats());

    let _ = std::fs::remove_dir_all("./test_stress_threads");
}
