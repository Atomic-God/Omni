use cognition::traits::{PerceptionModule, ReasoningModule};
use cognition::CognitionCore;
use log::{error, info};
use memory::{EncoderConfig, MemoryStore, MindPack, VocabStore};
use perception::decoder::TextDecoder;

/// Trait for extending OmniMind capabilities.
pub trait ExtensionModule: Send + Sync {
    fn process(&self, input: &str, mind: &mut OmniMind);
}

/// OmniMind is the high-level interface for the Omni Forge system.
/// It orchestrates cognitive processes, memory persistence, and interaction.
pub struct OmniMind {
    pub cognition: CognitionCore, // Public for extensions
    extensions: Vec<Box<dyn ExtensionModule>>,
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
        }
    }

    pub fn register_extension(&mut self, extension: Box<dyn ExtensionModule>) {
        self.extensions.push(extension);
    }

    /// Learns from the provided text using the Perception-Cognition pipeline.
    pub fn learn(&mut self, text: &str) {
        info!("Learning text: {}", text);
        // 1. Perception Layer (Encoding)
        // Ensure vocab is updated in Cognition (it handles it via learn_text currently).
        // In v5.1, we might separate this, but for now CognitionCore.learn_text does the heavy lifting.
        // We can create a temporary TextEncoder to validate or process, but CognitionCore is the learner.
        self.cognition.learn_text(text);

        // 2. Extensions
        let exts = std::mem::take(&mut self.extensions);
        for ext in &exts {
            ext.process(text, self);
        }
        self.extensions = exts;
    }

    /// Processes a query using Perception -> Cognition -> Perception (Decode) pipeline.
    pub fn ask(&self, question: &str) -> String {
        info!("Processing query: {}", question);

        // Use CognitionCore for reasoning-based answers (Yes/No, Fact retrieval)
        self.cognition.query(question)
    }

    /// Generates text from a raw HyperVector using the Perception layer (Decoder).
    pub fn generate(&self, hv: &core_vsa::HyperVector) -> String {
        let decoder = TextDecoder::new(self.cognition.index_memory.clone());
        decoder.decode_svo(hv).0
    }

    /// Saves the current state of the mind to the specified path using MindPack.
    pub fn save(&self, path: &str) -> Result<(), std::io::Error> {
        info!("Saving mind to {}", path);
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
                // Reconstruct or validate vocab if needed
                Ok(())
            }
            Err(e) => {
                error!("Failed to load mind: {}", e);
                Err(e)
            }
        }
    }
}
