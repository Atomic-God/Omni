use fabricator::facade::OmniForge;
use std::fs::File;
use std::io::Write;

#[test]
fn test_pack_and_clone() {
    let data_dir = "test_pack_data";
    std::fs::create_dir_all(data_dir).unwrap();
    let file_path = format!("{}/doc.txt", data_dir);
    let mut file = File::create(&file_path).unwrap();
    writeln!(file, "packaging test").unwrap();

    let forge = OmniForge::new();
    let mind_path = "packed_mind.omf";

    // Fabricate Base
    forge.fabricate_mind(data_dir, mind_path).expect("Fabrication failed");

    // Simulate Packing (using CLI logic approach via session manually or just snapshot reuse)
    // Here we just test that the artifact loads

    // Clone
    let clone_path = "cloned_mind.omf";
    std::fs::copy(mind_path, clone_path).expect("Clone failed");

    // Load Clone
    let runtime = OmniForge::new();
    runtime.load_runtime(clone_path, None).expect("Load clone failed");

    // Verify Independence
    runtime.learn_runtime("clone secret").unwrap();

    // Load Original
    let original_runtime = OmniForge::new();
    original_runtime.load_runtime(mind_path, None).expect("Load original failed");

    // Original should NOT know secret
    let ans = original_runtime.run_query("clone secret");
    assert!(!ans.contains("clone secret"), "Original mind polluted by clone!");

    // Cleanup
    std::fs::remove_dir_all(data_dir).unwrap();
    std::fs::remove_file(mind_path).unwrap();
    std::fs::remove_file(clone_path).unwrap();
}
