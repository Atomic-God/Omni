use cognition::CognitionCore;
use core_vsa::HyperVector;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read, Write};
use std::path::Path;

pub mod io;
pub mod invariant;
pub mod streaming;
use invariant::UniversalMindInvariant;

#[allow(dead_code)]
const MEMORY_VERSION: &str = "8.4";

// --- Forge Artifacts (Immutable) ---

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
    pub max_concepts: Option<usize>, // None = unlimited (Forge)
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MindMetadata {
    pub os: String,
    pub arch: String,
    pub timestamp: u64,
    pub source: String,
    pub core_hash: String,
    pub state: LifecycleState,
    pub compiler_version: String,
    pub semantic_version: String, // SemVer (e.g. 1.0.0)
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MindManifest { // Renamed from MindBlueprint to avoid confusion with the artifact
    pub name: String,
    pub base_version: String,
    pub required_capabilities: Vec<String>,
    pub overlay_policy: String, // e.g. "append_only", "ephemeral"
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MindPack {
    pub version: String, // Internal schema version
    pub memory: MemoryStore,
    pub vocab: VocabStore,
    pub encoder_config: EncoderConfig,
    pub learning_policies: LearningPolicies,
    pub metadata: MindMetadata,
    pub manifest: Option<MindManifest>,
}

// Aliases for "Snapshot & Cloning Standard"
pub type MindBlueprint = MindPack; // The Immutable Base
pub type MindDelta = PersonalMemory; // The Personal Learning

// --- Snapshot Management ---

pub fn save_snapshot(mind: &MindPack, path: &str) -> Result<(), std::io::Error> {
    let path_obj = Path::new(path);
    if let Some(parent) = path_obj.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let file = File::create(path)?;
    let mut zip = zip::ZipWriter::new(file);
    // Use Deflated for universal lossless compression (pkzip compatible)
    let options = zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    // 1. Metadata
    zip.start_file("metadata.json", options)?;
    serde_json::to_writer_pretty(&mut zip, &mind.metadata)?;

    // 2. Limits / Schema
    zip.start_file("limits.json", options)?;
    serde_json::to_writer_pretty(&mut zip, &mind.learning_policies)?;

    zip.start_file("schema.json", options)?;
    serde_json::to_writer_pretty(&mut zip, &mind.encoder_config)?;

    // 3. Manifest
    if let Some(manifest) = &mind.manifest {
        zip.start_file("manifest.json", options)?;
        serde_json::to_writer_pretty(&mut zip, manifest)?;
    }

    // 4. Binary Core
    zip.start_file("mind.bin", options)?;
    let bin_config = bincode::config::standard();
    bincode::serde::encode_into_std_write(&mind.memory.core, &mut zip, bin_config)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

    // 5. Integrity
    zip.start_file("integrity.hash", options)?;
    zip.write_all(mind.metadata.core_hash.as_bytes())?;

    zip.finish()?;
    Ok(())
}

pub fn load_snapshot(path: &str) -> Result<MindPack, std::io::Error> {
    let file = File::open(path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    // 1. Metadata
    let metadata: MindMetadata = {
        let file = archive.by_name("metadata.json")?;
        serde_json::from_reader(file)?
    };

    // 2. Core
    let core: CognitionCore = {
        let mut file = archive.by_name("mind.bin")?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        let bin_config = bincode::config::standard();
        bincode::serde::decode_from_slice(&buffer, bin_config)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?.0
    };

    // 3. Configs
    let learning_policies: LearningPolicies = {
        let file = archive.by_name("limits.json")?;
        serde_json::from_reader(file)?
    };

    let encoder_config: EncoderConfig = {
        let file = archive.by_name("schema.json")?;
        serde_json::from_reader(file)?
    };

    let mut manifest: Option<MindManifest> = if let Ok(file) = archive.by_name("manifest.json") {
        Some(serde_json::from_reader(file)?)
    } else {
        None
    };

    if manifest.is_none() {
        if let Ok(file) = archive.by_name("blueprint.json") {
            manifest = Some(serde_json::from_reader(file)?);
        }
    }

    // 4. Integrity Check
    let stored_hash = {
        let mut file = archive.by_name("integrity.hash")?;
        let mut s = String::new();
        file.read_to_string(&mut s)?;
        s
    };

    if stored_hash != metadata.core_hash {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Snapshot Integrity Violation: Hash Mismatch"));
    }

    let pack = MindPack {
        version: MEMORY_VERSION.to_string(),
        memory: MemoryStore { core: core.clone() },
        vocab: VocabStore { words: core.index_memory.clone() },
        encoder_config,
        learning_policies,
        metadata,
        manifest, // Renamed field
    };

    if let Err(e) = UniversalMindInvariant::check(&pack) {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, e));
    }

    Ok(pack)
}

// --- Runtime Personal Memory (Overlay) ---

#[derive(Serialize, Deserialize, Clone)]
pub struct PersonalMemory {
    pub core: CognitionCore,
    pub parent_hash: String,
    pub created_at: u64,
    pub last_accessed: u64,
}

pub fn save_personal(mem: &PersonalMemory, path: &str) -> Result<(), std::io::Error> {
    let mut file = File::create(path)?;
    let bin_config = bincode::config::standard();
    bincode::serde::encode_into_std_write(mem, &mut file, bin_config)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
    Ok(())
}

pub fn load_personal(path: &str) -> Result<PersonalMemory, std::io::Error> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer)?;

    let bin_config = bincode::config::standard();
    let (mem, _): (PersonalMemory, usize) = bincode::serde::decode_from_slice(&buffer, bin_config)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
    Ok(mem)
}

// Legacy Aliases
pub use load_snapshot as load_mind;
pub use save_snapshot as save_mind;
