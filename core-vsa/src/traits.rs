use crate::{HyperVector, SymbolGraph};
use std::path::Path;
use std::error::Error;

/// Interface for ingesting data from various sources.
pub trait Ingestor {
    /// Ingests data from a given path and returns a SymbolGraph.
    ///
    /// # Arguments
    /// * `path` - The path to the file or directory to ingest.
    fn ingest(&self, path: &Path) -> Result<SymbolGraph, Box<dyn Error + Send + Sync>>;
}

/// Interface for memory storage systems.
pub trait MemoryStore {
    fn store(&mut self, key: &str, vector: HyperVector) -> Result<(), Box<dyn Error>>;
    fn retrieve(&self, key: &str) -> Option<HyperVector>;
    fn query_nearest(&self, query: &HyperVector, k: usize) -> Vec<(String, f32)>;
}

/// Interface for cognitive modules (reasoning, sequence, etc.).
pub trait CognitiveModule {
    fn name(&self) -> &str;
    fn process(&mut self, input: &HyperVector) -> Result<HyperVector, Box<dyn Error>>;
}

/// Interface for neural perception modules.
pub trait PerceptionBackend {
    fn encode_image(&self, image_path: &Path) -> Result<HyperVector, Box<dyn Error>>;
    fn encode_audio(&self, audio_path: &Path) -> Result<HyperVector, Box<dyn Error>>;
    fn encode_text(&self, text: &str) -> Result<HyperVector, Box<dyn Error>>;
}
