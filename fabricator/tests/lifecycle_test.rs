use fabricator::facade::OmniForge;
use engine::LifecycleState;

#[test]
fn test_lifecycle_states() {
    let data_dir = "./test_lifecycle_data";
    let forge = OmniForge::new(data_dir);

    let status = forge.mind.lock().unwrap().lifecycle_status();
    assert!(status.contains("FORGE"));

    std::fs::remove_dir_all(data_dir).unwrap_or(());
}
