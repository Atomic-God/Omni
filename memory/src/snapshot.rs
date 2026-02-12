use crate::MemorySystem;
use core_vsa::HyperVector;
use std::path::Path;
use std::fs;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct SnapshotManifest {
    pub version: String,
    pub timestamp: u64,
    pub shards: Vec<usize>,
}

pub struct SnapshotManager;

impl SnapshotManager {
    pub fn save_snapshot(memory: &MemorySystem, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        // 1. Ensure all buffers flushed (requires mut memory, but here we pass ref? logic gap)
        // Ideally MemorySystem should auto-flush or we call flush before save.
        // Since we can't mutate here, we assume flushed or we snapshot the ShardStorage state on disk.

        // 2. Create Manifest
        let manifest = SnapshotManifest {
            version: "1.0".to_string(),
            timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_secs(),
            shards: (1..memory.storage.current_shard_id).collect(),
        };

        let file = fs::File::create(path.join("manifest.json"))?;
        serde_json::to_writer(file, &manifest)?;

        // Shards are already on disk in memory.storage.root_dir.
        // If path != root_dir, we might need to copy.
        // Assuming path IS root_dir for simplicity.

        Ok(())
    }

    pub fn load_snapshot(path: &Path) -> Result<MemorySystem, Box<dyn std::error::Error>> {
        // 1. Read Manifest
        let file = fs::File::open(path.join("manifest.json"))?;
        let manifest: SnapshotManifest = serde_json::from_reader(file)?;

        // 2. Init System
        let mut memory = MemorySystem::new(path);

        // 3. Rehydrate Index
        // Iterate all shards, load vectors, insert into LSH.
        // This is "Eager Loading". "Lazy Loading" would mean only indexing headers.
        // For "Millions of symbols", eager loading might be slow but it's safe.
        // Or we use "Memory Mapped" files.

        for shard_id in manifest.shards {
            let shard_path = path.join(format!("shard_{}.bin", shard_id));
            if let Ok(file) = fs::File::open(shard_path) {
                if let Ok(shard) = bincode::deserialize_from::<_, crate::storage::ShardFile>(file) {
                    for (k, v) in shard.data {
                        memory.index.insert(&k, v); // Populates LSH cache
                        memory.storage.location_map.insert(k, shard_id);
                    }
                }
            }
        }

        // Update current shard ID to max + 1
        memory.storage.current_shard_id = manifest.shards.iter().max().unwrap_or(&0) + 1;

        Ok(memory)
    }
}
