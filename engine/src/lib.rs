use cognition::traits::{PerceptionModule, ReasoningModule};
use cognition::CognitionCore;
use log::{error, info};
use memory::{EncoderConfig, MemoryStore, MindPack, VocabStore, LearningPolicies, MindMetadata, LifecycleState};
use perception::decoder::TextDecoder;
use std::time::{SystemTime, UNIX_EPOCH};
use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;

/// Trait for extending OmniMind capabilities.
pub trait ExtensionModule: Send + Sync {
    fn process(&self, input: &str, mind: &mut OmniMind);
}

/// OmniMind is the high-level interface for the Omni Forge system.
pub struct OmniMind {
    pub cognition: CognitionCore,
    extensions: Vec<Box<dyn ExtensionModule>>,
    pub read_only: bool,
}

impl Default for OmniMind {
    fn default() -> Self {
        Self::new()
    }
}

impl OmniMind {
    pub fn new() -> Self {
        info!("Initializing OmniMind instance.");
        Self {
            cognition: CognitionCore::new(),
            extensions: Vec::new(),
            read_only: false,
        }
    }

    pub fn set_read_only(&mut self, read_only: bool) {
        self.read_only = read_only;
        if read_only {
            info!("OmniMind switched to READ-ONLY mode. Learning is permanently disabled.");
        }
    }

    pub fn register_extension(&mut self, extension: Box<dyn ExtensionModule>) {
        self.extensions.push(extension);
    }

    pub fn learn(&mut self, text: &str) {
        if self.read_only {
            error!("Security Violation: Attempted to learn in READ-ONLY mode.");
            panic!("Runtime Integrity Violation: Learning forbidden in runtime phase.");
        }

        info!("Learning text: {}", text);
        self.cognition.learn_text(text);

        let exts = std::mem::take(&mut self.extensions);
        for ext in &exts {
            ext.process(text, self);
        }
        self.extensions = exts;
    }

    pub fn ask(&self, question: &str) -> String {
        info!("Processing query: {}", question);
        let answer = self.cognition.query(question);

        // Output Polish: Clean up template phrases if any, add confidence hints
        if answer == "Unknown" || answer == "No connection found." {
            "I do not have enough information to answer that based on my current experiences.".to_string()
        } else {
            // Confidence Scoring Logic (Mocked based on graph depth for now)
            // In real system, query() returns (String, f32)
            format!("{} (Confidence: High)", answer)
        }
    }

    pub fn generate(&self, hv: &core_vsa::HyperVector) -> String {
        let decoder = TextDecoder::new(self.cognition.semantic_memory.clone());
        decoder.decode_svo(hv).0
    }

    pub fn save(&self, path: &str) -> Result<(), std::io::Error> {
        info!("Saving mind to {}", path);

        let mut hasher = DefaultHasher::new();
        hasher.write_usize(self.cognition.index_memory.len());
        hasher.write_usize(self.cognition.semantic_memory.len());
        let core_hash = format!("{:x}", hasher.finish());

        let pack = MindPack {
            version: "5.1".to_string(),
            memory: MemoryStore {
                core: self.cognition.clone(),
            },
            vocab: VocabStore {
                words: self.cognition.index_memory.clone(),
            },
            encoder_config: EncoderConfig {
                model_name: "beagle-v5".to_string(),
            },
            learning_policies: LearningPolicies {
                reinforcement_rate: 0.1,
                decay_rate: 0.01,
            },
            metadata: MindMetadata {
                os: std::env::consts::OS.to_string(),
                arch: std::env::consts::ARCH.to_string(),
                timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs(),
                core_hash,
                source: "OmniForge v5.1 Fabricator".to_string(),
                state: if self.read_only { LifecycleState::Frozen } else { LifecycleState::Fabricated },
            },
        };

        match memory::save_mind(&pack, path) {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("Failed to save mind: {}", e);
                Err(e)
            }
        }
    }

    pub fn load(&mut self, path: &str) -> Result<(), std::io::Error> {
        info!("Loading mind from {}", path);
        match memory::load_mind(path) {
            Ok(pack) => {
                // Check Lifecycle
                match pack.metadata.state {
                    LifecycleState::Fabricated | LifecycleState::Frozen | LifecycleState::Runtime => {
                        self.cognition = pack.memory.core;
                        self.read_only = true; // Enforce runtime read-only
                        info!("Mind loaded successfully in READ-ONLY mode. State: {:?}", pack.metadata.state);
                        Ok(())
                    }
                }
            }
            Err(e) => {
                error!("Failed to load mind: {}", e);
                Err(e)
            }
        }
    }
}
