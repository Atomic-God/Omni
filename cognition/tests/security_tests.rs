use cognition::{CognitionCore, RelationType};
use std::time::Instant;

#[test]
fn test_inference_dos_protection() {
    let mut core = CognitionCore::new();
    let timestamp = 0;

    // Create a dense, cyclic graph to trap BFS
    // A -> B, A -> C, B -> A, C -> A...
    // Also a long chain: 0 -> 1 -> 2 ... -> 1000

    for i in 0..1000 {
        let current = format!("node_{}", i);
        let next = format!("node_{}", i + 1);
        core.add_relation(current.clone(), next.clone(), RelationType::Causal, 10, timestamp);

        // Add back-links to create cycles
        if i > 0 {
            let prev = format!("node_{}", i - 1);
            core.add_relation(current.clone(), prev, RelationType::Causal, 10, timestamp);
        }
    }

    let start = Instant::now();
    // Try to find a path that exceeds the limit (e.g., node_0 to node_600).
    // The BFS limit is 500.
    // However, BFS finds shortest path. 0->600 is length 600.
    // The max_depth in multi_hop_inference is usually 3.
    // Wait, multi_hop_inference takes max_depth as an argument!
    // But the hardcoded limit MAX_NODES checks *total visited nodes*.

    // If we ask for depth 1000, it should hit MAX_NODES before depth 1000.
    let result = core.multi_hop_inference("node_0", "node_600", 1000);

    let duration = start.elapsed();
    println!("Inference took: {:?}", duration);

    // It should return None because it stopped early due to MAX_NODES (500)
    assert!(result.is_none(), "Inference should have been aborted due to security limit.");
    assert!(duration.as_millis() < 500, "Inference took too long!");
}

#[test]
fn test_planning_dos_protection() {
    let mut core = CognitionCore::new();
    let timestamp = 0;

    // Create a long chain for planning: 0 -> 1 -> ... -> 300
    for i in 0..300 {
         let current = format!("state_{}", i);
         let next = format!("state_{}", i + 1);
         core.add_relation(current, next, RelationType::Causal, 100, timestamp);
    }

    let start = Instant::now();
    // Planning limit is 200 steps.
    // 0 -> 250 requires 250 steps.
    let plan = core.find_path("state_0", "state_250");

    let duration = start.elapsed();
    println!("Planning took: {:?}", duration);

    assert!(plan.is_none(), "Planning should have been aborted due to step limit.");
}
