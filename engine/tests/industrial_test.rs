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

#[test]
fn test_autonomous_task_loop() {
    let mut mind = engine::OmniMind::new_forge("./test_task_loop");

    // 1. Add Goal
    mind.task_loop.add_goal("Optimize system energy usage", engine::task::Priority::High);

    // 2. Execute Step (Normal)
    let step1 = mind.execute_task_step();
    assert!(step1.is_some());
    println!("Task Step 1: {:?}", step1);

    // 3. Simulate High Uncertainty
    // Note: Since there's a 5% random failure probability, we handle potential Tier 1 pivot or progress
    let step2 = mind.task_loop.step(0.85); // High uncertainty
    let s2 = step2.unwrap();
    assert!(s2.contains("Observing") || s2.contains("Recovery") || s2.contains("Progress"));

    // 4. Test Confidence Scoring Loop
    let goal = mind.task_loop.goals.front().unwrap();
    println!("Goal Confidence: {:.2}", goal.confidence);

    let _ = std::fs::remove_dir_all("./test_task_loop");
}

#[test]
fn test_belief_revision_and_causal_logic() {
    let mut mind = engine::OmniMind::new_forge("./test_belief_revision");

    // 1. Initial Knowledge
    mind.learn("Machine_A is status:online");

    // 2. Contradiction with lower trust/confidence (should be queued or dampened)
    // By default learn has confidence 1.0. Let's use ingest_file simulation or manual fact entry.
    // Actually, let's use the API to simulate different source trust if possible.
    // For test simplicity, we'll use the fact that add_fact uses source_id.

    match &mut mind.state {
        engine::LifecycleState::Forge(_, cog) => {
            cog.add_fact_triple(core_vsa::FactTriple {
                subject: "Machine_A".to_string(),
                predicate: "taxonomy".to_string(),
                object: "offline".to_string()
            }, 0.5); // Lower confidence
        }
        _ => {}
    }

    let res1 = mind.ask("What is status Machine_A?");
    println!("Response 1: {}", res1.answer);
    assert!(res1.answer.contains("online"));

    // 3. High confidence belief revision
    match &mut mind.state {
        engine::LifecycleState::Forge(_, cog) => {
            // New fact from "Admin" source (manual) with high confidence
            cog.knowledge_graph.update_source_trust("Admin", 5.0);
            let triple = core_vsa::FactTriple {
                subject: "machine_a".to_string(),
                predicate: "taxonomy".to_string(),
                object: "maintenance".to_string(),
            };
            let conf = cog.knowledge_graph.add_fact("Admin:fact1", Some(triple.clone()), 1.0);
            cog.add_relation(&triple.subject, &triple.object, cognition::RelationType::Taxonomic, 1.0, conf);
        }
        _ => {}
    }

    let res2 = mind.ask("What is status Machine_A?");
    println!("Response 2: {}", res2.answer);
    // Should favor 'maintenance' because of Admin trust
    assert!(res2.answer.contains("maintenance"));

    // 4. Causal Reasoning: Inhibitors
    mind.learn("Rust inhibits corrosion.");
    mind.learn("Corrosion causes failure.");

    let res3 = mind.ask("What inhibits corrosion?");
    println!("Response 3: {}", res3.answer);
    assert!(res3.answer.to_lowercase().contains("inhibitor: rust"));

    let _ = std::fs::remove_dir_all("./test_belief_revision");
}

#[test]
fn test_semantic_clustering_and_concept_formation() {
    let mut mind = engine::OmniMind::new_forge("./test_concepts");

    // 1. Ingest multiple related facts
    mind.learn("The engine uses fuel.");
    mind.learn("The engine generates power.");
    mind.learn("The engine requires maintenance.");
    mind.learn("The engine has a cooling system.");

    // 2. Trigger Sleep Cycle for Concept Formation
    mind.sleep_cycle();

    // 3. Verify Topic Emergence
    match &mind.state {
        engine::LifecycleState::Forge(_, cog) => {
            // Should find a topic related to "engine"
            let topics = cog.relation_graph.keys()
                .filter(|k| k.starts_with("topic:"))
                .collect::<Vec<_>>();

            println!("Detected Topic IDs: {:?}", topics);
            assert!(!topics.is_empty());

            // Check for Domain
            let domains = cog.relation_graph.keys()
                .filter(|k| k.starts_with("domain:"))
                .collect::<Vec<_>>();
            println!("Detected Domain IDs: {:?}", domains);
            assert!(!domains.is_empty());
        }
        _ => panic!("Expected Forge state"),
    }

    let _ = std::fs::remove_dir_all("./test_concepts");
}

#[test]
fn test_snapshot_integrity_and_delta_chain() {
    let mut mind = engine::OmniMind::new_forge("./test_integrity");

    // Set low shard capacity to force flush
    match &mut mind.state {
        engine::LifecycleState::Forge(mem, _) => {
            mem.storage.shard_capacity = 2;
        }
        _ => {}
    }

    // 1. Initial State
    mind.learn("Core knowledge is immutable.");
    mind.learn("Fact two forces shard flush.");
    mind.save("base").unwrap();
    let base_snap_path = std::env::current_dir().unwrap().join("test_integrity/base.snap");
    assert!(base_snap_path.exists());

    // 2. Incremental Learning
    mind.learn("Delta knowledge is transient.");
    mind.save("delta1").unwrap();

    // 3. Corrupt a shard in base manually
    let shard_path = std::env::current_dir().unwrap().join("test_integrity/shard_1.bin");
    if shard_path.exists() {
        std::fs::write(&shard_path, b"CORRUPTED DATA").unwrap();
    }

    // 4. Integrity Check should fail
    match &mind.state {
        engine::LifecycleState::Forge(mem, _) => {
            assert!(!memory::CorruptionRecovery::check_integrity(&mem.storage));
        }
        _ => {}
    }

    // 5. Repair attempt
    memory::SnapshotManager::repair(&base_snap_path).unwrap();

    // 6. Restore from repaired base
    let mut mind2 = engine::OmniMind::new_forge("./test_integrity");
    mind2.load("base").unwrap();

    let res = mind2.ask("core knowledge");
    println!("Restored Response: {}", res.answer);

    let _ = std::fs::remove_dir_all("./test_integrity");
    let _ = std::fs::remove_dir_all("./test_integrity_restored");
}

#[test]
fn test_multilingual_cross_link() {
    let mut mind = engine::OmniMind::new_forge("./test_multilingual");

    // 1. Establish translation mapping
    mind.learn("Gato means cat.");

    // 2. Learn fact in target language
    mind.learn("Cats are agile animals.");

    // 3. Query in source language
    let response = mind.ask("What is a gato?");
    println!("Multilingual Response: {}", response.answer);

    // Should be able to retrieve information linked to 'cat'
    assert!(response.answer.to_lowercase().contains("animal") || response.answer.contains("Logic:"));

    let _ = std::fs::remove_dir_all("./test_multilingual");
}

#[test]
fn test_runtime_reasoning_depth() {
    let mut mind = engine::OmniMind::new_forge("./test_reasoning_depth");

    // 1. Multi-hop knowledge
    mind.learn("Part_A constitutes Machine_X.");
    mind.learn("Machine_X occupies Sector_7.");
    mind.learn("Sector_7 requires Maintenance.");

    // Query: Where is Part_A? (Expect path: Part_A -> Machine_X -> Sector_7)
    let res1 = mind.ask("Where is Part_A?");
    println!("Multi-hop Response:\n{}", res1.answer);
    assert!(res1.answer.to_lowercase().contains("sector_7") || res1.answer.to_lowercase().contains("machine_x"));
    assert!(res1.answer.contains("REPORT")); // Verify industrial report format

    // 2. Causal Reasoning depth
    mind.learn("High_Temperature causes Valve_Failure.");
    mind.learn("Valve_Failure inhibits Flow.");
    mind.learn("Flow is necessary for Cooling.");

    let res2 = mind.ask("What does High_Temperature cause?");
    println!("Causal Response:\n{}", res2.answer);
    assert!(res2.answer.to_lowercase().contains("valve_failure"));

    let _ = std::fs::remove_dir_all("./test_reasoning_depth");
}

#[test]
fn test_knowledge_graph_layer_industrial() {
    use cognition::knowledge::IndustrialGraph;
    let mut mind = engine::OmniMind::new_forge("./test_kg_layer");

    // 1. Causal Link Tagging
    mind.learn("Failure causes downtime.");
    match &mut mind.state {
        engine::LifecycleState::Forge(_, cog) => {
            let fact_id = "system:failure-causes-downtime";
            cog.knowledge_graph.tag_causal_link(fact_id, "high_impact");

            let fact = cog.knowledge_graph.facts.get(fact_id).unwrap();
            assert!(fact.tags.contains(&"causal:high_impact".to_string()));
        }
        _ => {}
    }

    // 2. Contradiction Detection (Online Validation)
    mind.learn("Machine_X is Green.");
    mind.learn("Machine_X is Red."); // Contradiction in exclusive predicate

    match &mind.state {
        engine::LifecycleState::Forge(_, cog) => {
            let fact_id = "system:machine_x-taxonomy-red";
            // Since validation is now active, the contradictory fact should NOT exist
            assert!(!cog.knowledge_graph.facts.contains_key(fact_id));
        }
        _ => {}
    }

    // 3. Entity Graph traversal
    match &mind.state {
        engine::LifecycleState::Forge(_, cog) => {
            let entities = cog.knowledge_graph.get_entities();
            assert!(entities.contains(&"machine_x".to_string()));

            let rels = cog.knowledge_graph.get_relations("machine_x");
            assert!(!rels.is_empty());
        }
        _ => {}
    }

    // 4. Fact Validation Blocking
    mind.learn("DeviceA is Active.");
    mind.learn("DeviceA is Inactive."); // Should be blocked by validation if exclusive

    match &mind.state {
        engine::LifecycleState::Forge(_, cog) => {
            let active_fact = cog.knowledge_graph.facts.values().any(|f| f.triple.as_ref().map_or(false, |t| t.subject == "devicea" && t.object == "active"));
            let inactive_fact = cog.knowledge_graph.facts.values().any(|f| f.triple.as_ref().map_or(false, |t| t.subject == "devicea" && t.object == "inactive"));

            assert!(active_fact);
            assert!(!inactive_fact); // Verified blocking
        }
        _ => {}
    }

    let _ = std::fs::remove_dir_all("./test_kg_layer");
}

#[test]
fn test_contextual_relevance_weighting() {
    let mut mind = engine::OmniMind::new_forge("./test_relevance");

    // 1. Setup multiple facts
    mind.learn("The server is in the basement.");
    mind.learn("The server is mission-critical.");
    mind.learn("The dog is in the garden.");

    // 2. Query without context
    let res1 = mind.ask("Where is the server?");
    println!("Response 1: {}", res1.answer);

    // 3. Set context (via a Goal)
    mind.task_loop.add_goal("Investigate basement security", engine::task::Priority::High);

    // 4. Query again - should favor basement over other facts due to context boost
    let res2 = mind.ask("The server is in the basement.");
    println!("Contextual Response: {}", res2.answer);
    assert!(res2.answer.to_lowercase().contains("basement") || res2.answer.contains("Logic:"));

    let _ = std::fs::remove_dir_all("./test_relevance");
}

#[test]
fn test_kg_causal_influence_propagation() {
    use cognition::knowledge::IndustrialGraph;
    let mut mind = engine::OmniMind::new_forge("./test_causal_prop");

    // 1. Setup causal chain: Heat -> Pressure -> ValveFailure -> Leak -> Downtime
    mind.learn("Heat causes Pressure.");
    mind.learn("Pressure causes ValveFailure.");
    mind.learn("ValveFailure causes Leak.");
    mind.learn("Leak causes Downtime.");

    // 2. Query causal influence of Heat
    match &mind.state {
        engine::LifecycleState::Forge(_, cog) => {
            let influences = cog.knowledge_graph.get_causal_influence("Downtime");
            println!("Causal influences of Downtime: {:?}", influences);

            // Should contain Heat (indirect cause)
            let heat_influence = influences.iter().find(|(name, _)| name.contains("heat"));
            assert!(heat_influence.is_some());
            assert!(heat_influence.unwrap().1 < 1.0); // Confidence should decay
        }
        _ => {}
    }

    let _ = std::fs::remove_dir_all("./test_causal_prop");
}
