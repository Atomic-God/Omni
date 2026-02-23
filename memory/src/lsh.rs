use core_vsa::{HyperVector, DIMENSION};
use std::collections::{HashSet, BTreeMap};
use rand::seq::SliceRandom;
use tracing::debug;
use serde::{Serialize, Deserialize};

const NUM_TABLES: usize = 10;
const BITS_PER_KEY: usize = 16;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LSHIndex {
    tables: Vec<BTreeMap<u64, Vec<String>>>,
    masks: Vec<Vec<usize>>,
    vectors: BTreeMap<String, HyperVector>,
}

impl LSHIndex {
    pub fn new() -> Self {
        Self::with_dimension(DIMENSION)
    }

    pub fn with_dimension(dim: usize) -> Self {
        let mut rng = rand::thread_rng();
        let mut tables = Vec::with_capacity(NUM_TABLES);
        let mut masks = Vec::with_capacity(NUM_TABLES);

        for _ in 0..NUM_TABLES {
            tables.push(BTreeMap::new());
            let mut mask: Vec<usize> = (0..dim).collect();
            mask.shuffle(&mut rng);
            masks.push(mask.into_iter().take(BITS_PER_KEY).collect());
        }

        Self {
            tables,
            masks,
            vectors: BTreeMap::new(),
        }
    }

    pub fn insert(&mut self, key: &str, vector: HyperVector) {
        if self.vectors.contains_key(key) {
            self.remove(key);
        }
        self.vectors.insert(key.to_string(), vector.clone());

        for i in 0..NUM_TABLES {
            let hash = self.compute_hash(&vector, i);
            self.tables[i].entry(hash).or_insert_with(Vec::new).push(key.to_string());
        }
    }

    pub fn remove(&mut self, key: &str) {
        if let Some(vector) = self.vectors.remove(key) {
            for i in 0..NUM_TABLES {
                let hash = self.compute_hash(&vector, i);
                if let Some(bucket) = self.tables[i].get_mut(&hash) {
                    bucket.retain(|id| id != key);
                }
            }
        }
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.vectors.contains_key(key)
    }

    pub fn query(&self, query_vec: &HyperVector, k: usize) -> Vec<(String, f32)> {
        let mut candidates = HashSet::new();

        for i in 0..NUM_TABLES {
            let hash = self.compute_hash(query_vec, i);
            if let Some(bucket) = self.tables[i].get(&hash) {
                for id in bucket {
                    candidates.insert(id);
                }
            }
        }

        debug!("LSH Query: Found {} candidates for K={}", candidates.len(), k);

        let mut results: Vec<(String, f32)> = candidates.into_iter().filter_map(|id| {
            if let Some(vec) = self.vectors.get(id) {
                let sim = query_vec.similarity(vec);
                Some((id.clone(), sim))
            } else {
                None
            }
        }).collect();

        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        if results.len() > k {
            results.truncate(k);
        }

        results
    }

    fn compute_hash(&self, vector: &HyperVector, table_idx: usize) -> u64 {
        let mut hash: u64 = 0;
        for (i, &bit_idx) in self.masks[table_idx].iter().enumerate() {
            let word_idx = bit_idx / 64;
            let bit_offset = bit_idx % 64;
            let bit = (vector.words[word_idx] >> bit_offset) & 1;

            if bit == 1 {
                hash |= 1 << i;
            }
        }
        hash
    }

    pub fn len(&self) -> usize {
        self.vectors.len()
    }

    /// Synchronizes the index with a provided list of valid keys, removing any orphans.
    pub fn sync_with_keys(&mut self, valid_keys: &[String]) {
        let valid_set: HashSet<_> = valid_keys.iter().collect();
        let current_keys: Vec<_> = self.vectors.keys().cloned().collect();
        for key in current_keys {
            if !valid_set.contains(&key) {
                self.remove(&key);
            }
        }
    }

    /// Performs industrial optimization: removes empty buckets and shrinks map capacity.
    pub fn optimize(&mut self) {
        debug!("LSH Optimization: Pruning empty buckets.");
        for table in self.tables.iter_mut() {
            table.retain(|_, bucket| !bucket.is_empty());
        }
    }

    /// Returns a subset of the index containing only the specified keys.
    /// Used for delta snapshot optimization.
    pub fn get_subset(&self, keys: &HashSet<String>) -> Self {
        let mut subset = Self {
            tables: vec![BTreeMap::new(); NUM_TABLES],
            masks: self.masks.clone(),
            vectors: BTreeMap::new(),
        };

        for key in keys {
            if let Some(vec) = self.vectors.get(key) {
                subset.insert(key, vec.clone());
            }
        }
        subset
    }

    /// Merges another index into this one.
    pub fn merge(&mut self, other: Self) {
        for (key, vec) in other.vectors {
            self.insert(&key, vec);
        }
    }
}
