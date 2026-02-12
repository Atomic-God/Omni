use crate::MemorySystem;
use core_vsa::HyperVector;
use std::path::Path;
use std::fs;
use serde::{Serialize, Deserialize};

pub mod lsh;
pub mod storage;
pub mod snapshot;
pub mod consolidation; // Added

// Re-export
pub use lsh::LSHIndex;
pub use storage::ShardedStorage;
pub use snapshot::SnapshotManager;
pub use consolidation::Consolidator;

// ... (Rest of lib.rs logic, ensuring MemorySystem is visible)
use std::error::Error;
use core_vsa::traits::MemoryStore;

pub struct MemorySystem {
    pub index: LSHIndex,
    pub storage: ShardedStorage,
}

impl MemorySystem {
    pub fn new(path: &Path) -> Self {
        Self {
            index: LSHIndex::new(),
            storage: ShardedStorage::new(path),
        }
    }
}

impl MemoryStore for MemorySystem {
    fn store(&mut self, key: &str, vector: HyperVector) -> Result<(), Box<dyn Error>> {
        self.index.insert(key, vector.clone());
        self.storage.insert(key, vector);
        Ok(())
    }

    fn retrieve(&self, key: &str) -> Option<HyperVector> {
        if let Some(v) = self.storage.current_shard_buffer.get(key) {
             return Some(v.clone());
        }
        self.index.vectors.get(key).cloned()
    }

    fn query_nearest(&self, query: &HyperVector, k: usize) -> Vec<(String, f32)> {
        self.index.query(query, k)
    }
}
