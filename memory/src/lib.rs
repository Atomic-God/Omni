#![deny(warnings)]
use core_vsa::HyperVector;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;
use tracing::{info, debug};

pub mod lsh;
pub mod storage;
pub mod snapshot;
pub mod recovery;
pub mod hierarchy;
pub mod tests;
pub mod layered;
pub mod evolution;
pub mod consolidation; // New

pub use lsh::LSHIndex;
pub use storage::ShardedStorage;
pub use snapshot::{SnapshotManager, MindSnapshot, SnapshotHeader, MindPack};
pub use recovery::CorruptionRecovery;
pub use hierarchy::{MemoryLayer, MemoryEntry, HierarchicalMemory};
pub use layered::LayeredMemory;
use crate::evolution::{LifecycleManager, EpisodicEncoder, ForgettingEngine};
pub use consolidation::PrototypeConsolidator;

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
    pub vsa_precision: usize,
}

impl MemoryManager {
    pub fn new(root_dir: &Path) -> Self {
        Self::with_dimension(root_dir, core_vsa::DIMENSION)
    }

    pub fn with_dimension(root_dir: &Path, dim: usize) -> Self {
        fs::create_dir_all(root_dir).unwrap();
        Self {
            episodic_index: LSHIndex::with_dimension(dim),
            semantic_index: LSHIndex::with_dimension(dim),
            storage: ShardedStorage::new(root_dir),
            root_dir: root_dir.to_path_buf(),
            metadata: HashMap::new(),
            low_memory_mode: false,
            encoder: EpisodicEncoder::new(),
            vsa_precision: dim,
        }
    }

    pub fn set_low_memory_mode(&mut self, enabled: bool) {
        self.low_memory_mode = enabled;
        if enabled {
            info!("Industrial Adaptation: Reducing VSA precision for low-RAM environment.");
            self.vsa_precision = 2048;
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

        if snapshot.header.is_delta {
            SnapshotManager::apply_delta(self, &snapshot);
        } else {
            self.storage = snapshot.storage;
            self.episodic_index = snapshot.episodic_index;
            self.semantic_index = snapshot.semantic_index;
            self.metadata = snapshot.metadata;
        }
        Ok(())
    }

    pub fn save_delta(&self, name: &str, base_name: &str) -> Result<(), Box<dyn Error>> {
        let base_path = self.root_dir.join(format!("{}.snap", base_name));
        let base_snapshot = SnapshotManager::load(&base_path)?;
        let delta_path = self.root_dir.join(format!("{}.snap", name));
        SnapshotManager::save_delta(self, &delta_path, &base_snapshot)
    }

    pub fn export_mindpack(&self, path: &Path, encryption_key: Option<&str>) -> Result<(), Box<dyn Error>> {
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

        pack.export(path, &pack, encryption_key)?;
        Ok(())
    }

    pub fn incremental_restore(&mut self, snapshots: Vec<MindSnapshot>) -> Result<(), Box<dyn Error>> {
        info!("Industrial Restore: Applying {} snapshot layers.", snapshots.len());
        for snap in snapshots {
            if snap.header.is_delta {
                SnapshotManager::apply_delta(self, &snap);
            } else {
                self.storage = snap.storage;
                self.episodic_index = snap.episodic_index;
                self.semantic_index = snap.semantic_index;
                self.metadata = snap.metadata;
            }
        }
        Ok(())
    }

    /// Triggers a "Sleep Cycle" for deep memory consolidation and pruning.
    pub fn sleep_cycle(&mut self) {
        info!("Industrial Memory: Starting Sleep Cycle (Consolidation & Forgetting)...");

        // 1. Prototype generation and promotion
        self.consolidate_layers();

        // 2. Apply forgetting curve
        ForgettingEngine::prune_fading_memories(&mut self.metadata, 0.95);

        // 3. Synchronize indices
        let keys: Vec<String> = self.metadata.keys().cloned().collect();
        self.episodic_index.sync_with_keys(&keys);
        self.semantic_index.sync_with_keys(&keys);

        info!("Industrial Memory: Sleep Cycle complete. Active entries: {}", self.metadata.len());
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
        results
    }
}

impl HierarchicalMemory for MemoryManager {
    fn store_in_layer(&mut self, key: &str, vector: HyperVector, layer: MemoryLayer) -> Result<(), Box<dyn Error>> {
        let mut final_vec = vector;

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
            entry.update_access();
            return Some(entry.vector.clone());
        }
        None
    }

    fn consolidate_layers(&mut self) {
        // 1. Prototype Generation (Industrial Compression)
        let prototypes = PrototypeConsolidator::generate_prototypes(&self.metadata, 0.90);
        for (key, proto_vec) in prototypes {
             if let Some(entry) = self.metadata.get_mut(&key) {
                 entry.vector = proto_vec;
                 entry.layer = "semantic".to_string();
                 entry.confidence = 0.95;
                 self.semantic_index.insert(&key, entry.vector.clone());
                 self.episodic_index.remove(&key);
             }
        }

        // 2. Lifecycle Promotion
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
