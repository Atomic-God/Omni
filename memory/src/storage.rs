use core_vsa::HyperVector;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs::{self, File};
use log::info;
use serde::{Serialize, Deserialize};
use std::cell::RefCell;

#[derive(Serialize, Deserialize, Clone)]
pub struct ShardedStorage {
    pub root_dir: PathBuf,
    pub current_shard_id: usize,
    pub shard_capacity: usize,
    // RefCell for interior mutability during 'retrieve' (caching)
    // But Cache should be transient?
    // Serializing RefCell is tricky if we want to save cache state.
    // Usually we don't save cache.
    // We mark it skip?
    #[serde(skip, default = "default_buffer")]
    pub current_shard_buffer: RefCell<HashMap<String, HyperVector>>,

    pub location_map: HashMap<String, usize>,
}

fn default_buffer() -> RefCell<HashMap<String, HyperVector>> {
    RefCell::new(HashMap::new())
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
            current_shard_id: 1,
            shard_capacity: 1000,
            current_shard_buffer: RefCell::new(HashMap::new()),
            location_map: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: &str, vector: HyperVector) {
        self.current_shard_buffer.borrow_mut().insert(key.to_string(), vector);
        self.location_map.insert(key.to_string(), 0);

        if self.current_shard_buffer.borrow().len() >= self.shard_capacity {
            self.flush_shard();
        }
    }

    pub fn retrieve(&self, key: &str) -> Option<HyperVector> {
        let loc = self.location_map.get(key)?;

        if *loc == 0 {
            return self.current_shard_buffer.borrow().get(key).cloned();
        }

        // Cold Retrieval: Load shard
        // TODO: Cache loaded shard?
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

        let buffer = self.current_shard_buffer.borrow();
        let shard_data = ShardFile {
            id: shard_id,
            data: buffer.clone(),
        };

        let file = File::create(&shard_path).unwrap();
        bincode::serialize_into(file, &shard_data).unwrap();

        // Update locations
        for k in buffer.keys() {
            self.location_map.insert(k.clone(), shard_id);
        }

        // Reset buffer
        drop(buffer); // Release borrow
        self.current_shard_buffer.borrow_mut().clear();
        self.current_shard_id += 1;
    }
}
