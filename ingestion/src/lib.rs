use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use std::fs::{self, File};
use std::io::{BufReader, Read};
use log::{info, warn, error};
use sha2::{Sha256, Digest};
use std::collections::HashSet;
use std::time::{SystemTime, UNIX_EPOCH};
use whatlang::{detect};
use regex::Regex;
use once_cell::sync::Lazy;
use zip::read::ZipArchive;
use tar::Archive;
use flate2::read::GzDecoder;

pub mod adapters;
pub mod adapters_extended; // Added
pub mod fallback;
pub mod registry;

use adapters::{HtmlAdapter, DocxAdapter, MarkdownAdapter, JsonAdapter, PdfAdapterStub};
use adapters_extended::{SpreadsheetAdapter, PresentationAdapter, ImageAdapter};
use fallback::{SymbolExtractor, SymbolOnlyFallback};
use registry::DataIngestionRegistry;

// --- Trait Definitions ---

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
    pub structure_type: String, // e.g. "section:Introduction", "function:main"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticChunk {
    pub source: String,
    pub content: String,
    pub metadata: ChunkMetadata,
}

// --- Default Adapters ---

pub struct TextAdapter;
impl IngestionAdapter for TextAdapter {
    fn can_handle(&self, path: &std::path::Path) -> bool {
        matches!(path.extension().and_then(|s| s.to_str()), Some("txt" | "log" | "yml" | "toml"))
    }
    fn ingest(&self, path: &std::path::Path) -> Vec<SemanticChunk> {
        generic_read_file(path, "text")
    }
}

pub struct CodeAdapter;
impl IngestionAdapter for CodeAdapter {
    fn can_handle(&self, path: &std::path::Path) -> bool {
        matches!(path.extension().and_then(|s| s.to_str()), Some("rs" | "py" | "c" | "cpp" | "h" | "js" | "ts" | "java" | "go"))
    }
    fn ingest(&self, path: &std::path::Path) -> Vec<SemanticChunk> {
        let content = match read_file_stream(path) { Ok(c) => c, Err(_) => return vec![] };
        let ext = path.extension().unwrap().to_str().unwrap();
        chunk_content(&content, ext, path)
    }
}

pub struct StructureAdapterStub; // Renamed, now using JsonAdapter for JSON
impl IngestionAdapter for StructureAdapterStub {
    fn can_handle(&self, path: &std::path::Path) -> bool {
        matches!(path.extension().and_then(|s| s.to_str()), Some("csv" | "xml"))
    }
    fn ingest(&self, path: &std::path::Path) -> Vec<SemanticChunk> {
        generic_read_file(path, "structured")
    }
}

// --- Universal Ingestion Pipeline ---

pub fn ingest_path(path: PathBuf) -> Vec<SemanticChunk> {
    let mut chunks = Vec::new();
    let mut seen_hashes = HashSet::new();

    // Initialize Registry
    let mut registry = DataIngestionRegistry::new();
    registry.register(MarkdownAdapter);
    registry.register(JsonAdapter);
    registry.register(HtmlAdapter);
    registry.register(DocxAdapter);
    registry.register(TextAdapter);
    registry.register(CodeAdapter);
    registry.register(StructureAdapterStub);
    registry.register(PdfAdapterStub);
    // Extended
    registry.register(SpreadsheetAdapter);
    registry.register(PresentationAdapter);
    registry.register(ImageAdapter);

    info!("Ingestion Pipeline: Scanning {:?}", path);

    if path.is_file() {
        process_file(&path, &mut chunks, &mut seen_hashes, &registry);
    } else {
        for entry in walkdir::WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
            let p = entry.path();
            if p.to_string_lossy().contains("/.git/") || p.to_string_lossy().contains(".git/") {
                continue;
            }
            if p.is_file() {
                process_file(p, &mut chunks, &mut seen_hashes, &registry);
            }
        }
    }
    info!("Ingestion Pipeline: Ingested {} unique structural chunks.", chunks.len());
    chunks
}

fn process_file(path: &std::path::Path, chunks: &mut Vec<SemanticChunk>, seen_hashes: &mut HashSet<String>, registry: &DataIngestionRegistry) {
    // Try registry first
    let new_chunks = registry.ingest(path);
    if !new_chunks.is_empty() {
        for chunk in new_chunks {
            if !seen_hashes.contains(&chunk.metadata.hash) {
                seen_hashes.insert(chunk.metadata.hash.clone());
                chunks.push(chunk);
            }
        }
        return;
    }

    // Fallback logic for archives (Recursive)
    if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
        match ext {
            "zip" => process_zip(path, chunks, seen_hashes),
            "tar" => process_tar(path, chunks, seen_hashes),
            "gz" => process_tar_gz(path, chunks, seen_hashes),
            _ => {
                // FALLBACK: Treat as opaque binary/text symbols
                let fallback_content = generic_read_file_fallback(path);
                if !fallback_content.is_empty() {
                     let hash = compute_hash(&fallback_content);
                     if !seen_hashes.contains(&hash) {
                         seen_hashes.insert(hash.clone());
                         chunks.push(SemanticChunk {
                             source: path.to_string_lossy().to_string(),
                             content: fallback_content,
                             metadata: ChunkMetadata {
                                 hash,
                                 timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs(),
                                 file_type: "unknown".to_string(),
                                 language: "symbolic".to_string(),
                                 structure_type: "blob".to_string(),
                             },
                         });
                     }
                }
            }
        }
    }
}

// --- Helper Functions ---

fn generic_read_file(path: &std::path::Path, type_hint: &str) -> Vec<SemanticChunk> {
    match read_file_stream(path) {
        Ok(content) => chunk_content(&content, type_hint, path),
        Err(e) => {
            warn!("Failed to read {:?}: {}", path, e);
            vec![]
        }
    }
}

fn generic_read_file_fallback(path: &std::path::Path) -> String {
    let mut file = match File::open(path) { Ok(f) => f, Err(_) => return String::new() };
    let mut buffer = Vec::new();
    let metadata = match fs::metadata(path) { Ok(m) => m, Err(_) => return String::new() };

    if metadata.len() > 10 * 1024 * 1024 {
        let _ = file.take(10 * 1024 * 1024).read_to_end(&mut buffer);
    } else {
        let _ = file.read_to_end(&mut buffer);
    }

    SymbolExtractor.extract_symbols(&buffer)
}

fn read_file_stream(path: &std::path::Path) -> std::io::Result<String> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut buffer = Vec::new();
    let metadata = fs::metadata(path)?;

    // Safety limit: 10MB
    if metadata.len() > 10 * 1024 * 1024 {
        warn!("File {:?} is too large. Truncating to 10MB.", path);
        reader.take(10 * 1024 * 1024).read_to_end(&mut buffer)?;
    } else {
        reader.read_to_end(&mut buffer)?;
    }

    // Attempt UTF-8 conversion, replace invalid sequences (Lossy)
    let content = String::from_utf8_lossy(&buffer).to_string();
    Ok(content)
}

pub(crate) fn chunk_content(text: &str, file_type: &str, path: &std::path::Path) -> Vec<SemanticChunk> {
    chunk_content_with_structure(text, file_type, path, "root")
}

pub(crate) fn chunk_content_with_structure(text: &str, file_type: &str, path: &std::path::Path, structure_hint: &str) -> Vec<SemanticChunk> {
    let lang = detect(text).map(|info| info.lang().to_string()).unwrap_or_else(|| "unknown".to_string());

    // Binary check (Heuristic: many control chars)
    if lang == "unknown" && text.chars().take(100).filter(|c| c.is_control() && !c.is_whitespace()).count() > 5 {
        return vec![];
    }

    let raw_chunks = match file_type {
        "rs" => chunk_code(text, &RUST_FN_REGEX),
        "py" => chunk_code(text, &PY_DEF_REGEX),
        _ => chunk_text_smart(text),
    };

    let mut chunks = Vec::new();
    for (chunk_text, struct_type) in raw_chunks {
        // If the chunker returns generic "code_block" or "sentence_group", we can append the hint
        let final_struct_type = if struct_type == "sentence_group" || struct_type == "code_block" {
             if structure_hint != "root" {
                 format!("{}:{}", structure_hint, struct_type)
             } else {
                 struct_type
             }
        } else {
             // chunker found something specific (e.g. function:foo)
             struct_type
        };

        chunks.push(SemanticChunk {
            source: path.to_string_lossy().to_string(),
            content: chunk_text.clone(),
            metadata: ChunkMetadata {
                hash: compute_hash(&chunk_text),
                timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs(),
                file_type: file_type.to_string(),
                language: lang.clone(),
                structure_type: final_struct_type,
            },
        });
    }
    chunks
}

static RUST_FN_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"fn\s+(\w+)").unwrap());
static PY_DEF_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"def\s+(\w+)").unwrap());

fn chunk_code(text: &str, regex: &Regex) -> Vec<(String, String)> {
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

fn chunk_text_smart(text: &str) -> Vec<(String, String)> {
    let re = Regex::new(r"[.?!]\s+").unwrap();
    let sentences: Vec<&str> = re.split(text).collect();

    let mut chunks = Vec::new();
    let mut current_chunk = String::new();
    let max_chunk_size = 1000;

    for sent in sentences {
        let trimmed = sent.trim();
        if trimmed.is_empty() { continue; }

        if current_chunk.len() + trimmed.len() > max_chunk_size {
            if !current_chunk.is_empty() {
                chunks.push((current_chunk.clone(), "sentence_group".to_string()));
                current_chunk.clear();
            }
        }

        if !current_chunk.is_empty() {
            current_chunk.push_str(" ");
        }
        current_chunk.push_str(trimmed);
    }

    if !current_chunk.is_empty() {
        chunks.push((current_chunk, "sentence_group".to_string()));
    }
    chunks
}

pub(crate) fn compute_hash(text: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(text);
    hex::encode(hasher.finalize())
}

// Archive processing (legacy style, kept for now)
fn process_zip(path: &std::path::Path, chunks: &mut Vec<SemanticChunk>, seen_hashes: &mut HashSet<String>) {
    let file = match File::open(path) { Ok(f) => f, Err(e) => { error!("Failed to open zip: {}", e); return; } };
    let mut archive = match ZipArchive::new(file) { Ok(a) => a, Err(e) => { error!("Failed to parse zip: {}", e); return; } };

    for i in 0..archive.len() {
        let mut file = match archive.by_index(i) { Ok(f) => f, Err(_) => continue };
        if file.is_dir() { continue; }

        // Simple heuristic: process only text-like extensions inside zip
        let name = file.name().to_string();
        if !name.ends_with(".txt") && !name.ends_with(".md") && !name.ends_with(".rs") && !name.ends_with(".json") {
            continue;
        }

        let mut content = String::new();
        if file.read_to_string(&mut content).is_ok() {
             let file_chunks = chunk_content(&content, "zip_entry", path);
             for chunk in file_chunks {
                 if seen_hashes.contains(&chunk.metadata.hash) { continue; }
                 seen_hashes.insert(chunk.metadata.hash.clone());
                 chunks.push(chunk);
             }
        }
    }
}

fn process_tar(path: &std::path::Path, chunks: &mut Vec<SemanticChunk>, seen_hashes: &mut HashSet<String>) {
    let file = match File::open(path) { Ok(f) => f, Err(e) => { error!("Failed to open tar: {}", e); return; } };
    let mut archive = Archive::new(file);
    process_tar_entries(&mut archive, path, chunks, seen_hashes);
}

fn process_tar_gz(path: &std::path::Path, chunks: &mut Vec<SemanticChunk>, seen_hashes: &mut HashSet<String>) {
    let file = match File::open(path) { Ok(f) => f, Err(e) => { error!("Failed to open tar.gz: {}", e); return; } };
    let tar = GzDecoder::new(file);
    let mut archive = Archive::new(tar);
    process_tar_entries(&mut archive, path, chunks, seen_hashes);
}

fn process_tar_entries<R: Read>(archive: &mut Archive<R>, source_path: &std::path::Path, chunks: &mut Vec<SemanticChunk>, seen_hashes: &mut HashSet<String>) {
    if let Ok(entries) = archive.entries() {
        for entry in entries {
            if let Ok(mut file) = entry {
                let path_buf = match file.path() { Ok(p) => p.to_path_buf(), Err(_) => continue };
                let name = path_buf.to_string_lossy();

                if !name.ends_with(".txt") && !name.ends_with(".md") && !name.ends_with(".rs") && !name.ends_with(".json") {
                    continue;
                }

                let mut content = String::new();
                if file.read_to_string(&mut content).is_ok() {
                     let file_chunks = chunk_content(&content, "tar_entry", source_path);
                     for chunk in file_chunks {
                         if seen_hashes.contains(&chunk.metadata.hash) { continue; }
                         seen_hashes.insert(chunk.metadata.hash.clone());
                         chunks.push(chunk);
                     }
                }
            }
        }
    }
}
