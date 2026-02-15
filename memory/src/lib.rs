use core_vsa::HyperVector;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs::File;
use std::fs;
use log::info;

pub mod lsh;
pub mod storage;
pub mod snapshot;
// pub mod consolidation;
pub mod recovery;
pub mod hierarchy;
pub mod tests;
pub mod layered; // New

pub use lsh::LSHIndex;
pub use storage::ShardedStorage;
pub use snapshot::{SnapshotManager, MindSnapshot, SnapshotHeader, MindPack};
// pub use consolidation::Consolidator;
pub use recovery::CorruptionRecovery;
pub use hierarchy::{MemoryLayer, MemoryEntry, HierarchicalMemory};
pub use layered::LayeredMemory;

use std::error::Error;
use core_vsa::traits::MemoryStore;

// Version 1.0 (Binary)
pub const MEMORY_VERSION: u32 = 1;

pub struct MemoryManager {
    pub index: LSHIndex,
    pub storage: ShardedStorage,
    pub root_dir: PathBuf,
    pub metadata: HashMap<String, MemoryEntry>,
    pub low_memory_mode: bool, // Added
}

impl MemoryManager {
    pub fn new(root_dir: &Path) -> Self {
        fs::create_dir_all(root_dir).unwrap();
        Self {
            index: LSHIndex::new(),
            storage: ShardedStorage::new(root_dir),
            root_dir: root_dir.to_path_buf(),
            metadata: HashMap::new(),
            low_memory_mode: false,
        }
    }

    pub fn set_low_memory_mode(&mut self, enabled: bool) {
        self.low_memory_mode = enabled;
        if enabled {
            info!("Aggressive memory management enabled.");
            self.storage.shard_capacity = 100; // Smaller shards for low RAM
        }
    }

    pub fn save_snapshot(&self, name: &str) -> Result<(), Box<dyn Error>> {
        let snapshot_path = self.root_dir.join(format!("{}.snap", name));
        SnapshotManager::save(self, &snapshot_path)
    }

    pub fn load_snapshot(&mut self, name: &str) -> Result<(), Box<dyn Error>> {
        let snapshot_path = self.root_dir.join(format!("{}.snap", name));
        let snapshot = SnapshotManager::load(&snapshot_path)?;
        self.storage = snapshot.storage;
        self.index = snapshot.index;
        self.metadata = snapshot.metadata;
        Ok(())
    }

    pub fn save_delta(&self, name: &str, base_hash: &str) -> Result<(), Box<dyn Error>> {
        let snapshot_path = self.root_dir.join(format!("{}.delta", name));
        SnapshotManager::save_delta(self, &snapshot_path, base_hash)
    }

    pub fn export_mindpack(&self, path: &Path) -> Result<(), Box<dyn Error>> {
        let snapshot_path = self.root_dir.join("main.snap");
        self.save_snapshot("main")?;
        let snapshot = SnapshotManager::load(&snapshot_path)?;

        let pack = MindPack {
            snapshots: vec![snapshot],
            manifest: HashMap::from([
                ("engine_version".to_string(), "10.0".to_string()),
                ("created_by".to_string(), "OmniForge".to_string()),
            ]),
        };

        let file = File::create(path)?;
        bincode::serialize_into(file, &pack)?;
        Ok(())
    }

    pub fn verify(&self) -> bool {
        CorruptionRecovery::check_integrity(&self.storage)
    }

    pub fn apply_aging(&mut self, decay_rate: f32, prune_threshold: f32) {
        let _now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
        let mut to_remove = Vec::new();

        for (key, entry) in self.metadata.iter_mut() {
            entry.decay(decay_rate);

            if entry.importance < prune_threshold && entry.layer == "working" {
                to_remove.push(key.clone());
            }
        }

        for key in to_remove {
            info!("Aging system pruning low-relevance memory: {}", key);
            self.metadata.remove(&key);
            // In a full implementation, we'd also remove from LSH and storage
        }

        if self.metadata.len() > 10000 {
            info!("Memory capacity limit reached, triggering compression...");
            self.consolidate_layers();
        }
    }
}

impl MemoryStore for MemoryManager {
    fn store(&mut self, key: &str, vector: HyperVector) -> Result<(), Box<dyn Error>> {
        self.index.insert(key, vector.clone());
        self.storage.insert(key, vector.clone());
        self.metadata.insert(key.to_string(), MemoryEntry::new(vector, MemoryLayer::Working));
        Ok(())
    }

    fn retrieve(&self, key: &str) -> Option<HyperVector> {
        if let Some(v) = self.storage.retrieve(key) {
             return Some(v.clone());
        }
        None
    }

    fn query_nearest(&self, query: &HyperVector, k: usize) -> Vec<(String, f32)> {
        self.index.query(query, k)
    }
}

impl HierarchicalMemory for MemoryManager {
    fn store_in_layer(&mut self, key: &str, vector: HyperVector, layer: MemoryLayer) -> Result<(), Box<dyn Error>> {
        self.index.insert(key, vector.clone());
        self.storage.insert(key, vector.clone());
        self.metadata.insert(key.to_string(), MemoryEntry::new(vector, layer));
        Ok(())
    }

    fn retrieve_with_metrics(&mut self, key: &str) -> Option<HyperVector> {
        if let Some(entry) = self.metadata.get_mut(key) {
            entry.update_access();
            return Some(entry.vector.clone());
        }
        self.retrieve(key)
    }

    fn consolidate_layers(&mut self) {
        let mut changes = Vec::new();

        for (key, entry) in &self.metadata {
            if entry.layer == "working" && entry.importance > 5.0 {
                changes.push((key.clone(), MemoryLayer::Episodic));
            } else if entry.layer == "episodic" && entry.importance > 20.0 {
                changes.push((key.clone(), MemoryLayer::Invariant));
            }
        }

        for (key, new_layer) in changes {
            if let Some(entry) = self.metadata.get_mut(&key) {
                entry.layer = match new_layer {
                    MemoryLayer::Working => "working",
                    MemoryLayer::Episodic => "episodic",
                    MemoryLayer::Invariant => "invariant",
                }.to_string();
                info!("Consolidated {} to {}", key, entry.layer);
            }
        }
    }
}
