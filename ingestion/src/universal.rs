use core_vsa::{SymbolGraph, HyperVector, SymbolEdge};
use core_vsa::traits::Ingestor;
use std::path::Path;
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufRead, Read};
use tracing::{info, warn, debug};
use calamine::{Reader, open_workbook_auto};
use lopdf::Document;
use lofty::{Probe, TaggedFileExt, Accessor, AudioFile};
use walkdir::WalkDir;
use rayon::prelude::*;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::collections::HashSet;
use sha2::{Sha256, Digest};
use whatlang::detect;
use unicode_normalization::UnicodeNormalization;
use crate::metadata::NormalizedMetadata;
use crate::layout::SemanticLayoutExtractor;
use crate::ocr::SymbolicOCR;
use crate::video::VideoIngestor;
use crate::vision::VisionSemanticExtractor;
use crate::nlp::SymbolicNLP;
use crate::code_analysis::CodeAnalyzer;
use crate::data_meaning::DataMeaningExtractor;
use crate::audio_meaning::AudioMeaningExtractor;

pub struct UniversalIngestor {
    pub ocr: SymbolicOCR,
    pub vsa_dimension: usize,
}

impl UniversalIngestor {
    pub fn new() -> Self {
        Self {
            ocr: SymbolicOCR::new(),
            vsa_dimension: core_vsa::DIMENSION,
        }
    }

    pub fn with_dimension(dimension: usize) -> Self {
        Self {
            ocr: SymbolicOCR::new(),
            vsa_dimension: dimension,
        }
    }
}

#[derive(Default)]
struct DuplicateRegistry {
    hashes: HashSet<String>,
    size_fast_hashes: HashSet<(u64, u64)>,
    semantic_vectors: Vec<HyperVector>,
}

impl Ingestor for UniversalIngestor {
    fn ingest(&self, path: &Path) -> Result<SymbolGraph, Box<dyn Error + Send + Sync>> {
        info!("Industrial Ingestion Pipeline active: {:?}", path);
        let graph = Arc::new(Mutex::new(SymbolGraph::new()));
        let seen_hashes = Arc::new(Mutex::new(DuplicateRegistry::default()));

        if path.is_file() {
            if let Err(e) = self.process_single_file(path, &graph, &seen_hashes) {
                warn!("Failed to process {:?}: {}", path, e);
            }
        } else {
            let entries: Vec<_> = WalkDir::new(path)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_file())
                .filter(|e| !e.path().to_string_lossy().contains("/.git/"))
                .collect();

            let total = entries.len();
            let processed = AtomicUsize::new(0);
            let errors = AtomicUsize::new(0);

            info!("Industrial Batch Ingestion: {} files detected. Starting parallel pipeline.", total);

            let start_time = std::time::Instant::now();

            entries.par_iter().for_each(|entry| {
                match self.process_single_file(entry.path(), &graph, &seen_hashes) {
                    Ok(_) => {
                        let count = processed.fetch_add(1, Ordering::SeqCst) + 1;
                        if count % 10 == 0 || count == total {
                            let elapsed = start_time.elapsed().as_secs_f32();
                            let rate = count as f32 / elapsed;
                            info!("Ingestion Progress: {}/{} files ({}%) [Rate: {:.2} files/sec]", count, total, (count * 100) / total, rate);
                        }
                    }
                    Err(e) => {
                        warn!("Batch Pipeline Error [File: {:?}]: {}", entry.path(), e);
                        errors.fetch_add(1, Ordering::SeqCst);
                    }
                }
            });

            let final_count = processed.load(Ordering::SeqCst);
            let final_errors = errors.load(Ordering::SeqCst);
            let elapsed = start_time.elapsed();
            info!("Batch Ingestion Complete. Processed: {}, Errors: {}, Total Time: {:?}", final_count, final_errors, elapsed);

            // Add batch summary metadata node
            let mut g = graph.lock().unwrap();
            let mut summary_meta = std::collections::HashMap::new();
            summary_meta.insert("batch_root".to_string(), path.to_string_lossy().to_string());
            summary_meta.insert("files_processed".to_string(), final_count.to_string());
            summary_meta.insert("errors_encountered".to_string(), final_errors.to_string());
            summary_meta.insert("total_time_ms".to_string(), elapsed.as_millis().to_string());

            let summary_id = format!("batch:{}", seahash::hash(path.to_string_lossy().as_bytes()));
            g.add_node_with_confidence(&summary_id, HyperVector::deterministic(0xBA7C), summary_meta, 1.0);
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

impl UniversalIngestor {
    fn process_single_file(&self, path: &Path, graph: &Arc<Mutex<SymbolGraph>>, seen: &Arc<Mutex<DuplicateRegistry>>) -> Result<(), Box<dyn Error + Send + Sync>> {
        // Multi-stage Duplicate Detection
        let metadata = std::fs::metadata(path)?;
        let size = metadata.len();

        let fast_hash = self.compute_fast_hash(path).unwrap_or(0);
        {
            let mut s = seen.lock().unwrap();
            if s.size_fast_hashes.contains(&(size, fast_hash)) {
                // Potential duplicate, perform full hash check
                let file_hash = self.compute_file_hash(path).unwrap_or_else(|| format!("{:?}", path));
                if s.hashes.contains(&file_hash) {
                    debug!("Definitive duplicate detected (SHA256): {:?}", path);
                    return Ok(());
                }
                s.hashes.insert(file_hash);
            } else {
                s.size_fast_hashes.insert((size, fast_hash));
            }
        }

        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
        let result = match ext.as_str() {
            "xlsx" | "xls" | "ods" => self.process_spreadsheet(path, graph),
            "pdf" => self.process_pdf(path, graph),
            "csv" => self.process_csv(path, graph),
            "json" => self.process_json(path, graph),
            "xml" => self.process_xml(path, graph),
            "yaml" | "yml" => self.process_yaml(path, graph),
            "docx" => self.process_docx(path, graph),
            "rs" | "py" | "c" | "cpp" | "js" | "ts" | "java" | "go" | "rb" => self.process_code(path, graph),
            "rtf" => self.process_rtf(path, graph),
            "jpg" | "png" | "jpeg" | "webp" => self.process_image(path, graph, seen),
            "mp3" | "wav" | "flac" | "m4a" => self.process_audio(path, graph),
            "mp4" | "mkv" | "avi" => self.process_video(path, graph),
            "zip" | "tar" | "gz" => self.process_archive(path, graph),
            _ => self.process_text_generic(path, graph, seen),
        };

        // Industrial Error Recovery Fallback: if specialized parser fails, try generic text ingestion
        if let Err(ref e) = result {
            warn!("Specialized ingestion failed for {:?} ({}), attempting generic text fallback.", path, e);
            if let Ok(_) = self.process_text_generic(path, graph, seen) {
                info!("Fallback success: Ingested {:?} as generic text.", path);
                return Ok(());
            }
        }
        result
    }

    fn compute_fast_hash(&self, path: &Path) -> Option<u64> {
        use std::io::Read;
        let mut file = File::open(path).ok()?;
        let mut buffer = [0u8; 1024];
        let n = file.read(&mut buffer).ok()?;
        if n == 0 { return Some(0); }

        use std::collections::hash_map::DefaultHasher;
        use std::hash::Hasher;
        let mut hasher = DefaultHasher::new();
        hasher.write(&buffer[..n]);
        Some(hasher.finish())
    }

    fn compute_file_hash(&self, path: &Path) -> Option<String> {
        let mut file = File::open(path).ok()?;
        let mut hasher = Sha256::new();
        std::io::copy(&mut file, &mut hasher).ok()?;
        Some(hex::encode(hasher.finalize()))
    }

    fn process_spreadsheet(&self, path: &Path, graph: &Arc<Mutex<SymbolGraph>>) -> Result<(), Box<dyn Error + Send + Sync>> {
        let mut workbook = open_workbook_auto(path).map_err(|e| e.to_string())?;
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();

        let sheet_names = workbook.sheet_names().to_vec();
        for sheet_name in sheet_names {
            if let Ok(range) = workbook.worksheet_range(&sheet_name) {
                let mut headers: Vec<String> = Vec::new();
                let mut all_rows: Vec<Vec<String>> = Vec::new();

                for (i, row) in range.rows().enumerate() {
                    let row_vals: Vec<String> = row.iter().map(|c| {
                        match c {
                            calamine::Data::Empty => "".to_string(),
                            calamine::Data::String(s) => s.clone(),
                            calamine::Data::Float(f) => f.to_string(),
                            calamine::Data::Int(i) => i.to_string(),
                            calamine::Data::Bool(b) => b.to_string(),
                            calamine::Data::Error(e) => format!("Error: {:?}", e),
                            calamine::Data::DateTime(d) => d.to_string(),
                            calamine::Data::DateTimeIso(s) => s.clone(),
                            calamine::Data::DurationIso(s) => s.clone(),
                        }
                    }).collect();

                    if i == 0 {
                        headers = row_vals;
                        continue;
                    }

                    // Skip empty rows to reduce noise
                    if row_vals.iter().all(|v| v.is_empty()) { continue; }

                    all_rows.push(row_vals.clone());

                    let row_str = row_vals.join(",");
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
                    for (j, val) in row_vals.iter().enumerate() {
                        if j < headers.len() {
                            let header = &headers[j];
                            meta.insert(header, val.clone());
                            edges.push(SymbolEdge {
                                source: row_id.clone(),
                                target: format!("col:{}", header),
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
                    g.add_node_with_confidence(&row_id, HyperVector::random_dim(self.vsa_dimension), meta.to_map(), 1.0);
                    for edge in edges { g.edges.push(edge); }
                }

                let data_facts = DataMeaningExtractor::extract_spreadsheet_meaning(&headers, &all_rows);
                let mut g = graph.lock().unwrap();
                for fact in data_facts {
                    g.add_edge_with_confidence(&fact.subject, &fact.object, &fact.predicate, 1.0, 1.0);
                }
            }
        }
        Ok(())
    }

    fn process_pdf(&self, path: &Path, graph: &Arc<Mutex<SymbolGraph>>) -> Result<(), Box<dyn Error + Send + Sync>> {
        let doc = Document::load(path).map_err(|e| e.to_string())?;
        let mut full_text = String::new();
        for (page_num, _) in doc.get_pages() {
            if let Ok(text) = doc.extract_text(&[page_num]) {
                full_text.push_str(&text);
                full_text.push('\n');
            }
        }

        let layout = SemanticLayoutExtractor::extract_from_text(&full_text);
        let facts = SymbolicNLP::extract_deep_facts(&full_text);

        let file_hash = self.compute_file_hash(path).unwrap_or_else(|| format!("{:?}", path));
        let node_id = format!("doc:{}", file_hash);

        let mut meta = NormalizedMetadata::new(&path.to_string_lossy());
        meta.file_type = "pdf_document".to_string();
        meta.hash = file_hash;
        meta.insert("layout_elements", layout.len().to_string());
        meta.insert("extracted_meaning_count", facts.len().to_string());

        if let Some(info) = detect(&full_text) {
            meta.language = info.lang().to_string();
        }

        let mut g = graph.lock().unwrap();
        g.add_node_with_confidence(&node_id, HyperVector::random_dim(self.vsa_dimension), meta.to_map(), 1.0);
        for fact in facts {
             g.add_edge_with_confidence(&fact.subject, &fact.object, &fact.predicate, 1.0, 1.0);
        }
        Ok(())
    }

    fn process_csv(&self, path: &Path, graph: &Arc<Mutex<SymbolGraph>>) -> Result<(), Box<dyn Error + Send + Sync>> {
        let mut rdr = csv::Reader::from_path(path).map_err(|e| e.to_string())?;
        let headers = rdr.headers().map_err(|e| e.to_string())?.clone();
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();

        let mut all_rows = Vec::new();
        let header_vec: Vec<String> = headers.iter().map(|s| s.to_string()).collect();

        for result in rdr.records() {
            let record = result.map_err(|e| e.to_string())?;
            let row_vals: Vec<String> = record.iter().map(|s| s.to_string()).collect();
            all_rows.push(row_vals.clone());

            let mut h = Sha256::new();
            for field in &row_vals { h.update(field.as_bytes()); }
            let row_hash = hex::encode(h.finalize());
            let row_id = format!("row:{}", row_hash);

            let mut meta = NormalizedMetadata::new(&path.to_string_lossy());
            meta.file_type = "csv_row".to_string();
            meta.hash = row_hash;
            meta.timestamp = now;

            let mut edges = Vec::new();
            for (j, val) in row_vals.iter().enumerate() {
                if j < headers.len() {
                    meta.insert(&headers[j], val.to_string());
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
            g.add_node_with_confidence(&row_id, HyperVector::random_dim(self.vsa_dimension), meta.to_map(), 1.0);
            g.edges.extend(edges);
        }

        let data_facts = DataMeaningExtractor::extract_spreadsheet_meaning(&header_vec, &all_rows);
        let mut g = graph.lock().unwrap();
        for fact in data_facts {
            g.add_edge_with_confidence(&fact.subject, &fact.object, &fact.predicate, 1.0, 1.0);
        }

        Ok(())
    }

    fn process_image(&self, path: &Path, graph: &Arc<Mutex<SymbolGraph>>, seen: &Arc<Mutex<DuplicateRegistry>>) -> Result<(), Box<dyn Error + Send + Sync>> {
        let img = image::open(path).map_err(|e| e.to_string())?;
        let ocr_text = self.ocr.extract_text(&img);
        let (semantic_vec, meaning_desc, vision_meta) = VisionSemanticExtractor::extract_deep_meaning(&img);

        {
            let mut s = seen.lock().unwrap();
            for prev in s.semantic_vectors.iter() {
                if semantic_vec.similarity(prev) > 0.95 {
                    info!("Near-duplicate visual detected: {:?}", path);
                    return Ok(());
                }
            }
            s.semantic_vectors.push(semantic_vec.clone());
        }

        let file_hash = self.compute_file_hash(path).unwrap_or_else(|| format!("{:?}", path));
        let node_id = format!("img:{}", file_hash);

        let mut meta = NormalizedMetadata::new(&path.to_string_lossy());
        meta.file_type = "image".to_string();
        meta.hash = file_hash;
        meta.insert("ocr_content", ocr_text);
        meta.insert("deep_semantic_meaning", meaning_desc);
        for (k, v) in vision_meta { meta.insert(&k, v); }

        let mut g = graph.lock().unwrap();
        g.add_node_with_confidence(&node_id, semantic_vec, meta.to_map(), 1.0);
        Ok(())
    }

    fn process_video(&self, path: &Path, graph: &Arc<Mutex<SymbolGraph>>) -> Result<(), Box<dyn Error + Send + Sync>> {
        let report = VideoIngestor::process_video(path)?;
        let file_hash = self.compute_file_hash(path).unwrap_or_else(|| format!("{:?}", path));
        let node_id = format!("video:{}", file_hash);

        let mut meta = NormalizedMetadata::new(&path.to_string_lossy());
        meta.file_type = "video".to_string();
        meta.hash = file_hash;
        meta.insert("video_report", report);

        let mut g = graph.lock().unwrap();
        g.add_node_with_confidence(&node_id, HyperVector::random_dim(self.vsa_dimension), meta.to_map(), 1.0);
        Ok(())
    }

    fn process_audio(&self, path: &Path, graph: &Arc<Mutex<SymbolGraph>>) -> Result<(), Box<dyn Error + Send + Sync>> {
        let file_hash = self.compute_file_hash(path).unwrap_or_else(|| format!("{:?}", path));
        let node_id = format!("audio:{}", file_hash);
        let mut meta = NormalizedMetadata::new(&path.to_string_lossy());
        meta.file_type = "audio".to_string();
        meta.hash = file_hash;

        // Try to read tags, but don't fail if they are missing or corrupted
        match Probe::open(path).and_then(|p| p.read()) {
            Ok(tagged_file) => {
                if let Some(tag) = tagged_file.primary_tag() {
                    if let Some(title) = tag.title() { meta.insert("title", title.to_string()); }
                    if let Some(artist) = tag.artist() { meta.insert("artist", artist.to_string()); }
                    if let Some(album) = tag.album() { meta.insert("album", album.to_string()); }
                }
                let properties = tagged_file.properties();
                meta.insert("duration_seconds", properties.duration().as_secs().to_string());
                meta.insert("sample_rate", properties.sample_rate().unwrap_or(0).to_string());
                meta.insert("bitrate", properties.audio_bitrate().unwrap_or(0).to_string());
            }
            Err(e) => {
                warn!("Lofty: Could not read metadata for {:?}: {}", path, e);
                meta.insert("metadata_error", e.to_string());
            }
        }

        // Extract deep symbolic meaning from audio (even if metadata failed)
        let (content_vec, content_desc) = AudioMeaningExtractor::extract_signature(path);
        meta.insert("acoustic_signature", content_desc);

        let mut g = graph.lock().unwrap();
        g.add_node_with_confidence(&node_id, content_vec, meta.to_map(), 1.0);
        Ok(())
    }

    fn process_archive(&self, path: &Path, graph: &Arc<Mutex<SymbolGraph>>) -> Result<(), Box<dyn Error + Send + Sync>> {
        let file = File::open(path).map_err(|e| e.to_string())?;
        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
        let archive_hash = self.compute_file_hash(path).unwrap_or_else(|| format!("{:?}", path));
        let archive_id = format!("archive:{}", archive_hash);
        {
            let mut g = graph.lock().unwrap();
            let mut meta = NormalizedMetadata::new(&path.to_string_lossy());
            meta.file_type = "archive".to_string();
            meta.hash = archive_hash.clone();
            g.add_node_with_confidence(&archive_id, HyperVector::random_dim(self.vsa_dimension), meta.to_map(), 1.0);
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
                 g.add_node_with_confidence(&entry_id, HyperVector::random_dim(self.vsa_dimension), meta.to_map(), 1.0);
                 g.add_edge_with_confidence(&entry_id, &archive_id, "contained_in", 1.0, 1.0);
            }
        }
        Ok(())
    }

    fn process_xml(&self, path: &Path, graph: &Arc<Mutex<SymbolGraph>>) -> Result<(), Box<dyn Error + Send + Sync>> {
        use crate::adapters::XmlAdapter;
        use crate::IngestionAdapter;

        let adapter = XmlAdapter;
        let chunks = adapter.ingest(path);
        for chunk in chunks {
            let node_id = format!("xml:{}", chunk.metadata.hash);
            let mut g = graph.lock().unwrap();
            let mut meta = chunk.metadata.to_map();
            meta.insert("source".to_string(), path.to_string_lossy().to_string());
            g.add_node_with_confidence(&node_id, HyperVector::random_dim(self.vsa_dimension), meta, 1.0);

            let facts = SymbolicNLP::extract_deep_facts(&chunk.content);
            for fact in facts {
                g.add_edge_with_confidence(&fact.subject, &fact.object, &fact.predicate, 1.0, 1.0);
            }
        }
        Ok(())
    }

    fn process_json(&self, path: &Path, graph: &Arc<Mutex<SymbolGraph>>) -> Result<(), Box<dyn Error + Send + Sync>> {
        let file = File::open(path)?;
        let val: serde_json::Value = serde_json::from_reader(file)?;

        let schema_facts = DataMeaningExtractor::extract_schema_meaning("root", &val);

        let file_hash = self.compute_file_hash(path).unwrap_or_else(|| "none".to_string());
        let node_id = format!("json:{}", file_hash);
        let mut meta = NormalizedMetadata::new(&path.to_string_lossy());
        meta.file_type = "json_data".to_string();
        meta.hash = file_hash;

        let mut g = graph.lock().unwrap();
        g.add_node_with_confidence(&node_id, HyperVector::random_dim(self.vsa_dimension), meta.to_map(), 1.0);
        for fact in schema_facts {
            g.add_edge_with_confidence(&fact.subject, &fact.object, &fact.predicate, 1.0, 1.0);
        }
        Ok(())
    }

    fn process_yaml(&self, path: &Path, graph: &Arc<Mutex<SymbolGraph>>) -> Result<(), Box<dyn Error + Send + Sync>> {
        let file = File::open(path)?;
        let val: serde_json::Value = serde_yaml::from_reader(file).map_err(|e: serde_yaml::Error| e.to_string())?;
        let schema_facts = DataMeaningExtractor::extract_schema_meaning("root", &val);

        let file_hash = self.compute_file_hash(path).unwrap_or_else(|| "none".to_string());
        let node_id = format!("yaml:{}", file_hash);
        let mut meta = NormalizedMetadata::new(&path.to_string_lossy());
        meta.file_type = "yaml_data".to_string();
        meta.hash = file_hash;

        let mut g = graph.lock().unwrap();
        g.add_node_with_confidence(&node_id, HyperVector::random_dim(self.vsa_dimension), meta.to_map(), 1.0);
        for fact in schema_facts {
            g.add_edge_with_confidence(&fact.subject, &fact.object, &fact.predicate, 1.0, 1.0);
        }
        Ok(())
    }

    fn process_docx(&self, path: &Path, graph: &Arc<Mutex<SymbolGraph>>) -> Result<(), Box<dyn Error + Send + Sync>> {
        use crate::adapters::DocxAdapter;
        use crate::IngestionAdapter;

        let adapter = DocxAdapter;
        let chunks = adapter.ingest(path);

        for chunk in chunks {
            let node_id = format!("docx:{}", chunk.metadata.hash);
            let mut g = graph.lock().unwrap();

            let mut meta = chunk.metadata.to_map();
            meta.insert("source".to_string(), path.to_string_lossy().to_string());

            g.add_node_with_confidence(&node_id, HyperVector::random_dim(self.vsa_dimension), meta, 1.0);

            // Extract NLP facts from DOCX content
            let facts = SymbolicNLP::extract_deep_facts(&chunk.content);
            for fact in facts {
                g.add_edge_with_confidence(&fact.subject, &fact.object, &fact.predicate, 1.0, 1.0);
            }
        }
        Ok(())
    }

    fn process_rtf(&self, path: &Path, graph: &Arc<Mutex<SymbolGraph>>) -> Result<(), Box<dyn Error + Send + Sync>> {
        let mut file = File::open(path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;

        // Very basic RTF stripping: remove anything inside {} or starting with \
        let re = regex::Regex::new(r"\{.*?\}|\\.*?\s|\\.*?[^a-zA-Z]").unwrap();
        let stripped = re.replace_all(&content, " ");
        let final_text = stripped.split_whitespace().collect::<Vec<_>>().join(" ");

        let chunk_hash = {
            let mut h = Sha256::new(); h.update(final_text.as_bytes()); hex::encode(h.finalize())
        };
        let node_id = format!("rtf:{}", chunk_hash);

        let mut meta = NormalizedMetadata::new(&path.to_string_lossy());
        meta.file_type = "rtf_document".to_string();
        meta.hash = chunk_hash;

        let facts = SymbolicNLP::extract_deep_facts(&final_text);

        let mut g = graph.lock().unwrap();
        g.add_node_with_confidence(&node_id, HyperVector::random_dim(self.vsa_dimension), meta.to_map(), 1.0);
        for fact in facts {
            g.add_edge_with_confidence(&fact.subject, &fact.object, &fact.predicate, 1.0, 1.0);
        }
        Ok(())
    }

    fn process_code(&self, path: &Path, graph: &Arc<Mutex<SymbolGraph>>) -> Result<(), Box<dyn Error + Send + Sync>> {
        let facts = CodeAnalyzer::extract_code_meaning(path);
        let file_hash = self.compute_file_hash(path).unwrap_or_else(|| "none".to_string());
        let node_id = format!("code:{}", file_hash);

        let mut meta = NormalizedMetadata::new(&path.to_string_lossy());
        meta.file_type = "source_code".to_string();
        meta.hash = file_hash;
        meta.language = path.extension().and_then(|s| s.to_str()).unwrap_or("unknown").to_string();
        meta.insert("extracted_code_facts", facts.len().to_string());

        let mut g = graph.lock().unwrap();
        g.add_node_with_confidence(&node_id, HyperVector::random_dim(self.vsa_dimension), meta.to_map(), 1.0);
        for fact in facts {
            g.add_edge_with_confidence(&fact.subject, &fact.object, &fact.predicate, 1.0, 1.0);
        }
        Ok(())
    }

    fn process_text_generic(&self, path: &Path, graph: &Arc<Mutex<SymbolGraph>>, seen: &Arc<Mutex<DuplicateRegistry>>) -> Result<(), Box<dyn Error + Send + Sync>> {
        let file = File::open(path)?;
        let file_meta = file.metadata()?;
        let reader = BufReader::new(file);
        let mut content_buffer = String::new();
        for line in reader.lines() {
            let l = line?;
            if self.compute_entropy(&l) < 2.0 { continue; }
            content_buffer.push_str(&l);
            content_buffer.push_str("\n");
            if content_buffer.len() > 5000 {
                self.ingest_text_chunk(&content_buffer, path, graph, &file_meta, seen)?;
                content_buffer.clear();
            }
        }
        if !content_buffer.is_empty() {
            self.ingest_text_chunk(&content_buffer, path, graph, &file_meta, seen)?;
        }
        Ok(())
    }

    fn compute_entropy(&self, s: &str) -> f32 {
        if s.is_empty() { return 0.0; }
        let mut counts = [0usize; 256];
        for &b in s.as_bytes() { counts[b as usize] += 1; }
        let len = s.len() as f32;
        counts.iter().filter(|&&c| c > 0).map(|&c| {
            let p = c as f32 / len;
            -p * p.log2()
        }).sum()
    }

    fn encode_text_deterministic(&self, text: &str) -> HyperVector {
        let words: Vec<String> = text.split_whitespace()
            .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()).to_lowercase())
            .filter(|w| !w.is_empty())
            .collect();

        if words.is_empty() {
            return HyperVector::deterministic_dim(0, self.vsa_dimension);
        }

        let mut words = words;
        words.sort();

        let mut result = None;
        for word in words {
            let h = seahash::hash(word.as_bytes());
            let v = HyperVector::deterministic_dim(h, self.vsa_dimension);
            match result {
                None => result = Some(v),
                Some(r) => result = Some(r.bundle(&v)),
            }
        }
        result.unwrap_or_else(|| HyperVector::random_dim(self.vsa_dimension))
    }

    fn ingest_text_chunk(&self, text_raw: &str, path: &Path, graph: &Arc<Mutex<SymbolGraph>>, file_meta: &std::fs::Metadata, seen: &Arc<Mutex<DuplicateRegistry>>) -> Result<(), Box<dyn Error + Send + Sync>> {
        let text: String = text_raw.nfc().collect();
        let chunk_hash = {
            let mut h = Sha256::new(); h.update(text.as_bytes()); hex::encode(h.finalize())
        };

        {
            let mut s = seen.lock().unwrap();
            if s.hashes.contains(&chunk_hash) { return Ok(()); }
            s.hashes.insert(chunk_hash.clone());
        }

        let node_id = format!("chunk:{}", chunk_hash);
        let mut meta = NormalizedMetadata::new(&path.to_string_lossy());
        meta.file_type = "text_chunk".to_string();
        meta.hash = chunk_hash;
        meta.timestamp = file_meta.modified()?.duration_since(std::time::UNIX_EPOCH)?.as_secs();

        if let Some(info) = detect(&text) {
             meta.language = info.lang().to_string();
        }

        let facts = SymbolicNLP::extract_deep_facts(&text);
        meta.insert("extracted_meaning_count", facts.len().to_string());

        let semantic_vec = self.encode_text_deterministic(&text);

        {
            let mut s = seen.lock().unwrap();
            for prev in s.semantic_vectors.iter() {
                if semantic_vec.similarity(prev) > 0.95 {
                    debug!("Smarter Duplicate Detection: Skipping semantically redundant chunk (sim > 0.95)");
                    return Ok(());
                }
            }
            s.semantic_vectors.push(semantic_vec.clone());
        }

        let mut g = graph.lock().unwrap();
        g.add_node_with_confidence(&node_id, semantic_vec, meta.to_map(), 1.0);
        for fact in facts {
            g.add_edge_with_confidence(&fact.subject, &fact.object, &fact.predicate, 1.0, 1.0);
        }
        Ok(())
    }
}
