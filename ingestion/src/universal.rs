use core_vsa::{SymbolGraph, HyperVector, SymbolEdge};
use core_vsa::traits::Ingestor;
use std::path::Path;
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, Read, BufRead};
use log::{info, warn};
use calamine::{Reader, open_workbook_auto};
use lopdf::Document;
use lofty::{Probe, TaggedFileExt, Accessor, AudioFile};
use walkdir::WalkDir;
use rayon::prelude::*;
use std::sync::{Arc, Mutex};
use std::collections::HashSet;
use sha2::{Sha256, Digest};
use whatlang::detect;
use unicode_normalization::UnicodeNormalization;
use crate::metadata::NormalizedMetadata;
use crate::layout::SemanticLayoutExtractor;
use crate::ocr::SymbolicOCR;
use crate::video::VideoIngestor;

pub struct UniversalIngestor;

impl Ingestor for UniversalIngestor {
    fn ingest(&self, path: &Path) -> Result<SymbolGraph, Box<dyn Error + Send + Sync>> {
        info!("Ingesting: {:?}", path);
        let graph = Arc::new(Mutex::new(SymbolGraph::new()));
        let seen_hashes = Arc::new(Mutex::new(HashSet::new()));

        if path.is_file() {
            if let Err(e) = process_single_file(path, &graph, &seen_hashes) {
                warn!("Failed to process {:?}: {}", path, e);
            }
        } else {
            let entries: Vec<_> = WalkDir::new(path)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_file())
                .filter(|e| !e.path().to_string_lossy().contains("/.git/"))
                .collect();

            entries.par_iter().for_each(|entry| {
                if let Err(e) = process_single_file(entry.path(), &graph, &seen_hashes) {
                    warn!("Failed to process {:?}: {}", entry.path(), e);
                }
            });
        }
        let result = graph.lock().unwrap().clone();
        Ok(result)
    }
}

pub struct UniversalAdapter;
impl crate::IngestionAdapter for UniversalAdapter {
    fn can_handle(&self, _path: &Path) -> bool { true }
    fn ingest(&self, path: &Path) -> Vec<crate::SemanticChunk> {
        crate::generic_read_file(path, "universal_fallback")
    }
}

fn process_single_file(path: &Path, graph: &Arc<Mutex<SymbolGraph>>, seen: &Arc<Mutex<HashSet<String>>>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let file_hash = compute_file_hash(path).unwrap_or_else(|| format!("{:?}", path));
    {
        let mut s = seen.lock().unwrap();
        if s.contains(&file_hash) {
            info!("Skipping duplicate file: {:?}", path);
            return Ok(());
        }
        s.insert(file_hash.clone());
    }

    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
    match ext.as_str() {
        "xlsx" | "xls" | "ods" => process_spreadsheet(path, graph),
        "pdf" => process_pdf(path, graph),
        "csv" => process_csv(path, graph),
        "json" => process_json(path, graph),
        "yaml" | "yml" => process_yaml(path, graph),
        "docx" => process_docx(path, graph),
        "rs" | "py" | "c" | "cpp" | "js" | "ts" | "java" | "go" | "rb" => process_code(path, graph),
        "jpg" | "png" | "jpeg" | "webp" => process_image(path, graph),
        "mp3" | "wav" | "flac" | "m4a" => process_audio(path, graph),
        "mp4" | "mkv" | "avi" => process_video(path, graph),
        "zip" | "tar" | "gz" => process_archive(path, graph),
        _ => process_text_generic(path, graph, seen),
    }
}

fn compute_file_hash(path: &Path) -> Option<String> {
    let mut file = File::open(path).ok()?;
    let mut hasher = Sha256::new();
    std::io::copy(&mut file, &mut hasher).ok()?;
    Some(hex::encode(hasher.finalize()))
}

fn process_spreadsheet(path: &Path, graph: &Arc<Mutex<SymbolGraph>>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut workbook = open_workbook_auto(path).map_err(|e| e.to_string())?;
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();

    let sheet_names = workbook.sheet_names().to_vec();
    for sheet_name in sheet_names {
        if let Ok(range) = workbook.worksheet_range(&sheet_name) {
            let mut headers: Vec<String> = Vec::new();
            for (i, row) in range.rows().enumerate() {
                if i == 0 {
                    for cell in row.iter() { headers.push(cell.to_string()); }
                    continue;
                }
                let row_str = row.iter().map(|c| c.to_string()).collect::<Vec<String>>().join(",");
                let row_hash = {
                    let mut h = Sha256::new(); h.update(row_str.as_bytes()); hex::encode(h.finalize())
                };
                let row_id = format!("row:{}#{}", sheet_name, row_hash);

                let mut meta = NormalizedMetadata::new(&path.to_string_lossy());
                meta.file_type = "spreadsheet_row".to_string();
                meta.hash = row_hash;
                meta.timestamp = now;
                meta.insert("sheet", sheet_name.clone());

                let mut edges = Vec::new();
                for (j, cell) in row.iter().enumerate() {
                    if j < headers.len() {
                        let header = &headers[j];
                        let val = cell.to_string();
                        meta.insert(header, val.clone());
                        let col_id = format!("col:{}", header);
                        edges.push(SymbolEdge {
                            source: row_id.clone(),
                            target: col_id,
                            relation: "has_field".to_string(),
                            weight: 1.0,
                            confidence: 1.0,
                            source_reliability: 1.0,
                            reinforcement_count: 1,
                            timestamp: now,
                        });
                    }
                }
                let mut g = graph.lock().unwrap();
                g.add_node_with_confidence(&row_id, HyperVector::random(), meta.to_map(), 1.0);
                for edge in edges { g.edges.push(edge); }
            }
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

    let layout = SemanticLayoutExtractor::extract_from_text(&full_text);
    let file_hash = compute_file_hash(path).unwrap_or_else(|| format!("{:?}", path));
    let node_id = format!("doc:{}", file_hash);

    let mut meta = NormalizedMetadata::new(&path.to_string_lossy());
    meta.file_type = "pdf_document".to_string();
    meta.hash = file_hash;
    meta.insert("layout_elements", layout.len().to_string());

    if let Some(info) = detect(&full_text) {
        meta.language = info.lang().to_string();
    }

    let mut g = graph.lock().unwrap();
    g.add_node_with_confidence(&node_id, HyperVector::random(), meta.to_map(), 1.0);
    Ok(())
}

fn process_csv(path: &Path, graph: &Arc<Mutex<SymbolGraph>>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut rdr = csv::Reader::from_path(path).map_err(|e| e.to_string())?;
    let headers = rdr.headers().map_err(|e| e.to_string())?.clone();
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
    for result in rdr.records() {
        let record = result.map_err(|e| e.to_string())?;
        let mut h = Sha256::new();
        for field in &record { h.update(field.as_bytes()); }
        let row_hash = hex::encode(h.finalize());
        let row_id = format!("row:{}", row_hash);

        let mut meta = NormalizedMetadata::new(&path.to_string_lossy());
        meta.file_type = "csv_row".to_string();
        meta.hash = row_hash;
        meta.timestamp = now;

        let mut edges = Vec::new();
        for (j, field) in record.iter().enumerate() {
            if j < headers.len() {
                meta.insert(&headers[j], field.to_string());
                edges.push(SymbolEdge {
                    source: row_id.clone(),
                    target: format!("col:{}", &headers[j]),
                    relation: "has_field".to_string(),
                    weight: 1.0,
                    confidence: 1.0,
                    source_reliability: 1.0,
                    reinforcement_count: 1,
                    timestamp: now,
                });
            }
        }
        let mut g = graph.lock().unwrap();
        g.add_node_with_confidence(&row_id, HyperVector::random(), meta.to_map(), 1.0);
        g.edges.extend(edges);
    }
    Ok(())
}

fn process_image(path: &Path, graph: &Arc<Mutex<SymbolGraph>>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let img = image::open(path).map_err(|e| e.to_string())?;
    let ocr_text = SymbolicOCR::extract_text(&img);
    let file_hash = compute_file_hash(path).unwrap_or_else(|| format!("{:?}", path));
    let node_id = format!("img:{}", file_hash);

    let mut meta = NormalizedMetadata::new(&path.to_string_lossy());
    meta.file_type = "image".to_string();
    meta.hash = file_hash;
    meta.insert("ocr_content", ocr_text);

    let mut g = graph.lock().unwrap();
    g.add_node_with_confidence(&node_id, HyperVector::random(), meta.to_map(), 1.0);
    Ok(())
}

fn process_video(path: &Path, graph: &Arc<Mutex<SymbolGraph>>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let report = VideoIngestor::process_video(path)?;
    let file_hash = compute_file_hash(path).unwrap_or_else(|| format!("{:?}", path));
    let node_id = format!("video:{}", file_hash);

    let mut meta = NormalizedMetadata::new(&path.to_string_lossy());
    meta.file_type = "video".to_string();
    meta.hash = file_hash;
    meta.insert("video_report", report);

    let mut g = graph.lock().unwrap();
    g.add_node_with_confidence(&node_id, HyperVector::random(), meta.to_map(), 1.0);
    Ok(())
}

fn process_audio(path: &Path, graph: &Arc<Mutex<SymbolGraph>>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let tagged_file = Probe::open(path).map_err(|e| e.to_string())?.read().map_err(|e| e.to_string())?;
    let file_hash = compute_file_hash(path).unwrap_or_else(|| format!("{:?}", path));
    let node_id = format!("audio:{}", file_hash);

    let mut meta = NormalizedMetadata::new(&path.to_string_lossy());
    meta.file_type = "audio".to_string();
    meta.hash = file_hash;

    if let Some(tag) = tagged_file.primary_tag() {
        if let Some(title) = tag.title() { meta.insert("title", title.to_owned().to_string()); }
        if let Some(artist) = tag.artist() { meta.insert("artist", artist.to_owned().to_string()); }
        if let Some(album) = tag.album() { meta.insert("album", album.to_owned().to_string()); }
    }

    let properties = tagged_file.properties();
    meta.insert("duration_seconds", properties.duration().as_secs().to_string());

    let mut g = graph.lock().unwrap();
    g.add_node_with_confidence(&node_id, HyperVector::random(), meta.to_map(), 1.0);
    Ok(())
}

fn process_archive(path: &Path, graph: &Arc<Mutex<SymbolGraph>>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let file = File::open(path).map_err(|e| e.to_string())?;
    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
    let archive_hash = compute_file_hash(path).unwrap_or_else(|| format!("{:?}", path));
    let archive_id = format!("archive:{}", archive_hash);
    {
        let mut g = graph.lock().unwrap();
        let mut meta = NormalizedMetadata::new(&path.to_string_lossy());
        meta.file_type = "archive".to_string();
        meta.hash = archive_hash.clone();
        g.add_node_with_confidence(&archive_id, HyperVector::random(), meta.to_map(), 1.0);
    }
    if ext == "zip" {
        let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
        for i in 0..archive.len() {
             let file = archive.by_index(i).map_err(|e| e.to_string())?;
             if file.is_dir() { continue; }
             let name = file.name().to_string();
             let entry_id = format!("{}#{}", archive_id, name);

             let mut meta = NormalizedMetadata::new(&path.to_string_lossy());
             meta.file_type = "archive_entry".to_string();
             meta.insert("entry_name", name);

             let mut g = graph.lock().unwrap();
             g.add_node_with_confidence(&entry_id, HyperVector::random(), meta.to_map(), 1.0);
             g.add_edge_with_confidence(&entry_id, &archive_id, "contained_in", 1.0, 1.0);
        }
    }
    Ok(())
}

fn process_json(path: &Path, graph: &Arc<Mutex<SymbolGraph>>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let file_hash = compute_file_hash(path).unwrap_or_else(|| "none".to_string());
    let node_id = format!("json:{}", file_hash);
    let mut meta = NormalizedMetadata::new(&path.to_string_lossy());
    meta.file_type = "json_data".to_string();
    meta.hash = file_hash;

    let mut g = graph.lock().unwrap();
    g.add_node_with_confidence(&node_id, HyperVector::random(), meta.to_map(), 1.0);
    Ok(())
}

fn process_yaml(path: &Path, graph: &Arc<Mutex<SymbolGraph>>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let file_hash = compute_file_hash(path).unwrap_or_else(|| "none".to_string());
    let node_id = format!("yaml:{}", file_hash);
    let mut meta = NormalizedMetadata::new(&path.to_string_lossy());
    meta.file_type = "yaml_data".to_string();
    meta.hash = file_hash;

    let mut g = graph.lock().unwrap();
    g.add_node_with_confidence(&node_id, HyperVector::random(), meta.to_map(), 1.0);
    Ok(())
}

fn process_docx(path: &Path, graph: &Arc<Mutex<SymbolGraph>>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let file_hash = compute_file_hash(path).unwrap_or_else(|| "none".to_string());
    let node_id = format!("doc:{}", file_hash);
    let mut meta = NormalizedMetadata::new(&path.to_string_lossy());
    meta.file_type = "docx_document".to_string();
    meta.hash = file_hash;

    let mut g = graph.lock().unwrap();
    g.add_node_with_confidence(&node_id, HyperVector::random(), meta.to_map(), 1.0);
    Ok(())
}

fn process_code(path: &Path, graph: &Arc<Mutex<SymbolGraph>>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let file_hash = compute_file_hash(path).unwrap_or_else(|| "none".to_string());
    let node_id = format!("code:{}", file_hash);
    let mut meta = NormalizedMetadata::new(&path.to_string_lossy());
    meta.file_type = "source_code".to_string();
    meta.hash = file_hash;
    meta.language = path.extension().and_then(|s| s.to_str()).unwrap_or("unknown").to_string();

    let mut g = graph.lock().unwrap();
    g.add_node_with_confidence(&node_id, HyperVector::random(), meta.to_map(), 1.0);
    Ok(())
}

fn process_text_generic(path: &Path, graph: &Arc<Mutex<SymbolGraph>>, seen: &Arc<Mutex<HashSet<String>>>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let file = File::open(path)?;
    let file_meta = file.metadata()?;
    let reader = BufReader::new(file);
    let mut content_buffer = String::new();
    for line in reader.lines() {
        let l = line?;
        if compute_entropy(&l) < 2.0 { continue; }
        content_buffer.push_str(&l);
        content_buffer.push_str("\n");
        if content_buffer.len() > 5000 {
            ingest_text_chunk(&content_buffer, path, graph, &file_meta, seen)?;
            content_buffer.clear();
        }
    }
    if !content_buffer.is_empty() {
        ingest_text_chunk(&content_buffer, path, graph, &file_meta, seen)?;
    }
    Ok(())
}

fn compute_entropy(s: &str) -> f32 {
    if s.is_empty() { return 0.0; }
    let mut counts = [0usize; 256];
    for &b in s.as_bytes() { counts[b as usize] += 1; }
    let len = s.len() as f32;
    counts.iter().filter(|&&c| c > 0).map(|&c| {
        let p = c as f32 / len;
        -p * p.log2()
    }).sum()
}

fn ingest_text_chunk(text_raw: &str, path: &Path, graph: &Arc<Mutex<SymbolGraph>>, file_meta: &std::fs::Metadata, seen: &Arc<Mutex<HashSet<String>>>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let text: String = text_raw.nfc().collect();
    let chunk_hash = {
        let mut h = Sha256::new(); h.update(text.as_bytes()); hex::encode(h.finalize())
    };

    {
        let mut s = seen.lock().unwrap();
        if s.contains(&chunk_hash) { return Ok(()); }
        s.insert(chunk_hash.clone());
    }

    let node_id = format!("chunk:{}", chunk_hash);
    let mut meta = NormalizedMetadata::new(&path.to_string_lossy());
    meta.file_type = "text_chunk".to_string();
    meta.hash = chunk_hash;
    meta.timestamp = file_meta.modified()?.duration_since(std::time::UNIX_EPOCH)?.as_secs();

    if let Some(info) = detect(&text) {
         meta.language = info.lang().to_string();
    }
    let mut g = graph.lock().unwrap();
    g.add_node_with_confidence(&node_id, HyperVector::random(), meta.to_map(), 1.0);
    Ok(())
}
