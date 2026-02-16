use engine::OmniMind;
use cognition::abductive::AbductiveReasoner;
use ingestion::vision::VisionSemanticExtractor;
use image::{DynamicImage, RgbaImage};
use memory::HierarchicalMemory;

#[test]
fn test_industrial_meaning_extraction() {
    let mut mind = OmniMind::new_forge("./test_industrial_meaning");

    // 1. Deep NLP Meaning: SVO with causality
    mind.learn("Failure causes downtime. Downtime triggers alert.");

    // 2. Transitive Industrial Logic
    let response = mind.ask("What does failure cause?");
    println!("Response: {}", response);
    assert!(response.contains("downtime") || response.contains("Logic:"));

    // 3. Abductive Reasoning Test
    let cog = match &mind.state {
        engine::LifecycleState::Forge(_, c) => c,
        _ => panic!("Expected Forge state"),
    };
    let reasoner = AbductiveReasoner::new(0.5);
    let explanation = reasoner.explain(&cog.knowledge_graph, "alert");
    assert!(explanation.is_some());
    println!("Abduction Hypothesis: {}", explanation.unwrap().hypothesis);

    let _ = std::fs::remove_dir_all("./test_industrial_meaning");
}

#[test]
fn test_vision_grammar_and_entropy() {
    let img = DynamicImage::ImageRgba8(RgbaImage::new(200, 200));
    let (vec, meaning, meta) = VisionSemanticExtractor::extract_deep_meaning(&img);

    assert!(vec.dim == core_vsa::DIMENSION);
    assert!(meta.contains_key("visual_atom_count"));
    assert!(meta.contains_key("conceptual_entropy"));
    println!("Vision Meaning: {}", meaning);
}

#[test]
fn test_memory_prototype_merging() {
    let mut mind = OmniMind::new_forge("./test_memory_merging");

    for i in 0..10 {
        mind.learn(&format!("Machine {} is active", i));
    }

    match &mut mind.state {
        engine::LifecycleState::Forge(mem, _) => {
            mem.consolidate_layers();
        }
        _ => {}
    }

    let stats = mind.memory_stats();
    println!("Stats after merging: {}", stats);

    let _ = std::fs::remove_dir_all("./test_memory_merging");
}

#[test]
fn test_hardware_adaptation_and_multilingual() {
    let mut mind = OmniMind::new_forge("./test_industrial_final");

    // 1. Multilingual
    mind.learn("Gato means cat.");
    let response = mind.ask("What is a gato?");
    println!("Multilingual Response: {}", response);
    assert!(response.to_lowercase().contains("cat"));

    // 2. Hardware Mode check
    let mut adapter = runtime::adaptation::HardwareAdapter::new();
    adapter.live_adjust();
    println!("Hardware optimized to mode: {:?}", adapter.current_mode);

    let _ = std::fs::remove_dir_all("./test_industrial_final");
}
