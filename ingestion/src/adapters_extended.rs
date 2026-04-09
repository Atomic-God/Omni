use crate::{IngestionAdapter, SemanticChunk, chunk_content_with_structure};
use std::path::Path;
use tracing::warn;
use csv::ReaderBuilder;

// --- Spreadsheet Adapter ---
pub struct SpreadsheetAdapter;
impl IngestionAdapter for SpreadsheetAdapter {
    fn can_handle(&self, path: &Path) -> bool {
        matches!(path.extension().and_then(|s| s.to_str()), Some("csv" | "tsv"))
    }
    fn ingest(&self, path: &Path) -> Vec<SemanticChunk> {
        let delimiter = if path.extension().and_then(|s| s.to_str()) == Some("tsv") { b'\t' } else { b',' };

        let mut chunks = Vec::new();
        if let Ok(mut rdr) = ReaderBuilder::new().delimiter(delimiter).from_path(path) {
            if let Ok(headers) = rdr.headers() {
                let header_str = headers.iter().collect::<Vec<&str>>().join(" | ");
                chunks.extend(chunk_content_with_structure(&header_str, "spreadsheet_header", path, "header"));

                for result in rdr.records() {
                    if let Ok(record) = result {
                        let row_str = record.iter().collect::<Vec<&str>>().join(" | ");
                        chunks.extend(chunk_content_with_structure(&row_str, "spreadsheet_row", path, "row"));
                    }
                }
            }
        } else {
            warn!("Failed to parse spreadsheet: {:?}", path);
        }
        chunks
    }
}

// --- Presentation Adapter ---
pub struct PresentationAdapter;
impl IngestionAdapter for PresentationAdapter {
    fn can_handle(&self, path: &Path) -> bool {
        matches!(path.extension().and_then(|s| s.to_str()), Some("pptx" | "ppt"))
    }
    fn ingest(&self, _path: &Path) -> Vec<SemanticChunk> {
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
