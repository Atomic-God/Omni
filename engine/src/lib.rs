use cognition::traits::{PerceptionModule, ReasoningModule};
use cognition::CognitionCore;
use log::{error, info};
use memory::{EncoderConfig, MemoryStore, MindPack, VocabStore, LearningPolicies, MindMetadata};
use perception::decoder::TextDecoder;
use std::time::{SystemTime, UNIX_EPOCH};
use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;

/// Trait for extending OmniMind capabilities.
pub trait ExtensionModule: Send + Sync {
    fn process(&self, input: &str, mind: &mut OmniMind);
}

/// OmniMind is the high-level interface for the Omni Forge system.
/// It orchestrates cognitive processes, memory persistence, and interaction.
pub struct OmniMind {
    pub cognition: CognitionCore, // Public for extensions
    extensions: Vec<Box<dyn ExtensionModule>>,
    pub read_only: bool,
}

impl Default for OmniMind {
    fn default() -> Self {
        Self::new()
    }
}

impl OmniMind {
    /// Creates a new, empty OmniMind instance.
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
            info!("OmniMind switched to READ-ONLY mode.");
        }
    }

    pub fn register_extension(&mut self, extension: Box<dyn ExtensionModule>) {
        self.extensions.push(extension);
    }

    /// Learns from the provided text using the Perception-Cognition pipeline.
    pub fn learn(&mut self, text: &str) {
        if self.read_only {
            error!("Security Violation: Attempted to learn in READ-ONLY mode.");
            panic!("Runtime Integrity Violation: Learning forbidden in runtime phase.");
        }

        info!("Learning text: {}", text);
        self.cognition.learn_text(text);

        // Extensions
        let exts = std::mem::take(&mut self.extensions);
        for ext in &exts {
            ext.process(text, self);
        }
        self.extensions = exts;
    }

    /// Processes a query using Perception -> Cognition -> Perception (Decode) pipeline.
    pub fn ask(&self, question: &str) -> String {
        info!("Processing query: {}", question);
        self.cognition.query(question)
    }

    /// Generates text from a raw HyperVector using the Perception layer (Decoder).
    pub fn generate(&self, hv: &core_vsa::HyperVector) -> String {
        let decoder = TextDecoder::new(self.cognition.semantic_memory.clone());
        decoder.decode_svo(hv).0
    }

    /// Saves the current state of the mind to the specified path using MindPack.
    pub fn save(&self, path: &str) -> Result<(), std::io::Error> {
        info!("Saving mind to {}", path);

        // Compute Integrity Hash (Naive: Hash of number of keys)
        // Real production would hash the serialized content.
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

    /// Loads a mind state from the specified path.
    pub fn load(&mut self, path: &str) -> Result<(), std::io::Error> {
        info!("Loading mind from {}", path);
        match memory::load_mind(path) {
            Ok(pack) => {
                self.cognition = pack.memory.core;
                // Verify integrity if needed
                self.read_only = true; // Enforce runtime read-only by default on load
                Ok(())
            }
            Err(e) => {
                error!("Failed to load mind: {}", e);
                Err(e)
            }
        }
    }
}
