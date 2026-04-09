use crate::{IngestionAdapter, SemanticChunk, ChunkMetadata};
use std::path::Path;
use tracing::warn;
use image::GenericImageView;
use std::time::{SystemTime, UNIX_EPOCH};
use std::fs::File;
use id3::TagLike;

// --- Media Descriptors ---

#[derive(Debug)]
pub struct ImageDescriptor {
    pub width: u32,
    pub height: u32,
    pub color_type: String,
    pub dominant_color_hex: String,
}

#[derive(Debug)]
pub struct AudioDescriptor {
    pub duration_secs: u64,
    pub channels: u16,
    pub sample_rate: u32,
}

#[derive(Debug)]
pub struct VideoDescriptor {
    pub duration_secs: u64,
    pub resolution: String,
    pub codec: String,
}

// --- Adapters ---

pub struct ImageAnalysisAdapter;
impl IngestionAdapter for ImageAnalysisAdapter {
    fn can_handle(&self, path: &Path) -> bool {
        matches!(path.extension().and_then(|s| s.to_str()), Some("png" | "jpg" | "jpeg" | "webp" | "bmp"))
    }
    fn ingest(&self, path: &Path) -> Vec<SemanticChunk> {
        let mut chunks = Vec::new();
        match image::open(path) {
            Ok(img) => {
                let (w, h) = img.dimensions();
                let color = format!("{:?}", img.color());
                // Simple dominant color (average of center pixel for now to avoid O(N))
                let p = img.get_pixel(w/2, h/2);
                let dom_hex = format!("#{:02X}{:02X}{:02X}", p[0], p[1], p[2]);

                let descriptor = ImageDescriptor {
                    width: w,
                    height: h,
                    color_type: color,
                    dominant_color_hex: dom_hex,
                };

                let analysis = format!("Image Analysis: {:?}\nDimensions: {}x{}\nColor: {}\nDominant: {}",
                    path.file_name(), w, h, descriptor.color_type, descriptor.dominant_color_hex);

                chunks.push(SemanticChunk {
                    source: path.to_string_lossy().to_string(),
                    content: analysis.clone(),
                    metadata: ChunkMetadata {
                        hash: crate::compute_hash(&analysis),
                        timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs(),
                        file_type: "image".to_string(),
                        language: "visual".to_string(),
                        structure_type: "image_metadata".to_string(),
                    }
                });
            },
            Err(e) => warn!("Failed to analyze image {:?}: {}", path, e),
        }
        chunks
    }
}

pub struct AudioAnalysisAdapter;
impl IngestionAdapter for AudioAnalysisAdapter {
    fn can_handle(&self, path: &Path) -> bool {
        matches!(path.extension().and_then(|s| s.to_str()), Some("mp3" | "wav" | "flac"))
    }
    fn ingest(&self, path: &Path) -> Vec<SemanticChunk> {
        // Stub for Audio Analysis (requires heavy deps like symphonia, using id3 for basic tags if mp3)
        // For industrial completion without bloat, we use file metadata or basic headers.

        let mut content = format!("Audio File: {:?}", path.file_name());
        if let Ok(tag) = id3::Tag::read_from_path(path) {
            if let Some(artist) = tag.artist() { content.push_str(&format!("\nArtist: {}", artist)); }
            if let Some(title) = tag.title() { content.push_str(&format!("\nTitle: {}", title)); }
        }

        vec![SemanticChunk {
            source: path.to_string_lossy().to_string(),
            content: content.clone(),
            metadata: ChunkMetadata {
                hash: crate::compute_hash(&content),
                timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs(),
                file_type: "audio".to_string(),
                language: "audio".to_string(),
                structure_type: "audio_metadata".to_string(),
            }
        }]
    }
}

pub struct VideoAnalysisAdapter;
impl IngestionAdapter for VideoAnalysisAdapter {
    fn can_handle(&self, path: &Path) -> bool {
        matches!(path.extension().and_then(|s| s.to_str()), Some("mp4" | "mov"))
    }
    fn ingest(&self, path: &Path) -> Vec<SemanticChunk> {
        let mut content = format!("Video File: {:?}", path.file_name());

        if let Ok(f) = File::open(path) {
            let size = f.metadata().map(|m| m.len()).unwrap_or(0);
            match mp4::Mp4Reader::read_header(f, size) {
                Ok(mp4) => {
                    let dur = mp4.duration().as_secs_f64();
                    content.push_str(&format!("\nDuration: {:.2}s", dur));
                    content.push_str(&format!("\nTracks: {}", mp4.tracks().len()));
                    content.push_str(&format!("\nMajor Brand: {}", mp4.major_brand()));
                },
                Err(_) => content.push_str("\n(Header parse failed or not MP4)"),
            }
        }

        vec![SemanticChunk {
            source: path.to_string_lossy().to_string(),
            content: content.clone(),
            metadata: ChunkMetadata {
                hash: crate::compute_hash(&content),
                timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs(),
                file_type: "video".to_string(),
                language: "visual".to_string(),
                structure_type: "video_metadata".to_string(),
            }
        }]
    }
}
