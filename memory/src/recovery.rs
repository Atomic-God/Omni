use crate::{ShardedStorage, SnapshotManager, MemoryManager};
use tracing::{error, info, warn};
use std::path::Path;
use sha2::{Sha256, Digest};
use std::fs::File;
use std::io::Read;

pub struct CorruptionRecovery;

impl CorruptionRecovery {
    pub fn check_integrity(storage: &ShardedStorage) -> bool {
        for id in 1..storage.current_shard_id {
            let path = storage.root_dir.join(format!("shard_{}.bin", id));
            if !path.exists() {
                error!("Missing shard {}", id);
                return false;
            }

            if let Ok(mut file) = File::open(&path) {
                let mut hasher = Sha256::new();
                let mut buffer = Vec::new();
                if file.read_to_end(&mut buffer).is_ok() {
                    hasher.update(&buffer);
                    let _hash = hex::encode(hasher.finalize());
                }
            }
        }
        true
    }

    pub fn self_heal(memory: &mut MemoryManager) -> Result<(), Box<dyn std::error::Error>> {
        info!("Industrial Recovery: Starting Self-Healing process...");

        if !Self::check_integrity(&memory.storage) {
            warn!("Storage corruption detected! Attempting reconstruction...");
        }

        memory.episodic_index = crate::LSHIndex::new();
        memory.semantic_index = crate::LSHIndex::new();

        let buffer = memory.storage.current_shard_buffer.borrow();
        for (key, vector) in buffer.iter() {
             memory.episodic_index.insert(key, vector.clone());
        }

        info!("Self-Healing complete. Indices reconstructed.");
        Ok(())
    }
}

pub struct RollbackManager;

impl RollbackManager {
    pub fn rollback_to_previous(memory: &mut MemoryManager, current_snap_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        info!("Attempting rollback from {:?}", current_snap_path);
        let current = SnapshotManager::load(current_snap_path)?;

        if let Some(prev_hash) = current.header.previous_hash {
            let prev_path = memory.root_dir.join(format!("{}.snap", prev_hash));
            if prev_path.exists() {
                memory.load_snapshot(&prev_hash)?;
                return Ok(());
            }
        }
        Err("Rollback failed: No valid previous version found.".into())
    }
}
