use cognition::CognitionCore;
use core_vsa::HyperVector;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;

#[allow(dead_code)]
const MEMORY_VERSION: &str = "1.0";

#[derive(Serialize, Deserialize, Clone)]
pub struct VocabStore {
    pub words: HashMap<String, HyperVector>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MemoryStore {
    pub core: CognitionCore,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct EncoderConfig {
    pub model_name: String,
}

#[derive(Serialize, Deserialize)]
pub struct MindPack {
    pub version: String,
    pub memory: MemoryStore,
    pub vocab: VocabStore,
    pub encoder_config: EncoderConfig,
}

pub fn save_mind(mind: &MindPack, path: &str) -> Result<(), std::io::Error> {
    let file = File::create(path)?;
    let mut zip = zip::ZipWriter::new(file);
    let options =
        zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Stored);

    zip.start_file("mind.json", options)?;
    serde_json::to_writer(&mut zip, mind)?;
    zip.finish()?;
    Ok(())
}

pub fn load_mind(path: &str) -> Result<MindPack, std::io::Error> {
    let file = File::open(path)?;
    let mut archive = zip::ZipArchive::new(file)?;
    let file = archive.by_name("mind.json")?;
    let mind: MindPack = serde_json::from_reader(file)?;
    Ok(mind)
}

// Legacy helpers if needed, or remove. Keeping for compatibility or internal use.
pub fn save_core(core: &CognitionCore, path: &str) -> Result<(), std::io::Error> {
    let file = File::create(path)?;
    serde_json::to_writer(file, core)?;
    Ok(())
}

pub fn load_core(path: &str) -> Result<CognitionCore, std::io::Error> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let core: CognitionCore = serde_json::from_reader(reader)?;
    Ok(core)
}
