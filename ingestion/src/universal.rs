use crate::{IngestionAdapter, SemanticChunk, ChunkMetadata, compute_hash};
use std::path::Path;
use std::fs::File;
use std::io::Read;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct UniversalAdapter;

impl IngestionAdapter for UniversalAdapter {
    fn can_handle(&self, _path: &Path) -> bool {
        true // Catch-all
    }

    fn ingest(&self, path: &Path) -> Vec<SemanticChunk> {
        let mut chunks = Vec::new();

        let mut file = match File::open(path) {
            Ok(f) => f,
            Err(_) => return vec![],
        };

        let metadata = match file.metadata() {
            Ok(m) => m,
            Err(_) => return vec![],
        };

        // Limit read to 10MB to avoid OOM
        let len = metadata.len();
        let read_len = if len > 10 * 1024 * 1024 { 10 * 1024 * 1024 } else { len as usize };
        let mut buffer = vec![0; read_len];
        if file.read_exact(&mut buffer).is_err() && buffer.is_empty() {
            return vec![];
        }

        // 1. Attempt Text (UTF-8)
        if let Ok(text) = std::str::from_utf8(&buffer) {
            // It's valid text, return as generic text chunk
            chunks.push(SemanticChunk {
                source: path.to_string_lossy().to_string(),
                content: text.to_string(),
                metadata: ChunkMetadata {
                    hash: compute_hash(text),
                    timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs(),
                    file_type: "text/plain".to_string(),
                    language: "unknown".to_string(),
                    structure_type: "generic_text".to_string(),
                }
            });
            return chunks;
        }

        // 2. Binary Analysis
        let header_hex = buffer.iter().take(16).map(|b| format!("{:02X}", b)).collect::<Vec<String>>().join(" ");
        let entropy = calculate_entropy(&buffer);

        let analysis = format!(
            "File Analysis: {:?}\nSize: {} bytes\nMagic Bytes: {}\nEntropy: {:.4} (High=Compressed/Encrypted)\n",
            path.file_name().unwrap_or_default(),
            len,
            header_hex,
            entropy
        );

        chunks.push(SemanticChunk {
            source: path.to_string_lossy().to_string(),
            content: analysis.clone(),
            metadata: ChunkMetadata {
                hash: compute_hash(&analysis),
                timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs(),
                file_type: "application/octet-stream".to_string(),
                language: "binary_analysis".to_string(),
                structure_type: "file_metadata".to_string(),
            }
        });

        // 3. Strings Extraction (printable ASCII > 4 chars)
        let extracted_strings = extract_strings(&buffer);
        if !extracted_strings.is_empty() {
            let content = format!("Extracted Strings:\n{}", extracted_strings.join("\n"));
            chunks.push(SemanticChunk {
                source: path.to_string_lossy().to_string(),
                content: content.clone(),
                metadata: ChunkMetadata {
                    hash: compute_hash(&content),
                    timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs(),
                    file_type: "application/octet-stream".to_string(),
                    language: "strings".to_string(),
                    structure_type: "extracted_text".to_string(),
                }
            });
        }

        chunks
    }
}

fn calculate_entropy(data: &[u8]) -> f32 {
    let mut counts = [0usize; 256];
    for &b in data {
        counts[b as usize] += 1;
    }
    let total = data.len() as f32;
    let mut entropy = 0.0;
    for &count in &counts {
        if count > 0 {
            let p = count as f32 / total;
            entropy -= p * p.log2();
        }
    }
    entropy
}

fn extract_strings(data: &[u8]) -> Vec<String> {
    let mut strings = Vec::new();
    let mut current_string = String::new();

    for &b in data {
        // Printable ASCII range (approx)
        if b >= 32 && b <= 126 {
            current_string.push(b as char);
        } else {
            if current_string.len() >= 4 {
                strings.push(current_string.clone());
            }
            current_string.clear();
        }
    }
    if current_string.len() >= 4 {
        strings.push(current_string);
    }

    // Limit to reasonable amount
    if strings.len() > 1000 {
        strings.truncate(1000);
        strings.push("... (truncated)".to_string());
    }
    strings
}
