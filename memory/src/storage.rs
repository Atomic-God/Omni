use core_vsa::HyperVector;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::fs::{self, File};
use tracing::info;
use serde::{Serialize, Deserialize};
use std::cell::RefCell;

#[derive(Serialize, Deserialize)]
pub struct ShardedStorage {
    #[serde(skip)]
    pub root_dir: PathBuf,
    pub current_shard_id: usize,
    pub shard_capacity: usize,
    // RefCell for interior mutability during 'retrieve' (caching)
    #[serde(skip, default = "default_buffer")]
    pub current_shard_buffer: RefCell<BTreeMap<String, HyperVector>>,

    pub location_map: BTreeMap<String, usize>,
    pub shard_hashes: BTreeMap<usize, String>,
}

fn default_buffer() -> RefCell<BTreeMap<String, HyperVector>> {
    RefCell::new(BTreeMap::new())
}

#[derive(Serialize, Deserialize)]
pub struct ShardFile {
    pub id: usize,
    pub data: BTreeMap<String, HyperVector>,
}

impl Clone for ShardedStorage {
    fn clone(&self) -> Self {
        Self {
            root_dir: self.root_dir.clone(),
            current_shard_id: self.current_shard_id,
            shard_capacity: self.shard_capacity,
            current_shard_buffer: RefCell::new(self.current_shard_buffer.borrow().clone()),
            location_map: self.location_map.clone(),
            shard_hashes: self.shard_hashes.clone(),
        }
    }
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
            current_shard_buffer: RefCell::new(BTreeMap::new()),
            location_map: BTreeMap::new(),
            shard_hashes: BTreeMap::new(),
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
        use sha2::{Sha256, Digest};
        let shard_id = self.current_shard_id;
        let shard_path = self.root_dir.join(format!("shard_{}.bin", shard_id));

        info!("Flushing memory to shard {}", shard_id);

        let buffer = self.current_shard_buffer.borrow();
        let shard_data = ShardFile {
            id: shard_id,
            data: buffer.clone(),
        };

        let bytes = bincode::serialize(&shard_data).unwrap();
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let hash = hex::encode(hasher.finalize());

        fs::write(&shard_path, &bytes).unwrap();
        self.shard_hashes.insert(shard_id, hash);

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
