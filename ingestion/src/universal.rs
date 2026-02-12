use core_vsa::{SymbolGraph, HyperVector, SymbolNode, SymbolEdge};
use core_vsa::traits::Ingestor;
use std::path::Path;
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, Read};
use log::{info, warn, error};
use calamine::{Reader, Xlsx, open_workbook, DataType};
use lopdf::Document;
use lofty::{Probe, TaggedFileExt};
use walkdir::WalkDir;
use rayon::prelude::*;
use std::sync::{Arc, Mutex};
use image::GenericImageView;

// --- New Industrial Ingestor ---
pub struct UniversalIngestor;

impl Ingestor for UniversalIngestor {
    fn ingest(&self, path: &Path) -> Result<SymbolGraph, Box<dyn Error + Send + Sync>> {
        info!("Ingesting: {:?}", path);

        let graph = Arc::new(Mutex::new(SymbolGraph::new()));

        if path.is_file() {
            process_single_file(path, &graph)?;
        } else {
            // Parallel walk
            let entries: Vec<_> = WalkDir::new(path)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_file())
                .filter(|e| !e.path().to_string_lossy().contains("/.git/"))
                .collect();

            entries.par_iter().for_each(|entry| {
                if let Err(e) = process_single_file(entry.path(), &graph) {
                    warn!("Failed to process {:?}: {}", entry.path(), e);
                }
            });
        }

        let final_graph = graph.lock().unwrap().clone();
        Ok(final_graph)
    }
}

// --- Legacy Adapter for Backward Compatibility ---
// Needed because lib.rs and tests expect UniversalAdapter to implement IngestionAdapter
pub struct UniversalAdapter;

impl crate::IngestionAdapter for UniversalAdapter {
    fn can_handle(&self, _path: &Path) -> bool {
        true
    }

    fn ingest(&self, path: &Path) -> Vec<crate::SemanticChunk> {
        // Fallback to generic text reading for the legacy chunk system
        // But we should try to be smarter if possible.
        // For now, simple fallback is enough to fix compilation.
        crate::generic_read_file(path, "universal_fallback")
    }
}

// --- Internal Processing Logic ---

fn process_single_file(path: &Path, graph: &Arc<Mutex<SymbolGraph>>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");

    match ext {
        "xlsx" | "xls" | "ods" => process_spreadsheet(path, graph),
        "pdf" => process_pdf(path, graph),
        "csv" => process_csv(path, graph),
        "jpg" | "png" | "jpeg" => process_image(path, graph),
        "mp3" | "wav" | "flac" => process_audio(path, graph),
        "zip" | "tar" | "gz" => process_archive(path, graph),
        _ => process_text_generic(path, graph), // Code, Text, Markdown, etc.
    }
}

// --- Spreadsheet Processor ---
fn process_spreadsheet(path: &Path, graph: &Arc<Mutex<SymbolGraph>>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut workbook: Xlsx<_> = open_workbook(path).map_err(|e| e.to_string())?;

    if let Some(Ok(range)) = workbook.worksheet_range_at(0) {
        let mut headers = Vec::new();
        for (i, row) in range.rows().enumerate() {
            if i == 0 {
                // Header Row
                for cell in row {
                    headers.push(cell.to_string());
                }
                continue;
            }

            // Data Row
            let row_id = format!("{:?}#row{}", path, i);
            let mut row_node = SymbolNode {
                id: row_id.clone(),
                vector: HyperVector::random(), // Placeholder encoding
                metadata: std::collections::HashMap::new(),
            };
            row_node.metadata.insert("type".to_string(), "spreadsheet_row".to_string());

            // Create edges to headers
            for (j, cell) in row.iter().enumerate() {
                if j < headers.len() {
                    let header = &headers[j];
                    let val = cell.to_string();
                    row_node.metadata.insert(header.clone(), val.clone());

                    // Add edge to "Column" concept?
                    // For now, we just embed the data in metadata.
                }
            }

            let mut g = graph.lock().unwrap();
            g.add_node(&row_node.id, row_node.vector, row_node.metadata);
        }
    }
    Ok(())
}

// --- PDF Processor (using lopdf) ---
fn process_pdf(path: &Path, graph: &Arc<Mutex<SymbolGraph>>) -> Result<(), Box<dyn Error + Send + Sync>> {
    // lopdf Document::load can fail if file is locked or invalid
    let doc = Document::load(path).map_err(|e| e.to_string())?;
    let mut full_text = String::new();

    // Simple text extraction from pages
    for (page_num, _object_id) in doc.get_pages() {
        if let Ok(text) = doc.extract_text(&[page_num]) {
            full_text.push_str(&text);
            full_text.push('\n');
        }
    }

    // Create Node
    let node_id = format!("{:?}", path);
    let mut metadata = std::collections::HashMap::new();
    metadata.insert("type".to_string(), "pdf_document".to_string());
    metadata.insert("page_count".to_string(), doc.get_pages().len().to_string());
    // Store first 1k chars as preview? Or full text?
    // Store full text in metadata is dangerous for memory.
    // Store hash.
    metadata.insert("content_preview".to_string(), full_text.chars().take(200).collect());

    let vector = HyperVector::deterministic(full_text.len() as u64); // Placeholder

    let mut g = graph.lock().unwrap();
    g.add_node(&node_id, vector, metadata);

    Ok(())
}

// --- CSV Processor ---
fn process_csv(path: &Path, graph: &Arc<Mutex<SymbolGraph>>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut rdr = csv::Reader::from_path(path).map_err(|e| e.to_string())?;
    let headers = rdr.headers().map_err(|e| e.to_string())?.clone();

    for (i, result) in rdr.records().enumerate() {
        let record = result.map_err(|e| e.to_string())?;
        let row_id = format!("{:?}#row{}", path, i);

        let mut metadata = std::collections::HashMap::new();
        metadata.insert("type".to_string(), "csv_row".to_string());

        for (j, field) in record.iter().enumerate() {
            if j < headers.len() {
                metadata.insert(headers[j].to_string(), field.to_string());
            }
        }

        let vector = HyperVector::random();
        let mut g = graph.lock().unwrap();
        g.add_node(&row_id, vector, metadata);
    }
    Ok(())
}

// --- Image Processor ---
fn process_image(path: &Path, graph: &Arc<Mutex<SymbolGraph>>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let img = image::open(path).map_err(|e| e.to_string())?;
    let (width, height) = img.dimensions();

    let node_id = format!("{:?}", path);
    let mut metadata = std::collections::HashMap::new();
    metadata.insert("type".to_string(), "image".to_string());
    metadata.insert("width".to_string(), width.to_string());
    metadata.insert("height".to_string(), height.to_string());
    metadata.insert("color_type".to_string(), format!("{:?}", img.color()));

    // Compute simple perceptual hash (average color)
    // Placeholder: Random vector
    let vector = HyperVector::random();

    let mut g = graph.lock().unwrap();
    g.add_node(&node_id, vector, metadata);
    Ok(())
}

// --- Audio Processor ---
fn process_audio(path: &Path, graph: &Arc<Mutex<SymbolGraph>>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let tagged_file = Probe::open(path)
        .map_err(|e| e.to_string())?
        .read()
        .map_err(|e| e.to_string())?;

    let node_id = format!("{:?}", path);
    let mut metadata = std::collections::HashMap::new();
    metadata.insert("type".to_string(), "audio".to_string());

    if let Some(tag) = tagged_file.primary_tag() {
        if let Some(title) = tag.title() { metadata.insert("title".to_string(), title.to_string()); }
        if let Some(artist) = tag.artist() { metadata.insert("artist".to_string(), artist.to_string()); }
    }

    let properties = tagged_file.properties();
    metadata.insert("duration_seconds".to_string(), properties.duration().as_secs().to_string());

    let vector = HyperVector::random();
    let mut g = graph.lock().unwrap();
    g.add_node(&node_id, vector, metadata);
    Ok(())
}

// --- Archive Processor ---
fn process_archive(path: &Path, graph: &Arc<Mutex<SymbolGraph>>) -> Result<(), Box<dyn Error + Send + Sync>> {
    // Basic recursion: Unzip to temp, process, delete temp.
    // Or in-memory.
    // In-memory is safer for "Zero-Install".

    let file = File::open(path).map_err(|e| e.to_string())?;
    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");

    if ext == "zip" {
        let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
        for i in 0..archive.len() {
             let mut file = archive.by_index(i).map_err(|e| e.to_string())?;
             if file.is_dir() { continue; }

             let name = file.name().to_string();
             // Skip if likely binary/large inside zip to avoid bombs
             if name.ends_with(".exe") || name.ends_with(".dll") { continue; }

             // We can read content here.
             // But to reuse logic, we need to pass "path-like" and content.
             // For now, we just extract metadata of the entry.

             let entry_id = format!("{:?}#{}", path, name);
             let mut metadata = std::collections::HashMap::new();
             metadata.insert("type".to_string(), "archive_entry".to_string());
             metadata.insert("container".to_string(), format!("{:?}", path));

             let vector = HyperVector::random();
             let mut g = graph.lock().unwrap();
             g.add_node(&entry_id, vector, metadata);
        }
    }
    // (Tar logic similar, omitted for brevity but stubbed)
    Ok(())
}

// --- Generic Text/Code Processor ---
fn process_text_generic(path: &Path, graph: &Arc<Mutex<SymbolGraph>>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();

    // Limit 10MB
    if let Ok(meta) = file.metadata() {
        if meta.len() > 10 * 1024 * 1024 {
            return Ok(()); // Skip large
        }
    }

    file.read_to_end(&mut buffer)?;

    // UTF-8 check
    if let Ok(text) = String::from_utf8(buffer) {
        // Use the chunker logic from lib.rs (but updated)
        // Here we just make a node for the file.
        let node_id = format!("{:?}", path);
        let mut metadata = std::collections::HashMap::new();
        metadata.insert("type".to_string(), "text_file".to_string());
        metadata.insert("char_count".to_string(), text.len().to_string());

        // Detect language
        if let Some(info) = whatlang::detect(&text) {
             metadata.insert("language".to_string(), info.lang().to_string());
        }

        let vector = HyperVector::random(); // TODO: Encode text
        let mut g = graph.lock().unwrap();
        g.add_node(&node_id, vector, metadata);
    }
    Ok(())
}
