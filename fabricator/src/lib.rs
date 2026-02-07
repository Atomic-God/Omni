use engine::OmniMind;
use memory::{MindPack, MemoryStore, VocabStore, EncoderConfig, LearningPolicies, MindMetadata, LifecycleState};
use log::{info, warn};
use std::path::PathBuf;
use learning::LearningEngine;
use std::time::{SystemTime, UNIX_EPOCH};

pub mod facade;
pub mod live;
pub mod merge;

pub struct FabricationPipeline;

impl FabricationPipeline {
    pub fn new() -> Self {
        Self
    }

    pub fn fabricate(&self, data_path: &str) -> MindPack {
        info!("Starting fabrication from: {}", data_path);
        let mut mind = OmniMind::new();
        let mut engine = LearningEngine::new();

        // Ingest data
        let path = PathBuf::from(data_path);
        if path.exists() {
             let chunks = ingestion::ingest_path(path);
             // Use Learning Engine
             engine.learn(&mut mind.cognition, chunks);
        } else {
            warn!("Data path not found or empty: {}", data_path);
        }

        // Final consolidation
        engine.consolidate(&mut mind.cognition);
        engine.freeze();

        info!("Fabrication complete. Packaging mind.");

        let core_hash = mind.cognition.compute_integrity_hash();

        MindPack {
            version: "5.1".to_string(),
            memory: MemoryStore { core: mind.cognition.clone() },
            vocab: VocabStore { words: mind.cognition.index_memory.clone() },
            encoder_config: EncoderConfig { model_name: "beagle-v5".to_string() },
            learning_policies: LearningPolicies {
                reinforcement_rate: 0.1,
                decay_rate: 0.01,
            },
            metadata: MindMetadata {
                os: std::env::consts::OS.to_string(),
                arch: std::env::consts::ARCH.to_string(),
                timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs(),
                core_hash,
                source: data_path.to_string(),
                state: LifecycleState::Fabricated,
            },
        }
    }
}
