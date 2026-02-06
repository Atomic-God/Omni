use cognition::CognitionCore;
use core_vsa::HyperVector;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;

#[allow(dead_code)]
const MEMORY_VERSION: &str = "5.1";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum LifecycleState {
    Fabricated,
    Frozen,
    Runtime,
}

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

#[derive(Serialize, Deserialize, Clone)]
pub struct LearningPolicies {
    pub reinforcement_rate: f32,
    pub decay_rate: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MindMetadata {
    pub os: String,
    pub arch: String,
    pub timestamp: u64,
    pub source: String,
    pub core_hash: String,
    pub state: LifecycleState,
}

#[derive(Serialize, Deserialize)]
pub struct MindPack {
    pub version: String,
    pub memory: MemoryStore,
    pub vocab: VocabStore,
    pub encoder_config: EncoderConfig,
    pub learning_policies: LearningPolicies,
    pub metadata: MindMetadata,
}

pub fn save_mind(mind: &MindPack, path: &str) -> Result<(), std::io::Error> {
    // Validate path
    let path_obj = std::path::Path::new(path);
    if let Some(parent) = path_obj.parent() {
        std::fs::create_dir_all(parent)?;
    }

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

    // Integrity Check (Schema Version)
    if mind.version != "5.1" {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Unsupported MindPack version"));
    }

    Ok(mind)
}

// Legacy helpers
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
