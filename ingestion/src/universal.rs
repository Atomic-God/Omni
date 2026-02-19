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
use sha2::{Sha256, Digest};
use whatlang::{detect, Lang};

// ... UniversalIngestor struct ...
pub struct UniversalIngestor;

impl Ingestor for UniversalIngestor {
    fn ingest(&self, path: &Path) -> Result<SymbolGraph, Box<dyn Error + Send + Sync>> {
        info!("Ingesting: {:?}", path);
        let graph = Arc::new(Mutex::new(SymbolGraph::new()));

        if path.is_file() {
            process_single_file(path, &graph)?;
        } else {
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
        Ok(graph.lock().unwrap().clone())
    }
}

pub struct UniversalAdapter;
impl crate::IngestionAdapter for UniversalAdapter {
    fn can_handle(&self, _path: &Path) -> bool { true }
    fn ingest(&self, path: &Path) -> Vec<crate::SemanticChunk> {
        crate::generic_read_file(path, "universal_fallback")
    }
}

// ... Internal Processing Logic ...

fn process_single_file(path: &Path, graph: &Arc<Mutex<SymbolGraph>>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
    match ext {
        "xlsx" | "xls" | "ods" => process_spreadsheet(path, graph),
        "pdf" => process_pdf(path, graph),
        "csv" => process_csv(path, graph),
        "jpg" | "png" | "jpeg" => process_image(path, graph),
        "mp3" | "wav" | "flac" => process_audio(path, graph),
        "zip" | "tar" | "gz" => process_archive(path, graph),
        _ => process_text_generic(path, graph),
    }
}

fn compute_file_hash(path: &Path) -> Option<String> {
    let mut file = File::open(path).ok()?;
    let mut hasher = Sha256::new();
    std::io::copy(&mut file, &mut hasher).ok()?;
    Some(hex::encode(hasher.finalize()))
}

// ... Processors (Spreadsheet, PDF, CSV, Image, Audio, Archive - largely same as before) ...

fn process_spreadsheet(path: &Path, graph: &Arc<Mutex<SymbolGraph>>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut workbook: Xlsx<_> = open_workbook(path).map_err(|e| e.to_string())?;
    if let Some(Ok(range)) = workbook.worksheet_range_at(0) {
        let mut headers = Vec::new();
        for (i, row) in range.rows().enumerate() {
            if i == 0 {
                for cell in row { headers.push(cell.to_string()); }
                continue;
            }
            let row_str = row.iter().map(|c| c.to_string()).collect::<Vec<_>>().join(",");
            let row_hash = {
                let mut h = Sha256::new(); h.update(row_str.as_bytes()); hex::encode(h.finalize())
            };
            let row_id = format!("row:{}", row_hash);
            let mut row_node = SymbolNode {
                id: row_id.clone(),
                vector: HyperVector::deterministic(u64::from_str_radix(&row_hash[0..16], 16).unwrap_or(0)),
                metadata: std::collections::HashMap::new(),
            };
            row_node.metadata.insert("type".to_string(), "spreadsheet_row".to_string());
            row_node.metadata.insert("source".to_string(), path.to_string_lossy().to_string());

            let mut edges = Vec::new();
            for (j, cell) in row.iter().enumerate() {
                if j < headers.len() {
                    let header = &headers[j];
                    let val = cell.to_string();
                    row_node.metadata.insert(header.clone(), val.clone());
                    let col_id = format!("col:{}", header);
                    edges.push(SymbolEdge {
                        source: row_id.clone(),
                        target: col_id,
                        relation: "has_field".to_string(),
                        weight: 1.0,
                    });
                }
            }
            let mut g = graph.lock().unwrap();
            g.add_node(&row_node.id, row_node.vector, row_node.metadata);
            for edge in edges { g.edges.push(edge); }
        }
    }
    Ok(())
}

fn process_pdf(path: &Path, graph: &Arc<Mutex<SymbolGraph>>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let doc = Document::load(path).map_err(|e| e.to_string())?;
    let mut full_text = String::new();
    for (page_num, _) in doc.get_pages() {
        if let Ok(text) = doc.extract_text(&[page_num]) {
            full_text.push_str(&text);
            full_text.push('\n');
        }
    }
    let file_hash = compute_file_hash(path).unwrap_or_else(|| format!("{:?}", path));
    let node_id = format!("doc:{}", file_hash);
    let mut metadata = std::collections::HashMap::new();
    metadata.insert("type".to_string(), "pdf_document".to_string());
    metadata.insert("page_count".to_string(), doc.get_pages().len().to_string());
    metadata.insert("source".to_string(), path.to_string_lossy().to_string());

    // Language Detection
    if let Some(info) = detect(&full_text) {
        metadata.insert("language".to_string(), info.lang().to_string());
        metadata.insert("script".to_string(), info.script().to_string());
    }

    metadata.insert("content_preview".to_string(), full_text.chars().take(200).collect());
    let vector = HyperVector::deterministic(full_text.len() as u64);
    let mut g = graph.lock().unwrap();
    g.add_node(&node_id, vector, metadata);
    Ok(())
}

fn process_csv(path: &Path, graph: &Arc<Mutex<SymbolGraph>>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut rdr = csv::Reader::from_path(path).map_err(|e| e.to_string())?;
    let headers = rdr.headers().map_err(|e| e.to_string())?.clone();
    for result in rdr.records() {
        let record = result.map_err(|e| e.to_string())?;
        let mut h = Sha256::new();
        for field in &record { h.update(field.as_bytes()); }
        let row_hash = hex::encode(h.finalize());
        let row_id = format!("row:{}", row_hash);
        let mut metadata = std::collections::HashMap::new();
        metadata.insert("type".to_string(), "csv_row".to_string());
        metadata.insert("source".to_string(), path.to_string_lossy().to_string());
        let mut edges = Vec::new();
        for (j, field) in record.iter().enumerate() {
            if j < headers.len() {
                metadata.insert(headers[j].to_string(), field.to_string());
                edges.push(SymbolEdge {
                    source: row_id.clone(),
                    target: format!("col:{}", headers[j]),
                    relation: "has_field".to_string(),
                    weight: 1.0,
                });
            }
        }
        let vector = HyperVector::random();
        let mut g = graph.lock().unwrap();
        g.add_node(&row_id, vector, metadata);
        g.edges.extend(edges);
    }
    Ok(())
}

fn process_image(path: &Path, graph: &Arc<Mutex<SymbolGraph>>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let img = image::open(path).map_err(|e| e.to_string())?;
    let (width, height) = img.dimensions();
    let file_hash = compute_file_hash(path).unwrap_or_else(|| format!("{:?}", path));
    let node_id = format!("img:{}", file_hash);
    let mut metadata = std::collections::HashMap::new();
    metadata.insert("type".to_string(), "image".to_string());
    metadata.insert("width".to_string(), width.to_string());
    metadata.insert("height".to_string(), height.to_string());
    metadata.insert("color_type".to_string(), format!("{:?}", img.color()));
    metadata.insert("source".to_string(), path.to_string_lossy().to_string());
    let vector = HyperVector::random();
    let mut g = graph.lock().unwrap();
    g.add_node(&node_id, vector, metadata);
    Ok(())
}

fn process_audio(path: &Path, graph: &Arc<Mutex<SymbolGraph>>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let tagged_file = Probe::open(path).map_err(|e| e.to_string())?.read().map_err(|e| e.to_string())?;
    let file_hash = compute_file_hash(path).unwrap_or_else(|| format!("{:?}", path));
    let node_id = format!("audio:{}", file_hash);
    let mut metadata = std::collections::HashMap::new();
    metadata.insert("type".to_string(), "audio".to_string());
    metadata.insert("source".to_string(), path.to_string_lossy().to_string());
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

fn process_archive(path: &Path, graph: &Arc<Mutex<SymbolGraph>>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let file = File::open(path).map_err(|e| e.to_string())?;
    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
    let archive_hash = compute_file_hash(path).unwrap_or_else(|| format!("{:?}", path));
    let archive_id = format!("archive:{}", archive_hash);
    {
        let mut g = graph.lock().unwrap();
        let mut meta = std::collections::HashMap::new();
        meta.insert("type".to_string(), "archive".to_string());
        meta.insert("source".to_string(), path.to_string_lossy().to_string());
        g.add_node(&archive_id, HyperVector::random(), meta);
    }
    if ext == "zip" {
        let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
        for i in 0..archive.len() {
             let mut file = archive.by_index(i).map_err(|e| e.to_string())?;
             if file.is_dir() { continue; }
             let name = file.name().to_string();
             if name.ends_with(".exe") || name.ends_with(".dll") { continue; }
             let entry_id = format!("{}#{}", archive_id, name);
             let mut metadata = std::collections::HashMap::new();
             metadata.insert("type".to_string(), "archive_entry".to_string());
             metadata.insert("container".to_string(), path.to_string_lossy().to_string());
             let vector = HyperVector::random();
             let mut g = graph.lock().unwrap();
             g.add_node(&entry_id, vector, metadata);
             g.add_edge(&entry_id, &archive_id, "contained_in", 1.0);
        }
    }
    Ok(())
}

fn process_text_generic(path: &Path, graph: &Arc<Mutex<SymbolGraph>>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    if let Ok(meta) = file.metadata() {
        if meta.len() > 10 * 1024 * 1024 { return Ok(()); }
    }
    file.read_to_end(&mut buffer)?;
    if let Ok(text) = String::from_utf8(buffer) {
        let file_hash = compute_file_hash(path).unwrap_or_else(|| format!("{:?}", path));
        let node_id = format!("doc:{}", file_hash);
        let mut metadata = std::collections::HashMap::new();
        metadata.insert("type".to_string(), "text_file".to_string());
        metadata.insert("char_count".to_string(), text.len().to_string());
        metadata.insert("source".to_string(), path.to_string_lossy().to_string());

        // Multilingual Detection
        if let Some(info) = detect(&text) {
             metadata.insert("language".to_string(), info.lang().to_string());
             metadata.insert("script".to_string(), info.script().to_string());
        }

        let vector = HyperVector::random();
        let mut g = graph.lock().unwrap();
        g.add_node(&node_id, vector, metadata);
    }
    Ok(())
}
