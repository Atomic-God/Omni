use cognition::CognitionCore;
use core_vsa::HyperVector;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read, Write};

#[allow(dead_code)]
const MEMORY_VERSION: &str = "5.2";

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

#[derive(Serialize, Deserialize, Clone)]
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

    // 1. Metadata (JSON - Human Readable)
    zip.start_file("metadata.json", options)?;
    serde_json::to_writer(&mut zip, &mind.metadata)?;

    // 2. Cognition Core (Binary - Compact/Fast)
    zip.start_file("memory.bin", options)?;
    let bin_config = bincode::config::standard();
    bincode::serde::encode_into_std_write(&mind.memory.core, &mut zip, bin_config)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

    // 3. Encoder Config & Policies (JSON)
    zip.start_file("config.json", options)?;
    serde_json::to_writer(&mut zip, &mind.encoder_config)?;
    zip.start_file("policies.json", options)?;
    serde_json::to_writer(&mut zip, &mind.learning_policies)?;

    // 4. Integrity Hash (Text)
    zip.start_file("integrity.hash", options)?;
    zip.write_all(mind.metadata.core_hash.as_bytes())?;

    zip.finish()?;
    Ok(())
}

pub fn load_mind(path: &str) -> Result<MindPack, std::io::Error> {
    let file = File::open(path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    // 1. Metadata
    let metadata: MindMetadata = {
        let meta_file = archive.by_name("metadata.json")?;
        serde_json::from_reader(meta_file)?
    };

    // 2. Memory
    let core: CognitionCore = {
        let mut mem_file = archive.by_name("memory.bin")?;
        let mut mem_buf = Vec::new();
        mem_file.read_to_end(&mut mem_buf)?;

        let bin_config = bincode::config::standard();
        bincode::serde::decode_from_slice(&mem_buf, bin_config)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?.0
    };

    // 3. Configs
    let encoder_config: EncoderConfig = {
        let config_file = archive.by_name("config.json")?;
        serde_json::from_reader(config_file)?
    };

    let learning_policies: LearningPolicies = {
        let policies_file = archive.by_name("policies.json")?;
        serde_json::from_reader(policies_file)?
    };

    // 4. Verify Integrity
    let stored_hash = {
        let mut hash_file = archive.by_name("integrity.hash")?;
        let mut stored_hash = String::new();
        hash_file.read_to_string(&mut stored_hash)?;
        stored_hash
    };

    if stored_hash != metadata.core_hash {
         return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Integrity Check Failed: Hash mismatch"));
    }

    let recomputed = core.compute_integrity_hash();
    if recomputed != stored_hash {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Integrity Check Failed: Content modified"));
    }

    Ok(MindPack {
        version: "5.2".to_string(),
        memory: MemoryStore { core: core.clone() },
        vocab: VocabStore { words: core.index_memory.clone() },
        encoder_config,
        learning_policies,
        metadata,
    })
}

// Legacy helpers (Deprecated)
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
