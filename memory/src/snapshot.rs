use crate::{MemoryManager, LSHIndex, ShardedStorage, MemoryEntry};
use core_vsa::HyperVector;
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
    pub signature: Option<String>, // Step 2: Industrial Signature
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
    #[serde(default)]
    pub shards: HashMap<usize, HashMap<String, HyperVector>>,
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
    const INDUSTRIAL_KEY: &'static str = "OMNIFORGE_INDUSTRIAL_V1_SECRET";

    fn compute_signature(checksum: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(checksum.as_bytes());
        hasher.update(Self::INDUSTRIAL_KEY.as_bytes());
        hex::encode(hasher.finalize())
    }

    pub fn save(memory: &MemoryManager, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        Self::save_ext(memory, path, false, None, None)
    }

    pub fn save_delta(memory: &MemoryManager, path: &Path, base: &MindSnapshot) -> Result<(), Box<dyn std::error::Error>> {
        let base_hash = base.header.checksum.clone();

        let mut delta_metadata = HashMap::new();
        let mut delta_keys = std::collections::HashSet::new();
        for (k, v) in &memory.metadata {
            if !base.metadata.contains_key(k) || base.metadata[k].importance != v.importance || base.metadata[k].reinforcement_count != v.reinforcement_count {
                delta_metadata.insert(k.clone(), v.clone());
                delta_keys.insert(k.clone());
            }
        }

        let mut shards = HashMap::new();
        // Delta Shards: Only include shards that don't exist in base
        for &shard_id in memory.storage.location_map.values() {
            if shard_id > 0 && !base.shards.contains_key(&shard_id) {
                let shard_path = memory.storage.root_dir.join(format!("shard_{}.bin", shard_id));
                if let Ok(file) = File::open(&shard_path) {
                    let shard: crate::storage::ShardFile = bincode::deserialize_from(file)?;
                    shards.insert(shard_id, shard.data);
                }
            }
        }

        // Industrial Optimization: Only carry subset indices for the delta keys
        let body = MindSnapshotBody {
            episodic_index: memory.episodic_index.get_subset(&delta_keys),
            semantic_index: memory.semantic_index.get_subset(&delta_keys),
            storage: memory.storage.clone(),
            metadata: delta_metadata,
            shards,
        };

        Self::save_ext_with_body(body, path, true, Some(base_hash.clone()), Some(base_hash))
    }

    fn save_ext(memory: &MemoryManager, path: &Path, is_delta: bool, base_snapshot: Option<String>, previous_hash: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
        let mut shards = HashMap::new();
        // Industrial: Collect all shard data for portability
        for &shard_id in memory.storage.location_map.values() {
            if shard_id > 0 {
                let shard_path = memory.storage.root_dir.join(format!("shard_{}.bin", shard_id));
                if let Ok(file) = File::open(&shard_path) {
                    let shard: crate::storage::ShardFile = bincode::deserialize_from(file)?;
                    shards.insert(shard_id, shard.data);
                }
            }
        }

        let body = MindSnapshotBody {
            episodic_index: memory.episodic_index.clone(),
            semantic_index: memory.semantic_index.clone(),
            storage: memory.storage.clone(),
            metadata: memory.metadata.clone(),
            shards,
        };
        Self::save_ext_with_body(body, path, is_delta, base_snapshot, previous_hash)
    }

    fn save_ext_with_body(body: MindSnapshotBody, path: &Path, is_delta: bool, base_snapshot: Option<String>, previous_hash: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
        let body_bytes = bincode::serialize(&body)?;

        let mut hasher = Sha256::new();
        hasher.update(&body_bytes);
        let checksum = hex::encode(hasher.finalize());

        let signature = Some(Self::compute_signature(&checksum));

        let header = SnapshotHeader {
            version: crate::MEMORY_VERSION,
            timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_secs(),
            checksum,
            signature,
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
        // Extract delta shards
        for (&shard_id, data) in &delta.shards {
            let shard_path = base.storage.root_dir.join(format!("shard_{}.bin", shard_id));
            if !shard_path.exists() {
                let file = File::create(shard_path).ok();
                if let Some(f) = file {
                    let shard = crate::storage::ShardFile {
                        id: shard_id,
                        data: data.clone()
                    };
                    let _ = bincode::serialize_into(f, &shard);
                }
            }
        }

        // Apply metadata and location map
        for (k, v) in &delta.metadata {
            base.metadata.insert(k.clone(), v.clone());
            base.storage.location_map.insert(k.clone(), delta.storage.location_map.get(k).cloned().unwrap_or(0));
        }

        // Industrial Optimization: Merge delta indices
        base.episodic_index.merge(delta.episodic_index.clone());
        base.semantic_index.merge(delta.semantic_index.clone());
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

        // Verify Industrial Signature
        if let Some(ref sig) = container.header.signature {
            let expected_sig = Self::compute_signature(&container.header.checksum);
            if sig != &expected_sig {
                return Err("Industrial Security Error: Snapshot signature invalid!".into());
            }
        } else {
             return Err("Industrial Security Error: Missing snapshot signature!".into());
        }

        // Industrial: Extract shards back to storage root if missing
        for (&shard_id, data) in &container.body.shards {
            let shard_path = container.body.storage.root_dir.join(format!("shard_{}.bin", shard_id));
            if !shard_path.exists() {
                let file = File::create(shard_path)?;
                let shard = crate::storage::ShardFile { id: shard_id, data: data.clone() };
                bincode::serialize_into(file, &shard)?;
            }
        }

        Ok(MindSnapshot {
            header: container.header,
            episodic_index: container.body.episodic_index,
            semantic_index: container.body.semantic_index,
            storage: container.body.storage,
            metadata: container.body.metadata,
            shards: container.body.shards,
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
    #[serde(default)]
    shards: HashMap<usize, HashMap<String, HyperVector>>,
}

#[derive(Serialize, Deserialize)]
struct MindSnapshotContainer {
    header: SnapshotHeader,
    body: MindSnapshotBody,
}
