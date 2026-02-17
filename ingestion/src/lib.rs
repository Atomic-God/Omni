#![deny(warnings)]
use core_vsa::traits::Ingestor;
use core_vsa::SymbolGraph;
use std::path::Path;
use std::error::Error;

pub mod adapters;
pub mod adapters_extended;
pub mod media_adapters;
pub mod fallback;
pub mod registry;
pub mod universal;
pub mod metadata;
pub mod layout;
pub mod ocr;
pub mod video;
pub mod vision;
pub mod nlp;
pub mod code_analysis;
pub mod data_meaning;
pub mod audio_meaning;

pub use registry::DataIngestionRegistry;
pub use universal::UniversalIngestor;
pub use universal::UniversalAdapter;

pub use adapters::JsonAdapter;
pub use fallback::SymbolExtractor;

// Main entry point for Universal Ingestion
pub fn ingest_graph(path: &Path) -> Result<SymbolGraph, Box<dyn Error + Send + Sync>> {
    let ingestor = UniversalIngestor::new();
    ingestor.ingest(path)
}

pub fn chunk_content(content: &str, type_hint: &str, path: &Path) -> Vec<SemanticChunk> {
    vec![SemanticChunk {
        source: path.to_string_lossy().to_string(),
        content: content.to_string(),
        metadata: ChunkMetadata {
            hash: compute_hash(content),
            timestamp: 0,
            file_type: type_hint.to_string(),
            language: "unknown".to_string(),
            structure_type: "text".to_string(),
        }
    }]
}

pub fn chunk_content_with_structure(content: &str, type_hint: &str, path: &Path, structure: &str) -> Vec<SemanticChunk> {
    vec![SemanticChunk {
        source: path.to_string_lossy().to_string(),
        content: content.to_string(),
        metadata: ChunkMetadata {
            hash: compute_hash(content),
            timestamp: 0,
            file_type: type_hint.to_string(),
            language: "unknown".to_string(),
            structure_type: structure.to_string(),
        }
    }]
}

pub fn compute_hash(s: &str) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(s.as_bytes());
    hex::encode(hasher.finalize())
}

use serde::{Serialize, Deserialize};

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

pub fn generic_read_file(path: &std::path::Path, type_hint: &str) -> Vec<SemanticChunk> {
    use std::fs::File;
    use std::io::Read;

    let mut file = match File::open(path) { Ok(f) => f, Err(_) => return vec![] };
    let mut buffer = String::new();
    if file.read_to_string(&mut buffer).is_err() { return vec![]; }

    vec![SemanticChunk {
        source: path.to_string_lossy().to_string(),
        content: buffer.clone(),
        metadata: ChunkMetadata {
            hash: compute_hash(&buffer),
            timestamp: 0,
            file_type: type_hint.to_string(),
            language: "unknown".to_string(),
            structure_type: "text".to_string(),
        }
    }]
}
