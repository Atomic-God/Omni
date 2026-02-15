use engine::OmniMind;
use std::path::PathBuf;
use core_vsa::HyperVector;

#[test]
fn test_omnimind_forge_to_runtime() {
    let base_path = "./test_forge_data";
    let delta_path = "./test_runtime_delta";

    // 1. Initialize in Forge Mode
    let mut forge = OmniMind::new_forge(base_path);
    forge.learn("Rust is safe");
    forge.save("base").expect("Failed to save forge snapshot");

    // 2. Initialize in Runtime Mode
    let mut runtime = OmniMind::new_runtime(base_path, delta_path);
    runtime.load("base").expect("Failed to load base snapshot");

    // 3. Query the runtime mind
    let result = runtime.ask("Rust is safe");
    assert!(result.contains("Closest match"));

    // 4. Learn in Runtime (Delta)
    runtime.learn("VSA is fast");
    let result2 = runtime.ask("VSA is fast");
    assert!(result2.contains("Closest match"));

    // Cleanup
    std::fs::remove_dir_all(base_path).unwrap_or(());
    std::fs::remove_dir_all(delta_path).unwrap_or(());
}
