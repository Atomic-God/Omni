use crate::{MemoryManager, LSHIndex, ShardedStorage, MemoryEntry};
use serde::{Serialize, Deserialize};
use std::path::Path;
use std::fs::{self, File};
use sha2::{Sha256, Digest};
use flate2::write::GzEncoder;
use flate2::read::GzDecoder;
use flate2::Compression;
use std::collections::HashMap;
use tracing::info;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SnapshotHeader {
    pub version: u32,
    pub timestamp: u64,
    pub checksum: String,
    pub is_delta: bool,
    pub base_snapshot: Option<String>,
    pub previous_hash: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct MindSnapshot {
    pub header: SnapshotHeader,
    pub episodic_index: LSHIndex,
    pub semantic_index: LSHIndex,
    pub storage: ShardedStorage,
    pub metadata: HashMap<String, MemoryEntry>,
}

#[derive(Serialize, Deserialize)]
pub struct MindPack {
    pub snapshots: Vec<MindSnapshot>,
    pub manifest: HashMap<String, String>,
    pub integrity_hashes: HashMap<String, String>,
}

impl MindPack {
    pub fn new() -> Self {
        Self {
            snapshots: Vec::new(),
            manifest: HashMap::new(),
            integrity_hashes: HashMap::new(),
        }
    }

    pub fn export<T: Serialize>(&self, path: &Path, data: &T, key: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
        let temp_path = path.with_extension("tmp_pack");
        let file = File::create(&temp_path)?;
        let mut encoder = GzEncoder::new(file, Compression::best());

        let mut bytes = bincode::serialize(data)?;
        if let Some(k) = key {
            info!("Industrial Snapshot: Applying encryption layer.");
            self.xor_transform(&mut bytes, k);
        }

        std::io::Write::write_all(&mut encoder, &bytes)?;
        encoder.finish()?;

        fs::rename(temp_path, path)?;
        Ok(())
    }

    pub fn import<T: for<'de> Deserialize<'de>>(&self, path: &Path, key: Option<&str>) -> Result<T, Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let mut decoder = GzDecoder::new(file);
        let mut bytes = Vec::new();
        std::io::Read::read_to_end(&mut decoder, &mut bytes)?;

        if let Some(k) = key {
            info!("Industrial Snapshot: Removing encryption layer.");
            self.xor_transform(&mut bytes, k);
        }

        let data = bincode::deserialize(&bytes)?;
        Ok(data)
    }

    fn xor_transform(&self, data: &mut [u8], key: &str) {
        let key_bytes = key.as_bytes();
        for (i, byte) in data.iter_mut().enumerate() {
            *byte ^= key_bytes[i % key_bytes.len()];
        }
    }
}

pub struct SnapshotManager;

impl SnapshotManager {
    pub fn save(memory: &MemoryManager, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        Self::save_ext(memory, path, false, None, None)
    }

    pub fn save_delta(memory: &MemoryManager, path: &Path, base: &MindSnapshot) -> Result<(), Box<dyn std::error::Error>> {
        let base_hash = base.header.checksum.clone();

        let mut delta_metadata = HashMap::new();
        for (k, v) in &memory.metadata {
            if !base.metadata.contains_key(k) || base.metadata[k].importance != v.importance || base.metadata[k].reinforcement_count != v.reinforcement_count {
                delta_metadata.insert(k.clone(), v.clone());
            }
        }

        let body = MindSnapshotBody {
            episodic_index: memory.episodic_index.clone(), // In industrial version, LSHIndex should also be delta-encoded, but for Phase-1 we keep it simple
            semantic_index: memory.semantic_index.clone(),
            storage: memory.storage.clone(),
            metadata: delta_metadata,
        };

        Self::save_ext_with_body(body, path, true, Some(base_hash.clone()), Some(base_hash))
    }

    fn save_ext(memory: &MemoryManager, path: &Path, is_delta: bool, base_snapshot: Option<String>, previous_hash: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
        let body = MindSnapshotBody {
            episodic_index: memory.episodic_index.clone(),
            semantic_index: memory.semantic_index.clone(),
            storage: memory.storage.clone(),
            metadata: memory.metadata.clone(),
        };
        Self::save_ext_with_body(body, path, is_delta, base_snapshot, previous_hash)
    }

    fn save_ext_with_body(body: MindSnapshotBody, path: &Path, is_delta: bool, base_snapshot: Option<String>, previous_hash: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
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
            previous_hash,
        };

        // Transactional Save: Atomic rename
        let temp_path = path.with_extension("tmp_snap");
        let file = File::create(&temp_path)?;
        let mut encoder = GzEncoder::new(file, Compression::best());

        let container = MindSnapshotContainer {
            header,
            body,
        };

        bincode::serialize_into(&mut encoder, &container)?;
        encoder.finish()?;

        fs::rename(temp_path, path)?;
        info!("Industrial Core: Atomic Snapshot saved to {:?}", path);

        Ok(())
    }

    pub fn apply_delta(base: &mut MemoryManager, delta: &MindSnapshot) {
        for (k, v) in &delta.metadata {
            base.metadata.insert(k.clone(), v.clone());
            base.storage.insert(k, v.vector.clone());
            if v.layer == "semantic" {
                base.semantic_index.insert(k, v.vector.clone());
            } else {
                base.episodic_index.insert(k, v.vector.clone());
            }
        }
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
            return Err("Industrial Robustness Error: Snapshot checksum mismatch!".into());
        }

        Ok(MindSnapshot {
            header: container.header,
            episodic_index: container.body.episodic_index,
            semantic_index: container.body.semantic_index,
            storage: container.body.storage,
            metadata: container.body.metadata,
        })
    }

    pub fn diff(a: &MindSnapshot, b: &MindSnapshot) -> Vec<String> {
        let mut changes = Vec::new();
        for key in b.metadata.keys() {
            if !a.metadata.contains_key(key) {
                changes.push(format!("Added: {}", key));
            } else if a.metadata[key].importance != b.metadata[key].importance {
                changes.push(format!("Modified: {}", key));
            }
        }
        for key in a.metadata.keys() {
            if !b.metadata.contains_key(key) {
                changes.push(format!("Removed: {}", key));
            }
        }
        changes
    }

    pub fn repair(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let mut decoder = GzDecoder::new(file);
        let container: MindSnapshotContainer = bincode::deserialize_from(&mut decoder)?;
        let body_bytes = bincode::serialize(&container.body)?;
        let mut hasher = Sha256::new();
        hasher.update(&body_bytes);
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
struct MindSnapshotBody {
    episodic_index: LSHIndex,
    semantic_index: LSHIndex,
    storage: ShardedStorage,
    metadata: HashMap<String, MemoryEntry>,
}

#[derive(Serialize, Deserialize)]
struct MindSnapshotContainer {
    header: SnapshotHeader,
    body: MindSnapshotBody,
}
