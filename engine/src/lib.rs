use cognition::CognitionCore;
use cognition::traits::{PerceptionModule, ReasoningModule};
use log::{error, info};

/// OmniMind is the high-level interface for the Omni Forge system.
/// It orchestrates cognitive processes, memory persistence, and interaction.
pub struct OmniMind {
    cognition: CognitionCore,
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
        }
    }

    /// Learns from the provided text, updating semantic and episodic memory.
    pub fn learn(&mut self, text: &str) {
        info!("Learning text: {}", text);
        self.cognition.learn_text(text);
    }

    /// Processes a natural language query and returns an answer.
    pub fn ask(&self, question: &str) -> String {
        info!("Processing query: {}", question);
        self.cognition.query(question)
    }

    /// Saves the current state of the mind to the specified path.
    pub fn save(&self, path: &str) -> Result<(), std::io::Error> {
        info!("Saving memory to {}", path);
        match memory::save(&self.cognition, path) {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("Failed to save memory: {}", e);
                Err(e)
            }
        }
    }

    /// Loads a mind state from the specified path.
    pub fn load(&mut self, path: &str) -> Result<(), std::io::Error> {
        info!("Loading memory from {}", path);
        match memory::load(path) {
            Ok(core) => {
                self.cognition = core;
                Ok(())
            }
            Err(e) => {
                error!("Failed to load memory: {}", e);
                Err(e)
            }
        }
    }
}
