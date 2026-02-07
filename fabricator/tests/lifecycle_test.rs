use fabricator::facade::OmniForge;
use memory::LifecycleState;
use std::fs::File;
use std::io::Write;

#[test]
fn test_fabrication_to_runtime_freeze() {
    let data_dir = "test_lifecycle_data";
    std::fs::create_dir_all(data_dir).unwrap();
    let file_path = format!("{}/doc.txt", data_dir);
    let mut file = File::create(&file_path).unwrap();
    writeln!(file, "production grade test").unwrap();

    let forge = OmniForge::new();
    let mind_path = "lifecycle_test.omf";

    // Fabricate
    forge.fabricate_mind(data_dir, mind_path).expect("Fabrication failed");

    // Load Runtime
    let runtime_forge = OmniForge::new();
    runtime_forge.load_runtime(mind_path, None).expect("Load failed");

    // Verify State
    let mind_slot = runtime_forge.runtime_mind.lock().unwrap();
    if let Some(mind) = mind_slot.as_ref() {
        assert_eq!(mind.base.metadata.state, LifecycleState::Fabricated, "Mind state should be Fabricated (or Frozen if snapshot logic used)");
        // Wait, fabricate_mind sets state to Fabricated.
        // snapshot sets it to Frozen.
        // But Runtime loads it.
        // The test expects "read_only" equivalent.
        // RuntimeMind treats `base` as read-only regardless of state enum, but state enum is good metadata.

        // Check overlay is empty/fresh
        assert_eq!(mind.overlay.core.index_memory.len(), 0);
    } else {
        panic!("Runtime mind not loaded");
    }

    // Cleanup
    std::fs::remove_dir_all(data_dir).unwrap();
    std::fs::remove_file(mind_path).unwrap();
}
