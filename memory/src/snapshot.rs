use crate::{MemoryManager, LSHIndex, ShardedStorage, MemoryEntry};
use serde::{Serialize, Deserialize};
use std::path::Path;
use std::fs::File;
use std::io::{Read, Write};
use sha2::{Sha256, Digest};
use flate2::write::GzEncoder;
use flate2::read::GzDecoder;
use flate2::Compression;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SnapshotHeader {
    pub version: u32,
    pub timestamp: u64,
    pub checksum: String,
    pub is_delta: bool,
    pub base_snapshot: Option<String>, // Hash of base snapshot
}

#[derive(Serialize, Deserialize)]
pub struct MindSnapshot {
    pub header: SnapshotHeader,
    pub index: LSHIndex,
    pub storage: ShardedStorage,
    pub metadata: HashMap<String, MemoryEntry>,
}

#[derive(Serialize, Deserialize)]
pub struct MindPack {
    pub snapshots: Vec<MindSnapshot>,
    pub manifest: HashMap<String, String>,
}

pub struct SnapshotManager;

impl SnapshotManager {
    pub fn save(memory: &MemoryManager, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        Self::save_ext(memory, path, false, None)
    }

    pub fn save_delta(memory: &MemoryManager, path: &Path, base_hash: &str) -> Result<(), Box<dyn std::error::Error>> {
        Self::save_ext(memory, path, true, Some(base_hash.to_string()))
    }

    fn save_ext(memory: &MemoryManager, path: &Path, is_delta: bool, base_snapshot: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
        let body = MindSnapshotBody {
            index: memory.index.clone(),
            storage: memory.storage.clone(),
            metadata: memory.metadata.clone(),
        };

        let body_bytes = bincode::serialize(&body)?;

        let mut hasher = Sha256::new();
        hasher.update(&body_bytes);
        let checksum = hex::encode(hasher.finalize());

        let header = SnapshotHeader {
            version: crate::MEMORY_VERSION,
            timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_secs(),
            checksum,
            is_delta,
            base_snapshot,
        };

        let file = File::create(path)?;
        let mut encoder = GzEncoder::new(file, Compression::best());

        let container = MindSnapshotContainer {
            header,
            body,
        };

        bincode::serialize_into(&mut encoder, &container)?;
        encoder.finish()?;

        Ok(())
    }

    pub fn load(path: &Path) -> Result<MindSnapshot, Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let mut decoder = GzDecoder::new(file);

        let container: MindSnapshotContainer = bincode::deserialize_from(&mut decoder)?;

        let body_bytes = bincode::serialize(&container.body)?;
        let mut hasher = Sha256::new();
        hasher.update(&body_bytes);
        let computed = hex::encode(hasher.finalize());

        if computed != container.header.checksum {
            return Err("Snapshot checksum mismatch! File corrupted.".into());
        }

        Ok(MindSnapshot {
            header: container.header,
            index: container.body.index,
            storage: container.body.storage,
            metadata: container.body.metadata,
        })
    }
}

#[derive(Serialize, Deserialize)]
struct MindSnapshotBody {
    index: LSHIndex,
    storage: ShardedStorage,
    metadata: HashMap<String, MemoryEntry>,
}

#[derive(Serialize, Deserialize)]
struct MindSnapshotContainer {
    header: SnapshotHeader,
    body: MindSnapshotBody,
}
