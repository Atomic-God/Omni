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
pub mod recovery; // New

pub use lsh::LSHIndex;
pub use storage::ShardedStorage;
pub use snapshot::{SnapshotManager, MindSnapshot, SnapshotHeader};
pub use consolidation::Consolidator;
pub use recovery::CorruptionRecovery;

use std::error::Error;
use core_vsa::traits::MemoryStore;

// Version 1.0 (Binary)
pub const MEMORY_VERSION: u32 = 1;

pub struct MemoryManager {
    pub index: LSHIndex,
    pub storage: ShardedStorage,
    pub root_dir: PathBuf,
}

impl MemoryManager {
    pub fn new(root_dir: &Path) -> Self {
        fs::create_dir_all(root_dir).unwrap();
        Self {
            index: LSHIndex::new(),
            storage: ShardedStorage::new(root_dir),
            root_dir: root_dir.to_path_buf(),
        }
    }

    pub fn save_snapshot(&self, name: &str) -> Result<(), Box<dyn Error>> {
        let snapshot_path = self.root_dir.join(format!("{}.snap", name));
        SnapshotManager::save(self, &snapshot_path)
    }

    pub fn load_snapshot(&mut self, name: &str) -> Result<(), Box<dyn Error>> {
        let snapshot_path = self.root_dir.join(format!("{}.snap", name));
        let snapshot = SnapshotManager::load(&snapshot_path)?;

        // Restore
        // 1. Storage
        self.storage = snapshot.storage;
        // 2. Index (Rebuild? Or load if serialized?)
        // Snapshot struct should contain index data or we rebuild.
        // For "Industrial", rebuilding LSH from millions of items is slow.
        // So we serialize LSH tables.
        self.index = snapshot.index;

        Ok(())
    }

    pub fn verify(&self) -> bool {
        // Run corruption check
        CorruptionRecovery::check_integrity(&self.storage)
    }
}

impl MemoryStore for MemoryManager {
    fn store(&mut self, key: &str, vector: HyperVector) -> Result<(), Box<dyn Error>> {
        self.index.insert(key, vector.clone());
        self.storage.insert(key, vector);
        Ok(())
    }

    fn retrieve(&self, key: &str) -> Option<HyperVector> {
        if let Some(v) = self.storage.retrieve(key) { // Uses RefCell internally now? No, storage needs refactor.
             // ShardedStorage::retrieve needs &mut self if it manages LRU cache.
             // If stateless read, &self is fine.
             // My previous `storage.rs` used `&mut self` for `retrieve`.
             // This conflicts with `MemoryStore` trait `&self`.
             // I must fix `ShardedStorage` to use interior mutability or stateless read.
             // For Phase 3, I'll use interior mutability in `ShardedStorage` (RefCell/Mutex).
             return Some(v.clone());
        }
        None
    }

    fn query_nearest(&self, query: &HyperVector, k: usize) -> Vec<(String, f32)> {
        self.index.query(query, k)
    }
}
