use core_vsa::traits::Ingestor;
use core_vsa::{SymbolGraph, HyperVector};
use std::path::Path;
use std::error::Error;
use log::info;

pub mod adapters;
pub mod adapters_extended;
pub mod media_adapters;
pub mod fallback;
pub mod registry;
pub mod universal;

pub use registry::DataIngestionRegistry;
pub use universal::UniversalIngestor;
pub use universal::UniversalAdapter; // Legacy support

// Re-export specific structs if needed for tests
pub use adapters::JsonAdapter;
pub use fallback::SymbolExtractor;

// Main entry point for Universal Ingestion
pub fn ingest_graph(path: &Path) -> Result<SymbolGraph, Box<dyn Error + Send + Sync>> {
    let ingestor = UniversalIngestor;
    ingestor.ingest(path)
}

// Legacy functions kept for backward compatibility (wrapped)
pub use universal::process_single_file; // Helper exposed? No, implementation detail.

use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use std::collections::HashSet;

pub trait IngestionAdapter: Send + Sync {
    fn can_handle(&self, path: &std::path::Path) -> bool;
    fn ingest(&self, path: &std::path::Path) -> Vec<SemanticChunk>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkMetadata {
    pub hash: String,
    pub timestamp: u64,
    pub file_type: String,
    pub language: String,
    pub structure_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticChunk {
    pub source: String,
    pub content: String,
    pub metadata: ChunkMetadata,
}

// ... helper functions for legacy adapters ...
// (We keep generic_read_file etc. for `adapters` crate usage)

pub fn generic_read_file(path: &std::path::Path, type_hint: &str) -> Vec<SemanticChunk> {
    // Stub implementation to satisfy legacy code linking
    // Real implementation would read file.
    use std::fs::File;
    use std::io::Read;

    let mut file = match File::open(path) { Ok(f) => f, Err(_) => return vec![] };
    let mut buffer = String::new();
    if file.read_to_string(&mut buffer).is_err() { return vec![]; }

    vec![SemanticChunk {
        source: path.to_string_lossy().to_string(),
        content: buffer.clone(),
        metadata: ChunkMetadata {
            hash: "stub".to_string(),
            timestamp: 0,
            file_type: type_hint.to_string(),
            language: "unknown".to_string(),
            structure_type: "text".to_string(),
        }
    }]
}
