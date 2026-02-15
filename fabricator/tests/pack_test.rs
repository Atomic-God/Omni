use fabricator::facade::OmniForge;

#[test]
fn test_pack_and_clone() {
    let data_dir = "./test_pack_data";
    std::fs::create_dir_all(data_dir).unwrap();
    let forge = OmniForge::new(data_dir);
    forge.learn("secret");

    let mind_path = "main";
    forge.mind.lock().unwrap().save(mind_path).expect("Save failed");

    let runtime = OmniForge::new("./runtime_data");
    runtime.load_runtime(data_dir, "./runtime_delta").expect("Load failed");
    runtime.mind.lock().unwrap().load(mind_path).expect("Load mind failed");

    let ans = runtime.run_query("secret");
    assert!(ans.contains("Closest match"));

    std::fs::remove_dir_all(data_dir).unwrap_or(());
    std::fs::remove_dir_all("./runtime_data").unwrap_or(());
    std::fs::remove_dir_all("./runtime_delta").unwrap_or(());
}
