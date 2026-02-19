use engine::OmniMind;
use std::sync::{Arc, Mutex};
use std::thread;

#[test]
fn test_high_concurrency_load() {
    let mind = Arc::new(Mutex::new(OmniMind::new_forge("./test_concurrency")));
    let mut handles = Vec::new();

    // Spawn 10 "Learners"
    for i in 0..10 {
        let m = Arc::clone(&mind);
        let handle = thread::spawn(move || {
            for j in 0..50 {
                let mut lock = m.lock().unwrap();
                lock.learn(&format!("Learner {} is sending data packet {}.", i, j));
            }
        });
        handles.push(handle);
    }

    // Spawn 10 "Querying"
    for i in 0..10 {
        let m = Arc::clone(&mind);
        let handle = thread::spawn(move || {
            for _ in 0..50 {
                let mut lock = m.lock().unwrap();
                let _ = lock.ask(&format!("What is Learner {} doing?", i));
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let lock = mind.lock().unwrap();
    println!("Concurrency Test Finished. Final stats: {}", lock.memory_stats());

    let _ = std::fs::remove_dir_all("./test_concurrency");
}
