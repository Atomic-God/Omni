use engine::OmniMind;
use memory::{MindPack, MemoryStore, VocabStore, EncoderConfig};
use log::info;
use std::path::Path;

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
        if Path::new(data_path).exists() {
             // Basic directory walk or file read
             // We can use the 'ingest' crate logic if exposed, but for now re-implement simple walk or use library
             // But 'ingest' crate depends on 'engine'. 'fabricator' depends on 'engine'.
             // 'fabricator' can depend on 'ingest' if 'ingest' exposes ingest_path publicly.
             // But ingest/src/lib.rs function is public.
             // Wait, I didn't add 'ingest' dependency to 'fabricator' in step 1.
             // I'll assume simple iteration here or add dependency.
             // For simplicity in this step, I'll just assume data_path is a text file or dir.
             // Let's implement a simple file reader here to avoid circular dep hell if ingest depends on engine.
             // Actually, ingest depends on engine. fabricator depends on engine. fabricator -> ingest is fine.
             // I'll check fabricator/Cargo.toml.
        }

        // Hardcoded example if path not found or simple text
        if data_path == "default" {
            mind.learn("the dog is an animal");
            mind.learn("the animal is living");
            mind.learn("dog eats food");
        } else {
             // Ingest from file
             if let Ok(content) = std::fs::read_to_string(data_path) {
                 mind.learn(&content);
             }
        }

        info!("Fabrication complete. Packaging mind.");

        MindPack {
            version: "5.1".to_string(),
            memory: MemoryStore { core: mind.cognition.clone() },
            vocab: VocabStore { words: mind.cognition.index_memory.clone() },
            encoder_config: EncoderConfig { model_name: "beagle-v5".to_string() },
        }
    }
}
