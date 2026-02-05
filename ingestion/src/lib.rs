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
    // Naive paragraph splitter
    text.split("\n\n")
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
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
