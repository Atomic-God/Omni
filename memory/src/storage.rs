use core_vsa::{HyperVector, DIMENSION};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs::{self, File};
use std::io::{Read, Write};
use log::{info, warn};
use bincode::{serialize, deserialize};
use serde::{Serialize, Deserialize};
use std::sync::{Arc, Mutex};

/// Storage Manager that handles Sharding and Persistence.
///
/// Strategy:
/// - Keep a "Hot" LSH Index in memory.
/// - Offload "Cold" vectors to Sharded Disk Files.
/// - On startup, load Indices (lightweight) or rebuild them.
///
/// For this phase, we implement basic Sharded Storage where:
/// - We write to `shard_N.bin` when `current_shard` is full.
/// - We keep a map `ID -> (ShardID, Offset)`?
///   - Offset is hard with compressed binaries.
///   - Simplification: `ID -> ShardID`. Load full shard into cache if needed.
pub struct ShardedStorage {
    pub root_dir: PathBuf,
    pub current_shard_id: usize,
    pub shard_capacity: usize,
    pub current_shard_buffer: HashMap<String, HyperVector>,

    // Global Index of ID -> ShardID (0 means memory/current, >0 means disk)
    pub location_map: HashMap<String, usize>,
}

#[derive(Serialize, Deserialize)]
pub struct ShardFile {
    pub id: usize,
    pub data: HashMap<String, HyperVector>,
}

impl ShardedStorage {
    pub fn new(root_dir: &Path) -> Self {
        if !root_dir.exists() {
            fs::create_dir_all(root_dir).unwrap();
        }

        Self {
            root_dir: root_dir.to_path_buf(),
            current_shard_id: 1, // Start at 1
            shard_capacity: 1000, // Small for testing, typically 10k-100k
            current_shard_buffer: HashMap::new(),
            location_map: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: &str, vector: HyperVector) {
        self.current_shard_buffer.insert(key.to_string(), vector);
        self.location_map.insert(key.to_string(), 0); // 0 = Hot Buffer

        if self.current_shard_buffer.len() >= self.shard_capacity {
            self.flush_shard();
        }
    }

    pub fn retrieve(&mut self, key: &str) -> Option<HyperVector> {
        let loc = self.location_map.get(key)?;

        if *loc == 0 {
            return self.current_shard_buffer.get(key).cloned();
        }

        // Cold Retrieval: Load shard
        // In a real DB, we'd cache this. Here we just read, find, return.
        let shard_path = self.root_dir.join(format!("shard_{}.bin", loc));
        if let Ok(file) = File::open(&shard_path) {
            let shard: ShardFile = bincode::deserialize_from(file).ok()?;
            shard.data.get(key).cloned()
        } else {
            None
        }
    }

    fn flush_shard(&mut self) {
        let shard_id = self.current_shard_id;
        let shard_path = self.root_dir.join(format!("shard_{}.bin", shard_id));

        info!("Flushing memory to shard {}", shard_id);

        let shard_data = ShardFile {
            id: shard_id,
            data: self.current_shard_buffer.clone(),
        };

        let file = File::create(&shard_path).unwrap();
        bincode::serialize_into(file, &shard_data).unwrap();

        // Update locations
        for k in self.current_shard_buffer.keys() {
            self.location_map.insert(k.clone(), shard_id);
        }

        // Reset buffer
        self.current_shard_buffer.clear();
        self.current_shard_id += 1;
    }
}
