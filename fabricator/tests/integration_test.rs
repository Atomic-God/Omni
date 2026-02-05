use fabricator::facade::OmniForge;
use std::fs::File;
use std::io::Write;

#[test]
fn test_fabrication_runtime_parity() {
    // 1. Create a source file
    let source_path = "test_corpus.txt";
    let mut file = File::create(source_path).unwrap();
    writeln!(file, "the sky is blue").unwrap();

    // 2. Fabricate
    let forge = OmniForge::new();
    let mind_path = "test_mind.omf";
    forge.fabricate_mind(source_path, mind_path).expect("Fabrication failed");

    // 3. Load in a new instance (simulating runtime)
    let runtime = OmniForge::new();
    runtime.load_mind(mind_path).expect("Load failed");

    // 4. Query
    let answer = runtime.run_query("Is sky blue?");
    assert_eq!(answer, "Yes");

    // Cleanup
    std::fs::remove_file(source_path).unwrap();
    std::fs::remove_file(mind_path).unwrap();
}
