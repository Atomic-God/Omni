use crate::{ShardedStorage, SnapshotManager, MemoryManager};
use log::{error, info, warn};
use std::path::Path;

pub struct CorruptionRecovery;

impl CorruptionRecovery {
    pub fn check_integrity(storage: &ShardedStorage) -> bool {
        for id in 1..storage.current_shard_id {
            let path = storage.root_dir.join(format!("shard_{}.bin", id));
            if !path.exists() {
                error!("Missing shard {}", id);
                return false;
            }
            if let Ok(meta) = std::fs::metadata(&path) {
                if meta.len() == 0 {
                    error!("Empty shard {}", id);
                    return false;
                }
            }
        }
        true
    }

    pub fn recover(_storage: &mut ShardedStorage) {
        info!("Starting recovery...");
    }
}

pub struct RollbackManager;

impl RollbackManager {
    pub fn rollback_to_previous(memory: &mut MemoryManager, current_snap_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        info!("Attempting rollback from {:?}", current_snap_path);
        let current = SnapshotManager::load(current_snap_path)?;

        if let Some(prev_hash) = current.header.previous_hash {
            info!("Previous version found with hash: {}", prev_hash);
            // Search for snapshot with this hash in root_dir
            // For now, we assume standard naming {hash}.snap
            let prev_path = memory.root_dir.join(format!("{}.snap", prev_hash));
            if prev_path.exists() {
                memory.load_snapshot(&prev_hash)?;
                info!("Rollback successful.");
                return Ok(());
            } else {
                warn!("Previous snapshot file not found at {:?}", prev_path);
            }
        } else {
            warn!("No previous version hash in current snapshot header.");
        }

        Err("Rollback failed: No valid previous version found.".into())
    }
}
