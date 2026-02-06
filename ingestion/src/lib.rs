use std::path::PathBuf;
use walkdir::WalkDir;
use serde::{Serialize, Deserialize};
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Read};
use log::{info, warn, error};
use sha2::{Sha256, Digest};
use std::collections::HashSet;
use std::time::{SystemTime, UNIX_EPOCH};
use whatlang::{detect, Lang};
use regex::Regex;
use once_cell::sync::Lazy;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkMetadata {
    pub hash: String,
    pub timestamp: u64,
    pub file_type: String,
    pub language: String,
    pub structure_type: String, // e.g., "function", "class", "text_block"
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
    info!("Ingestion Pipeline: Scanning {:?}", path);

    if path.is_file() {
        process_file(&path, &mut chunks, &mut seen_hashes);
    } else {
        for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
            if entry.path().is_file() {
                process_file(entry.path(), &mut chunks, &mut seen_hashes);
            }
        }
    }
    info!("Ingestion Pipeline: Ingested {} unique structural chunks.", chunks.len());
    chunks
}

pub fn ingest_stream(source_name: &str, reader: &mut dyn BufRead) -> Vec<SemanticChunk> {
    let mut chunks = Vec::new();
    let mut content = String::new();
    if let Err(e) = reader.read_to_string(&mut content) {
        error!("Stream ingestion failed: {}", e);
        return chunks;
    }

    let hash = compute_hash(&content);
    let lang = detect(&content).map(|info| info.lang().to_string()).unwrap_or_else(|| "unknown".to_string());

    // Chunking for stream
    let file_chunks = chunk_content(&content, "stream");
    for (chunk_text, struct_type) in file_chunks {
         chunks.push(SemanticChunk {
             source: source_name.to_string(),
             content: chunk_text,
             metadata: ChunkMetadata {
                 hash: hash.clone(),
                 timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs(),
                 file_type: "stream".to_string(),
                 language: lang.clone(),
                 structure_type: struct_type,
             },
         });
    }
    chunks
}

fn process_file(path: &std::path::Path, chunks: &mut Vec<SemanticChunk>, seen_hashes: &mut HashSet<String>) {
    if let Some(ext) = path.extension() {
        let ext_str = ext.to_string_lossy().to_lowercase();

        match ext_str.as_str() {
            "txt" | "md" | "rs" | "csv" | "json" | "py" | "c" | "cpp" | "h" | "toml" | "yaml" | "xml" => {
                 match read_file_stream(path) {
                    Ok(content) => {
                         let lang = detect(&content).map(|info| info.lang().to_string()).unwrap_or_else(|| "unknown".to_string());

                         if lang == "unknown" && is_likely_binary(&content) {
                             warn!("Skipping likely binary file: {:?}", path);
                             return;
                         }

                         let file_chunks = chunk_content(&content, &ext_str);
                         for (chunk_text, struct_type) in file_chunks {
                             let hash = compute_hash(&chunk_text);
                             if seen_hashes.contains(&hash) {
                                 continue; // Deduplicate
                             }
                             seen_hashes.insert(hash.clone());

                             chunks.push(SemanticChunk {
                                 source: path.to_string_lossy().to_string(),
                                 content: chunk_text,
                                 metadata: ChunkMetadata {
                                     hash,
                                     timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs(),
                                     file_type: ext_str.clone(),
                                     language: lang.clone(),
                                     structure_type: struct_type,
                                 },
                             });
                         }
                    },
                    Err(e) => warn!("Failed to read file {:?}: {}", path, e),
                 }
            },
            "pdf" => {
                warn!("PDF support requires external dependencies. Skipping: {:?}", path);
            },
            _ => {
                // Skip unknown
            }
        }
    }
}

fn read_file_stream(path: &std::path::Path) -> std::io::Result<String> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut content = String::new();
    let metadata = fs::metadata(path)?;
    if metadata.len() > 10 * 1024 * 1024 {
        warn!("File {:?} is too large ({:?} bytes). Truncating.", path, metadata.len());
        reader.take(10 * 1024 * 1024).read_to_string(&mut content)?;
    } else {
        reader.read_to_string(&mut content)?;
    }
    Ok(content)
}

fn is_likely_binary(text: &str) -> bool {
    text.chars().take(100).filter(|c| c.is_control() && !c.is_whitespace()).count() > 5
}

static RUST_FN_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"fn\s+(\w+)").unwrap());
static PY_DEF_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"def\s+(\w+)").unwrap());
static C_FN_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\w+\s+(\w+)\s*\(").unwrap());

fn chunk_content(text: &str, file_type: &str) -> Vec<(String, String)> {
    // Structural extraction
    match file_type {
        "rs" => chunk_code(text, &RUST_FN_REGEX),
        "py" => chunk_code(text, &PY_DEF_REGEX),
        "c" | "cpp" | "h" => chunk_code(text, &C_FN_REGEX),
        _ => chunk_text_default(text),
    }
}

fn chunk_code(text: &str, regex: &Regex) -> Vec<(String, String)> {
    // Naive code chunking: Split by double newlines, then identify if chunk defines a function.
    let paragraphs: Vec<&str> = text.split("\n\n").collect();
    let mut chunks = Vec::new();

    for para in paragraphs {
        let trimmed = para.trim();
        if trimmed.is_empty() { continue; }

        let struct_type = if let Some(caps) = regex.captures(trimmed) {
            format!("function:{}", &caps[1])
        } else {
            "code_block".to_string()
        };

        chunks.push((trimmed.to_string(), struct_type));
    }
    chunks
}

fn chunk_text_default(text: &str) -> Vec<(String, String)> {
    let max_chunk_size = 2000;
    let paragraphs: Vec<&str> = text.split("\n\n").collect();
    let mut chunks = Vec::new();
    let mut current_chunk = String::new();

    for para in paragraphs {
        let trimmed = para.trim();
        if trimmed.is_empty() { continue; }

        if current_chunk.len() + trimmed.len() > max_chunk_size {
            if !current_chunk.is_empty() {
                chunks.push((current_chunk.clone(), "text_block".to_string()));
                current_chunk.clear();
            }
        }

        if !current_chunk.is_empty() {
            current_chunk.push_str("\n\n");
        }
        current_chunk.push_str(trimmed);
    }

    if !current_chunk.is_empty() {
        chunks.push((current_chunk, "text_block".to_string()));
    }

    chunks
}

fn compute_hash(text: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(text);
    hex::encode(hasher.finalize())
}
