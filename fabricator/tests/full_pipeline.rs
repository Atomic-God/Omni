use fabricator::facade::OmniForge;
use std::fs::File;
use std::io::Write;

#[test]
fn test_full_pipeline() {
    // 1. Setup ingestion data
    let data_dir = "test_data";
    std::fs::create_dir_all(data_dir).unwrap();
    let file_path = format!("{}/corpus.txt", data_dir);
    let mut file = File::create(&file_path).unwrap();
    writeln!(file, "the sky is blue").unwrap();
    writeln!(file, "the blue thing reflects light").unwrap();

    // 2. Fabricate
    let forge = OmniForge::new();
    let mind_path = "production_mind.omf";
    forge.fabricate_mind(data_dir, mind_path).expect("Fabrication failed"); // Should handle directory walk (if implemented) or file

    // 3. Run
    let runtime = OmniForge::new();
    runtime.load_mind(mind_path).expect("Load failed");

    // 4. Query
    // sky -> blue -> reflects -> light?
    // "Does sky reflect light?" -> Parsing might be tricky.
    // "Is sky blue?" -> Yes.
    let ans1 = runtime.run_query("Is sky blue?");
    assert!(ans1.contains("Yes") || ans1.contains("related"));

    // Cleanup
    std::fs::remove_dir_all(data_dir).unwrap();
    std::fs::remove_file(mind_path).unwrap();
}
