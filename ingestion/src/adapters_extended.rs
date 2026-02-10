use crate::{IngestionAdapter, SemanticChunk, chunk_content_with_structure};
use std::path::Path;
use log::warn;

// --- Spreadsheet Adapter ---
pub struct SpreadsheetAdapter;
impl IngestionAdapter for SpreadsheetAdapter {
    fn can_handle(&self, path: &Path) -> bool {
        matches!(path.extension().and_then(|s| s.to_str()), Some("csv" | "tsv" | "xlsx" | "xls"))
    }
    fn ingest(&self, path: &Path) -> Vec<SemanticChunk> {
        // Stub for spreadsheet ingestion.
        // In a full impl, this would use `csv` crate or `calamine`.
        // We will read as text for CSV/TSV, and stub for XLSX.

        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
        if ext == "csv" || ext == "tsv" {
             // Basic text read
             if let Ok(content) = std::fs::read_to_string(path) {
                 return chunk_content_with_structure(&content, "spreadsheet", path, "table");
             }
        }

        warn!("Spreadsheet ingestion for {} is strictly text-based or stubbed.", ext);
        vec![]
    }
}

// --- Presentation Adapter ---
pub struct PresentationAdapter;
impl IngestionAdapter for PresentationAdapter {
    fn can_handle(&self, path: &Path) -> bool {
        matches!(path.extension().and_then(|s| s.to_str()), Some("pptx" | "ppt"))
    }
    fn ingest(&self, path: &Path) -> Vec<SemanticChunk> {
        warn!("Presentation ingestion (PPTX) is a stub. Requires zip+xml parsing similar to DOCX.");
        vec![]
    }
}

// --- Image Adapter (OCR Stub) ---
pub struct ImageAdapter;
impl IngestionAdapter for ImageAdapter {
    fn can_handle(&self, path: &Path) -> bool {
        matches!(path.extension().and_then(|s| s.to_str()), Some("png" | "jpg" | "jpeg" | "webp" | "tiff" | "bmp"))
    }
    fn ingest(&self, path: &Path) -> Vec<SemanticChunk> {
        warn!("Image ingestion requires OCR (GPU deferred). Skipping: {:?}", path);
        // We could return a "Image placeholder" chunk?
        vec![SemanticChunk {
            source: path.to_string_lossy().to_string(),
            content: "[Image Content: OCR Required]".to_string(),
            metadata: crate::ChunkMetadata {
                hash: crate::compute_hash("[Image Content: OCR Required]"),
                timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs(),
                file_type: "image".to_string(),
                language: "visual".to_string(),
                structure_type: "image_frame".to_string(),
            }
        }]
    }
}
