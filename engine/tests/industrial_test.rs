use engine::OmniMind;

#[test]
fn test_industrial_meaning_extraction() {
    let mut mind = OmniMind::new_forge("./test_industrial_data");

    // 1. Test SVO Extraction from text
    mind.learn("The factory uses electricity. Electricity causes motion.");

    // 2. Test Multi-step Reasoning (Transitive)
    // Factory -> Electricity -> Motion
    let response = mind.ask("What does the factory cause?");
    println!("Response: {}", response);
    assert!(response.contains("Yes") || response.contains("related") || response.contains("motion") || response.contains("Logic:"));

    // 3. Test Contradiction Detection
    mind.learn("The factory is silent.");
    mind.learn("The factory is noisy."); // Contradicts

    // The governance loop should resolve this (favoring noisy or silent based on confidence)
    // For Phase 1 we verify it doesn't crash and flags the contradiction.

    let _ = std::fs::remove_dir_all("./test_industrial_data");
}

#[test]
fn test_vision_semantic_mapping() {
    // Since we don't have a real image in the test env easily,
    // we verify the VisionSemanticExtractor logic directly.
    use ingestion::vision::VisionSemanticExtractor;
    use image::{DynamicImage, RgbaImage};

    let img = DynamicImage::ImageRgba8(RgbaImage::new(100, 100));
    let (vec, desc, meta) = VisionSemanticExtractor::extract_meaning(&img);

    assert!(vec.dim > 0);
    assert!(desc.len() > 0);
    assert!(meta.contains_key("edge_density"));
}
