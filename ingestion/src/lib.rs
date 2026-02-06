use std::path::PathBuf;
use walkdir::WalkDir;
use serde::{Serialize, Deserialize};
use std::fs;
use log::{info, warn};
use sha2::{Sha256, Digest};
use std::collections::HashSet;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkMetadata {
    pub hash: String,
    pub timestamp: u64,
    pub file_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticChunk {
    pub source: String,
    pub content: String,
    pub metadata: ChunkMetadata,
}

pub fn ingest_path(path: PathBuf) -> Vec<SemanticChunk> {
    let mut chunks = Vec::new();
    let mut seen_hashes = HashSet::new();
    info!("Ingesting path: {:?}", path);

    for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        if entry.path().is_file() {
            if let Some(ext) = entry.path().extension() {
                let ext_str = ext.to_string_lossy().to_lowercase();

                let (content, file_type) = match ext_str.as_str() {
                    "txt" | "md" | "rs" => {
                        (fs::read_to_string(entry.path()).ok(), ext_str.clone())
                    },
                    "json" => {
                        (read_json_as_text(entry.path()), "json".to_string())
                    },
                    "csv" => {
                        (fs::read_to_string(entry.path()).ok(), "csv".to_string())
                    },
                    "pdf" => {
                        warn!("PDF ingestion not yet implemented for: {:?}", entry.path());
                        (None, "pdf".to_string())
                    },
                    _ => (None, "unknown".to_string()),
                };

                if let Some(text) = content {
                    let file_chunks = chunk_text(&text);
                    for chunk_text in file_chunks {
                        let hash = compute_hash(&chunk_text);
                        if seen_hashes.contains(&hash) {
                            continue; // Deduplicate
                        }
                        seen_hashes.insert(hash.clone());

                        chunks.push(SemanticChunk {
                            source: entry.path().to_string_lossy().to_string(),
                            content: chunk_text,
                            metadata: ChunkMetadata {
                                hash,
                                timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs(),
                                file_type: file_type.clone(),
                            },
                        });
                    }
                }
            }
        }
    }
    info!("Ingested {} unique chunks.", chunks.len());
    chunks
}

fn chunk_text(text: &str) -> Vec<String> {
    let max_chunk_size = 1000;
    let paragraphs: Vec<&str> = text.split("\n\n").collect();
    let mut chunks = Vec::new();
    let mut current_chunk = String::new();

    for para in paragraphs {
        let trimmed = para.trim();
        if trimmed.is_empty() { continue; }

        if current_chunk.len() + trimmed.len() > max_chunk_size {
            if !current_chunk.is_empty() {
                chunks.push(current_chunk.clone());
                current_chunk.clear();
            }
        }

        if !current_chunk.is_empty() {
            current_chunk.push_str("\n\n");
        }
        current_chunk.push_str(trimmed);
    }

    if !current_chunk.is_empty() {
        chunks.push(current_chunk);
    }

    chunks
}

fn compute_hash(text: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(text);
    hex::encode(hasher.finalize())
}

fn read_json_as_text(path: &std::path::Path) -> Option<String> {
    let content = fs::read_to_string(path).ok()?;
    // Naive flatten: just use the raw JSON string.
    // In a real system, we might want to extract values.
    // For now, raw JSON provides context.
    Some(content)
}

pub struct DirectoryWatcher {
    // Legacy watcher logic can remain or be updated
}

impl DirectoryWatcher {
    pub fn new(_path: String, _mind: std::sync::Arc<std::sync::Mutex<engine::OmniMind>>) -> notify::Result<Self> {
        Ok(Self{})
    }
}
