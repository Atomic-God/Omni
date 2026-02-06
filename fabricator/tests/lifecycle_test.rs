use fabricator::facade::OmniForge;
use engine::OmniMind;
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

    // Load and Verify State
    let runtime_forge = OmniForge::new();
    runtime_forge.load_mind(mind_path).expect("Load failed");

    let mind = runtime_forge.mind.lock().unwrap();
    assert!(mind.read_only, "Mind should be read-only after loading");

    // Cleanup
    std::fs::remove_dir_all(data_dir).unwrap();
    std::fs::remove_file(mind_path).unwrap();
}
