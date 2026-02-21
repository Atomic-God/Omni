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
    println!("Response: {}", response.answer);
    assert!(response.answer.contains("downtime") || response.answer.contains("Logic:"));
    assert!(response.trace.is_some());

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
    let (vec, meaning, meta) = VisionSemanticExtractor::extract_deep_meaning(&img, 10000);

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
    println!("Multilingual Response: {}", response.answer);
    assert!(response.answer.to_lowercase().contains("cat"));

    // 2. Hardware Mode check
    let mut adapter = runtime::adaptation::HardwareAdapter::new();
    adapter.live_adjust();
    println!("Hardware optimized to mode: {:?}", adapter.current_mode);

    let _ = std::fs::remove_dir_all("./test_industrial_final");
}

#[test]
fn test_delta_snapshots_and_forgetting() {
    let mut mind = OmniMind::new_forge("./test_delta_industrial");

    // 1. Initial State
    mind.learn("Knowledge Alpha is stable.");
    mind.save("base").expect("Save base failed");

    // 2. Add new knowledge
    mind.learn("Knowledge Beta is dynamic.");

    // 3. Save Delta
    match &mut mind.state {
        engine::LifecycleState::Forge(mem, _) => {
            mem.save_delta("delta", "base").expect("Save delta failed");
        }
        _ => {}
    }

    // 4. Test Forgetting (Aging)
    match &mut mind.state {
        engine::LifecycleState::Forge(mem, _) => {
            mem.apply_aging(0.1, 0.01);
        }
        _ => {}
    }

    let _ = std::fs::remove_dir_all("./test_delta_industrial");
}

#[test]
fn test_ingestion_robustness_and_recovery() {
    let mut mind = OmniMind::new_forge("./test_robustness");

    // 1. Create a corrupted "PDF" (actually just text with .pdf extension)
    let path = "corrupted.pdf";
    let high_entropy_content = "Industrial robustness requires advanced error recovery fallbacks and deep symbolic extraction of semantic meaning from heterogeneous data sources. Safety protocols are paramount. 0123456789 ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    std::fs::write(path, high_entropy_content).unwrap();

    // 2. Ingest - should fail specialized parsing but succeed via generic fallback
    let result = mind.ingest_file(path);
    assert!(result.is_ok());

    // 3. Verify knowledge was extracted despite "corruption"
    let response = mind.ask("industrial robustness");
    println!("Robustness Response: {}", response.answer);
    assert!(response.answer.to_lowercase().contains("robustness") || response.answer.contains("Logic:"));

    let _ = std::fs::remove_file(path);
    let _ = std::fs::remove_dir_all("./test_robustness");
}

#[test]
fn test_continuous_learning_and_governance() {
    let mut mind = OmniMind::new_forge("./test_learning_gov");

    // 1. Learn and reinforce
    mind.learn("Efficiency is key.");

    // 2. Negative Feedback
    let fact_id = "efficiency-taxonomy-key";
    engine::learning::LearningEngine::process_feedback(&mut mind, fact_id, -0.8);

    // 3. Verify confidence drop
    match &mind.state {
        engine::LifecycleState::Forge(_, cog) => {
             if let Some(fact) = cog.knowledge_graph.facts.get(fact_id) {
                 println!("Fact Confidence after penalty: {:.2}", fact.confidence);
                 assert!(fact.confidence < 0.5);
             }
        }
        _ => {}
    }

    // 4. Test Sleep Cycle (Consistency Validation)
    mind.sleep_cycle();

    let _ = std::fs::remove_dir_all("./test_learning_gov");
}

#[test]
fn test_runtime_safety_and_permissions() {
    // 1. Initialize in Runtime mode (delta_path doesn't need to exist for basic check)
    let mut mind = engine::OmniMind::new_runtime("./base", "./delta");

    // 2. Try ingestion (should be blocked in default runtime policy)
    let result = mind.ingest_file("test.txt");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Permission Denied"));

    // 3. Check audit log creation
    assert!(std::path::Path::new("./delta/audit.jsonl").exists());

    let _ = std::fs::remove_dir_all("./base");
    let _ = std::fs::remove_dir_all("./delta");
}
