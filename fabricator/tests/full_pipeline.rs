use fabricator::facade::OmniForge;

#[test]
fn test_full_pipeline() {
    let data_dir = "./test_fabricator_data";
    std::fs::create_dir_all(data_dir).unwrap();

    let forge = OmniForge::new(data_dir);
    forge.learn("the sky is blue");

    let mind_path = "main"; // matches save name
    forge.mind.lock().unwrap().save(mind_path).expect("Save failed");

    let runtime = OmniForge::new("./runtime_data");
    runtime.load_runtime(data_dir, "./runtime_delta").expect("Load failed");
    runtime.mind.lock().unwrap().load(mind_path).expect("Load mind failed");

    let ans1 = runtime.run_query("the sky is blue");
    assert!(ans1.contains("Closest match"));

    std::fs::remove_dir_all(data_dir).unwrap_or(());
    std::fs::remove_dir_all("./runtime_data").unwrap_or(());
    std::fs::remove_dir_all("./runtime_delta").unwrap_or(());
}
