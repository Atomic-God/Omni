use std::path::PathBuf;
use walkdir::WalkDir;
use serde::{Serialize, Deserialize};
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Read, Cursor};
use log::{info, warn, error};
use sha2::{Sha256, Digest};
use std::collections::HashSet;
use std::time::{SystemTime, UNIX_EPOCH};
use whatlang::{detect, Lang};
use regex::Regex;
use once_cell::sync::Lazy;
use zip::read::ZipArchive;
use tar::Archive;
use flate2::read::GzDecoder;

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
    let ext_str = path.extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_else(|| "bin".to_string());

    match ext_str.as_str() {
        "zip" | "docx" | "pptx" | "xlsx" => process_zip(path, chunks, seen_hashes, &ext_str), // Office files are zips
        "tar" => process_tar(path, chunks, seen_hashes),
        "gz" => process_tar_gz(path, chunks, seen_hashes),
        "pdf" => process_pdf(path, chunks, seen_hashes),
        _ => process_generic(path, chunks, seen_hashes, &ext_str),
    }
}

fn process_generic(path: &std::path::Path, chunks: &mut Vec<SemanticChunk>, seen_hashes: &mut HashSet<String>, ext: &str) {
    // Try to read as text first
    match read_file_stream(path) {
        Ok(content) => {
             // Heuristic: Is it binary?
             if is_likely_binary(&content) {
                 warn!("Ingesting binary file as blob metadata: {:?}", path);
                 let fact = format!("File {} exists with type {}.", path.file_name().unwrap().to_string_lossy(), ext);
                 push_chunk(chunks, seen_hashes, path.to_string_lossy().to_string(), fact, ext.to_string(), "file_metadata");
                 return;
             }

             let file_chunks = chunk_content(&content, ext);
             for (chunk_text, struct_type) in file_chunks {
                 push_chunk(chunks, seen_hashes, path.to_string_lossy().to_string(), chunk_text, ext.to_string(), &struct_type);
             }
        },
        Err(e) => warn!("Failed to read file {:?}: {}", path, e),
    }
}

fn process_pdf(path: &std::path::Path, chunks: &mut Vec<SemanticChunk>, seen_hashes: &mut HashSet<String>) {
    match pdf_extract::extract_text(path) {
        Ok(text) => {
             let file_chunks = chunk_content(&text, "pdf");
             for (chunk_text, struct_type) in file_chunks {
                 push_chunk(chunks, seen_hashes, path.to_string_lossy().to_string(), chunk_text, "pdf".to_string(), &struct_type);
             }
        },
        Err(e) => {
            warn!("PDF extraction failed for {:?}: {}. Fallback to metadata.", path, e);
            let fact = format!("PDF Document {} exists.", path.file_name().unwrap().to_string_lossy());
            push_chunk(chunks, seen_hashes, path.to_string_lossy().to_string(), fact, "pdf".to_string(), "metadata_fallback");
        }
    }
}

fn process_zip(path: &std::path::Path, chunks: &mut Vec<SemanticChunk>, seen_hashes: &mut HashSet<String>, original_ext: &str) {
    let file = match File::open(path) { Ok(f) => f, Err(e) => { error!("Failed to open zip: {}", e); return; } };
    let mut archive = match ZipArchive::new(file) { Ok(a) => a, Err(e) => { error!("Failed to parse zip: {}", e); return; } };

    for i in 0..archive.len() {
        let mut file = match archive.by_index(i) { Ok(f) => f, Err(_) => continue };
        if file.is_dir() { continue; }

        let name = file.name().to_string();
        // Skip junk
        if name.starts_with("__MACOSX") || name.ends_with(".DS_Store") { continue; }

        // Recursive Office XML extraction could go here (document.xml)
        if (original_ext == "docx" && name == "word/document.xml") ||
           (original_ext == "pptx" && name.starts_with("ppt/slides/slide")) {
               // Extract text from XML (naive strip tags)
               let mut xml = String::new();
               if file.read_to_string(&mut xml).is_ok() {
                   let text = strip_xml_tags(&xml);
                   let file_chunks = chunk_content(&text, "office_xml");
                   for (chunk_text, struct_type) in file_chunks {
                       push_chunk(chunks, seen_hashes, format!("{}::{}", path.to_string_lossy(), name), chunk_text, "office_text".to_string(), &struct_type);
                   }
               }
               continue;
        }

        let mut content = String::new();
        // Try read as text
        if file.read_to_string(&mut content).is_ok() {
             let file_chunks = chunk_content(&content, "zip_entry");
             for (chunk_text, struct_type) in file_chunks {
                 push_chunk(chunks, seen_hashes, format!("{}::{}", path.to_string_lossy(), name), chunk_text, "zip_entry".to_string(), &struct_type);
             }
        }
    }
}

fn strip_xml_tags(xml: &str) -> String {
    let re = Regex::new(r"[.?!]s+").unwrap();
    re.replace_all(xml, " ").to_string()
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

                let mut content = String::new();
                if file.read_to_string(&mut content).is_ok() {
                     let file_chunks = chunk_content(&content, "tar_entry");
                     for (chunk_text, struct_type) in file_chunks {
                         push_chunk(chunks, seen_hashes, format!("{}::{}", source_path.to_string_lossy(), name), chunk_text, "tar_entry".to_string(), &struct_type);
                     }
                }
            }
        }
    }
}

fn push_chunk(chunks: &mut Vec<SemanticChunk>, seen_hashes: &mut HashSet<String>, source: String, content: String, file_type: String, structure_type: &str) {
    let hash = compute_hash(&content);
    if seen_hashes.contains(&hash) { return; }
    seen_hashes.insert(hash.clone());

    let lang = detect(&content).map(|info| info.lang().to_string()).unwrap_or_else(|| "unknown".to_string());

    chunks.push(SemanticChunk {
        source,
        content,
        metadata: ChunkMetadata {
            hash,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs(),
            file_type,
            language: lang,
            structure_type: structure_type.to_string(),
        },
    });
}

fn read_file_stream(path: &std::path::Path) -> std::io::Result<String> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut content = String::new();
    let metadata = fs::metadata(path)?;
    if metadata.len() > 10 * 1024 * 1024 {
        warn!("File {:?} is large ({:?} bytes). Truncating for safety.", path, metadata.len());
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
    match file_type {
        "rs" => chunk_code(text, &RUST_FN_REGEX),
        "py" => chunk_code(text, &PY_DEF_REGEX),
        "c" | "cpp" | "h" => chunk_code(text, &C_FN_REGEX),
        _ => chunk_text_smart(text),
    }
}

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
    let re = Regex::new(r"[.?!]s+").unwrap();
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

fn compute_hash(text: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(text);
    hex::encode(hasher.finalize())
}
