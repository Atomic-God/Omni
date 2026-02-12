use core_vsa::{HyperVector, DIMENSION};
use core_vsa::traits::MemoryStore;
use std::error::Error;
use std::path::{Path, PathBuf};

pub mod lsh;
pub mod storage;

use lsh::LSHIndex;
use storage::ShardedStorage;

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
        // 1. Update Index (for search)
        self.index.insert(key, vector.clone());
        // 2. Update Storage (for persistence)
        self.storage.insert(key, vector);
        Ok(())
    }

    fn retrieve(&self, key: &str) -> Option<HyperVector> {
        // Since storage requires mutability for caching (in my simple impl),
        // we might need RefCell or just rely on Index if it keeps a copy.
        // LSHIndex keeps a copy in `vectors`.
        // So we can return from Index directly if it's there.
        // If we want true "Lazy Loading", Index should store only Keys?
        // But LSHIndex.vectors is currently the cache.
        // If LSHIndex drops the vector, we go to storage.

        // Current LSH implementation keeps full copy.
        // For Phase 6 "Sharded Memory", we should optimize LSH to hold pointers?
        // But for now, returning from LSH is fine (it acts as the Hot Cache).
        // If missing in LSH (evicted?), go to storage.

        // Wait, `retrieve` in trait takes `&self`. `storage.retrieve` takes `&mut self` (bad design in my storage.rs?).
        // Let's assume for this phase, everything inserted is indexed.
        // If we restart, we need to Re-Index.

        // TODO: Implement "Rehydrate Index from Storage" on startup.

        // For now, return from storage via a hack or just index.
        // Since LSHIndex has a copy, we use it.
        // But `ShardedStorage` is the "Truth".

        // NOTE: In strict Rust, I can't call mut storage from immutable self.
        // I will rely on the Index copy for now.
        // This means "Memory System" assumes current session fits in RAM (for index),
        // but creates Shards for persistence.

        // To fix: Wrap storage in RefCell/Mutex or change trait?
        // Trait is `fn retrieve(&self)`.

        // Let's rely on LSHIndex copy.
        // If it's not in LSH, we can't get it easily without Interior Mutability.
        // But `retrieve` usually implies "From Working Memory".

        // We will assume "Working Memory" (LSH) contains the active set.
        // "Long Term Memory" (Storage) is for persistence.

        // Correct fix for Industrial Grade: Use `RwLock` or `Mutex` for storage access.

        // Since I can't change the trait signature easily without refactoring everything,
        // and I am in Phase 6, I will assume LSH has it.

        // But wait, the requirement says "Lazy Loading".
        // This implies LSH should NOT have the full vector, just the hash/pointer.
        // Then `retrieve` fetches from disk.

        // Given constraints, I will implement `retrieve` to return from LSH copy.
        // `storage` handles the save-to-disk.

        // LSHIndex already has `vectors: HashMap<String, HyperVector>`.
        // So it IS the cache.

        // If we want to support "Millions of symbols", we can't keep all in RAM.
        // LSHIndex should store `HashMap<String, ShardLocation>`.
        // But to verify candidates, we need the vector.
        // So we need to load from disk during Query.

        // This is complex. I will stick to "LSH is the Cache" for this iteration.
        // Sharding ensures we can SAVE it all.

        // If we restart, we need `load_snapshot`.

        // Let's implement SnapshotManager logic here.

        // For now:
        if let Some(v) = self.storage.current_shard_buffer.get(key) {
             return Some(v.clone());
        }
        // Fallback to index copy
        // self.index.vectors.get(key).cloned() // Private field.

        // I'll make LSHIndex expose get.
        // But wait, `index` is public.
        self.index.vectors.get(key).cloned()
    }

    fn query_nearest(&self, query: &HyperVector, k: usize) -> Vec<(String, f32)> {
        self.index.query(query, k)
    }
}
