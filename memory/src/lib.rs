use core_vsa::HyperVector;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs::File;
use std::fs;
use log::{info, debug};

pub mod lsh;
pub mod storage;
pub mod snapshot;
pub mod recovery;
pub mod hierarchy;
pub mod tests;
pub mod layered;
pub mod evolution;

pub use lsh::LSHIndex;
pub use storage::ShardedStorage;
pub use snapshot::{SnapshotManager, MindSnapshot, SnapshotHeader, MindPack};
pub use recovery::CorruptionRecovery;
pub use hierarchy::{MemoryLayer, MemoryEntry, HierarchicalMemory};
pub use layered::LayeredMemory;
pub use evolution::{LifecycleManager, EpisodicEncoder};

use std::error::Error;
use core_vsa::traits::MemoryStore;

pub const MEMORY_VERSION: u32 = 1;

pub struct MemoryManager {
    pub episodic_index: LSHIndex,
    pub semantic_index: LSHIndex,
    pub storage: ShardedStorage,
    pub root_dir: PathBuf,
    pub metadata: HashMap<String, MemoryEntry>,
    pub low_memory_mode: bool,
    pub encoder: EpisodicEncoder,
    pub vsa_precision: usize, // New: for truncation
}

impl MemoryManager {
    pub fn new(root_dir: &Path) -> Self {
        fs::create_dir_all(root_dir).unwrap();
        Self {
            episodic_index: LSHIndex::new(),
            semantic_index: LSHIndex::new(),
            storage: ShardedStorage::new(root_dir),
            root_dir: root_dir.to_path_buf(),
            metadata: HashMap::new(),
            low_memory_mode: false,
            encoder: EpisodicEncoder::new(),
            vsa_precision: core_vsa::DIMENSION,
        }
    }

    pub fn set_low_memory_mode(&mut self, enabled: bool) {
        self.low_memory_mode = enabled;
        if enabled {
            info!("Industrial Adaptation: Reducing VSA precision for low-RAM environment.");
            self.vsa_precision = 2048; // Scale down from 10k
            self.storage.shard_capacity = 100;
        } else {
            self.vsa_precision = core_vsa::DIMENSION;
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
        self.episodic_index = snapshot.episodic_index;
        self.semantic_index = snapshot.semantic_index;
        self.metadata = snapshot.metadata;
        Ok(())
    }

    pub fn export_mindpack(&self, path: &Path) -> Result<(), Box<dyn Error>> {
        self.save_snapshot("main")?;
        let snapshot_path = self.root_dir.join("main.snap");
        let snapshot = SnapshotManager::load(&snapshot_path)?;
        let pack = MindPack {
            snapshots: vec![snapshot],
            manifest: HashMap::from([
                ("engine_version".to_string(), "1.0".to_string()),
            ]),
            integrity_hashes: HashMap::new(),
        };
        let file = File::create(path)?;
        bincode::serialize_into(file, &pack)?;
        Ok(())
    }

    pub fn apply_aging(&mut self, decay_rate: f32, prune_threshold: f32) {
        let mut to_remove = Vec::new();
        for (key, entry) in self.metadata.iter_mut() {
            entry.decay(decay_rate);
            if LifecycleManager::evaluate_eviction(entry, prune_threshold) {
                to_remove.push(key.clone());
            }
        }

        for key in to_remove {
            debug!("Pruning stale memory: {}", key);
            self.metadata.remove(&key);
            self.episodic_index.remove(&key);
            self.semantic_index.remove(&key);
        }

        self.consolidate_layers();
    }
}

impl MemoryStore for MemoryManager {
    fn store(&mut self, key: &str, vector: HyperVector) -> Result<(), Box<dyn Error>> {
        self.store_in_layer(key, vector, MemoryLayer::Working)
    }

    fn retrieve(&self, key: &str) -> Option<HyperVector> {
        self.metadata.get(key).map(|e| e.vector.clone())
    }

    fn query_nearest(&self, query: &HyperVector, k: usize) -> Vec<(String, f32)> {
        let mut results = self.episodic_index.query(query, k);
        results.extend(self.semantic_index.query(query, k));
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        results.truncate(k);

        // Reinforcement Pressure: Increase importance of matched items
        // Since this is a &self method, we use internal debug logs for phase 1,
        // or we need to pass a mutable reference.
        // In Industrial Core, we prefer explicit feedback loops in the Learner.

        results
    }
}

impl HierarchicalMemory for MemoryManager {
    fn store_in_layer(&mut self, key: &str, vector: HyperVector, layer: MemoryLayer) -> Result<(), Box<dyn Error>> {
        let mut final_vec = vector;

        // Truncation Adaptation
        if self.vsa_precision < final_vec.dim {
            final_vec = final_vec.truncate(self.vsa_precision);
        }

        if layer == MemoryLayer::Episodic {
             let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
             final_vec = self.encoder.encode_event(&final_vec, now);
        }

        let entry = MemoryEntry::new(final_vec.clone(), layer);
        match layer {
            MemoryLayer::Semantic => self.semantic_index.insert(key, final_vec.clone()),
            _ => self.episodic_index.insert(key, final_vec.clone()),
        }
        self.storage.insert(key, final_vec);
        self.metadata.insert(key.to_string(), entry);
        Ok(())
    }

    fn retrieve_with_metrics(&mut self, key: &str) -> Option<HyperVector> {
        if let Some(entry) = self.metadata.get_mut(key) {
            entry.update_access(); // Reinforcement Pressure: boosts importance
            return Some(entry.vector.clone());
        }
        None
    }

    fn consolidate_layers(&mut self) {
        let mut changes = Vec::new();
        for (key, entry) in &self.metadata {
            if let Some(new_layer) = LifecycleManager::evaluate_promotion(entry) {
                changes.push((key.clone(), new_layer));
            }
        }

        for (key, layer) in changes {
            if let Some(entry) = self.metadata.get_mut(&key) {
                let old_layer = entry.layer.clone();
                entry.layer = match layer {
                    MemoryLayer::Working => "working",
                    MemoryLayer::Episodic => "episodic",
                    MemoryLayer::Semantic => "semantic",
                }.to_string();

                if old_layer != "semantic" && entry.layer == "semantic" {
                    self.episodic_index.remove(&key);
                    self.semantic_index.insert(&key, entry.vector.clone());
                    info!("Industrial Lifecycle: Fact Promoted to Semantic Stability: {}", key);
                }
            }
        }
    }
}
