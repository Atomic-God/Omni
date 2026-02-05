use engine::OmniMind;
use memory::{MindPack, MemoryStore, VocabStore, EncoderConfig, LearningPolicies};
use log::info;
use std::path::PathBuf;

pub mod facade;

pub struct FabricationPipeline;

impl FabricationPipeline {
    pub fn new() -> Self {
        Self
    }

    pub fn fabricate(&self, data_path: &str) -> MindPack {
        info!("Starting fabrication from: {}", data_path);
        let mut mind = OmniMind::new();

        // Ingest data
        let path = PathBuf::from(data_path);
        if path.exists() {
             let chunks = ingestion::ingest_path(path);
             for chunk in chunks {
                 mind.learn(&chunk.content);
             }
        } else if data_path == "default" {
            mind.learn("the dog is an animal");
            mind.learn("the animal is living");
            mind.learn("dog eats food");
        } else {
            log::warn!("Data path not found: {}", data_path);
        }

        info!("Fabrication complete. Packaging mind.");

        MindPack {
            version: "5.1".to_string(),
            memory: MemoryStore { core: mind.cognition.clone() },
            vocab: VocabStore { words: mind.cognition.index_memory.clone() },
            encoder_config: EncoderConfig { model_name: "beagle-v5".to_string() },
            learning_policies: LearningPolicies {
                reinforcement_rate: 0.1,
                decay_rate: 0.01,
            },
        }
    }
}
