use core_vsa::HyperVector;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use serde::{Serialize, Deserialize};
use std::fs;
use sha2::{Sha256, Digest};
use log::info;

pub mod lsh;
pub mod storage;
pub mod snapshot;
pub mod consolidation;
pub mod recovery;
pub mod hierarchy;
pub mod tests; // Tests module

pub use lsh::LSHIndex;
pub use storage::ShardedStorage;
pub use snapshot::{SnapshotManager, MindSnapshot, SnapshotHeader};
pub use consolidation::Consolidator;
pub use recovery::CorruptionRecovery;
pub use hierarchy::{MemoryLayer, MemoryEntry, HierarchicalMemory};

use std::error::Error;
use core_vsa::traits::MemoryStore;

// Version 1.0 (Binary)
pub const MEMORY_VERSION: u32 = 1;

pub struct MemoryManager {
    pub index: LSHIndex,
    pub storage: ShardedStorage,
    pub root_dir: PathBuf,
    pub metadata: HashMap<String, MemoryEntry>,
}

impl MemoryManager {
    pub fn new(root_dir: &Path) -> Self {
        fs::create_dir_all(root_dir).unwrap();
        Self {
            index: LSHIndex::new(),
            storage: ShardedStorage::new(root_dir),
            root_dir: root_dir.to_path_buf(),
            metadata: HashMap::new(),
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
        Ok(())
    }

    pub fn verify(&self) -> bool {
        CorruptionRecovery::check_integrity(&self.storage)
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
