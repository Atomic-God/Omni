use engine::RuntimeMind;
use memory::{save_snapshot, MindPack, MemoryStore, VocabStore, EncoderConfig, LearningPolicies, MindMetadata, LifecycleState};
use cognition::CognitionCore;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

fn create_dummy_base(path: &str) {
    let core = CognitionCore::new();
    let pack = MindPack {
        version: "8.4".to_string(),
        memory: MemoryStore { core: core.clone() },
        vocab: VocabStore { words: HashMap::new() },
        encoder_config: EncoderConfig { model_name: "test".to_string() },
        learning_policies: LearningPolicies {
            reinforcement_rate: 0.1,
            decay_rate: 0.01,
            max_concepts: None,
        },
        metadata: MindMetadata {
            os: "linux".to_string(),
            arch: "x86_64".to_string(),
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            core_hash: core.compute_integrity_hash(),
            source: "test".to_string(),
            state: LifecycleState::Frozen,
            compiler_version: "1.0.0".to_string(),
            semantic_version: "1.0.0".to_string(),
        },
        blueprint: None,
    };
    save_snapshot(&pack, path).unwrap();
}

#[test]
fn test_goal_persistence_in_overlay() {
    let base_path = "test_base_mind.zip";
    let overlay_path = "test_overlay.bin";

    // 1. Create base mind
    create_dummy_base(base_path);

    // 2. Load RuntimeMind
    let mut runtime = RuntimeMind::load(base_path, Some(overlay_path)).expect("Failed to load runtime");

    // 3. Add a goal to overlay
    runtime.overlay.core.add_goal(
        "Build a house".to_string(),
        "House built".to_string(),
        10
    );

    assert_eq!(runtime.overlay.core.goals.len(), 1);
    assert_eq!(runtime.overlay.core.goals[0].description, "Build a house");

    // 4. Save overlay
    runtime.save_overlay(overlay_path).expect("Failed to save overlay");

    // 5. Reload RuntimeMind
    let loaded_runtime = RuntimeMind::load(base_path, Some(overlay_path)).expect("Failed to reload runtime");

    // 6. Verify goal persistence
    assert_eq!(loaded_runtime.overlay.core.goals.len(), 1);
    assert_eq!(loaded_runtime.overlay.core.goals[0].description, "Build a house");
    assert_eq!(loaded_runtime.overlay.core.goals[0].priority, 10);

    // Cleanup
    std::fs::remove_file(base_path).unwrap_or(());
    std::fs::remove_file(overlay_path).unwrap_or(());
}
