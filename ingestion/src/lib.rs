use std::path::PathBuf;
use walkdir::WalkDir;
use serde::{Serialize, Deserialize};
use std::fs;
use log::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticChunk {
    pub source: String,
    pub content: String,
}

pub fn ingest_path(path: PathBuf) -> Vec<SemanticChunk> {
    let mut chunks = Vec::new();
    info!("Ingesting path: {:?}", path);

    for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        if entry.path().is_file() {
            if let Some(ext) = entry.path().extension() {
                let ext_str = ext.to_string_lossy();
                let content = match ext_str.as_ref() {
                    "txt" | "md" | "json" | "rs" | "csv" => {
                        fs::read_to_string(entry.path()).ok()
                    },
                    // Add PDF support logic here if lopdf added, else skip
                    _ => None,
                };

                if let Some(text) = content {
                    // Simple chunking: split by paragraphs or max size
                    // For prototype: one chunk per file or naive split
                    let file_chunks = chunk_text(&text);
                    for chunk_text in file_chunks {
                        chunks.push(SemanticChunk {
                            source: entry.path().to_string_lossy().to_string(),
                            content: chunk_text,
                        });
                    }
                }
            }
        }
    }
    chunks
}

fn chunk_text(text: &str) -> Vec<String> {
    // Semantic + Size-bounded chunking
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

pub struct DirectoryWatcher {
    // Legacy watcher logic can remain or be updated
}

impl DirectoryWatcher {
    pub fn new(_path: String, _mind: std::sync::Arc<std::sync::Mutex<engine::OmniMind>>) -> notify::Result<Self> {
        // Placeholder to satisfy existing code if any
        Ok(Self{})
    }
}
