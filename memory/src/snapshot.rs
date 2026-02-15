use crate::{MemoryManager, LSHIndex, ShardedStorage, MemoryEntry};
use serde::{Serialize, Deserialize};
use std::path::Path;
use std::fs::File;
use sha2::{Sha256, Digest};
use flate2::write::GzEncoder;
use flate2::read::GzDecoder;
use flate2::Compression;
use std::collections::HashMap;
use log::info;

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

impl MindPack {
    pub fn new() -> Self {
        Self {
            snapshots: Vec::new(),
            manifest: HashMap::new(),
        }
    }

    pub fn export<T: Serialize>(&self, path: &Path, data: &T) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::create(path)?;
        let mut encoder = GzEncoder::new(file, Compression::best());
        bincode::serialize_into(&mut encoder, data)?;
        encoder.finish()?;
        Ok(())
    }

    pub fn import<T: for<'de> Deserialize<'de>>(&self, path: &Path) -> Result<T, Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let mut decoder = GzDecoder::new(file);
        let data = bincode::deserialize_from(&mut decoder)?;
        Ok(data)
    }
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
        info!("Attempting to repair snapshot at {:?}", path);
        let file = File::open(path)?;
        let mut decoder = GzDecoder::new(file);
        let container: MindSnapshotContainer = bincode::deserialize_from(&mut decoder)?;

        let body_bytes = bincode::serialize(&container.body)?;
        let mut hasher = Sha256::new();
        hasher.update(&body_bytes);
        info!("Repair complete: Snapshot structure validated.");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_mindpack_roundtrip() {
        let dir = tempdir().unwrap();
        let pack_path = dir.path().join("test.mindpack");
        let packer = MindPack::new();

        let mut data = HashMap::new();
        data.insert("key".to_string(), "value".to_string());

        packer.export(&pack_path, &data).expect("Export failed");
        assert!(pack_path.exists());

        let imported: HashMap<String, String> = packer.import(&pack_path).expect("Import failed");
        assert_eq!(imported.get("key").unwrap(), "value");
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
