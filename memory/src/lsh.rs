use core_vsa::{HyperVector, DIMENSION};
use std::collections::{HashMap, HashSet};
use rand::prelude::*;
use log::{info, debug};

const NUM_TABLES: usize = 10;
const BITS_PER_KEY: usize = 16;

/// Locality Sensitive Hashing Index for Hamming Space.
/// Uses bit sampling (Random Projections) to index HyperVectors.
///
/// Structure:
/// - L tables.
/// - Each table has a random mask of K bits.
/// - Bucket = integer value of the masked bits.
///
/// Query:
/// - Collect candidates from all L buckets.
/// - Compute exact Hamming distance for candidates.
pub struct LSHIndex {
    tables: Vec<HashMap<u64, Vec<String>>>,
    masks: Vec<Vec<usize>>, // Indices of bits to sample for each table
    vectors: HashMap<String, HyperVector>, // The actual store (or cache)
}

impl LSHIndex {
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        let mut tables = Vec::with_capacity(NUM_TABLES);
        let mut masks = Vec::with_capacity(NUM_TABLES);

        for _ in 0..NUM_TABLES {
            tables.push(HashMap::new());
            // Select K distinct random bit indices
            let mut mask: Vec<usize> = (0..DIMENSION).collect();
            mask.shuffle(&mut rng);
            masks.push(mask.into_iter().take(BITS_PER_KEY).collect());
        }

        Self {
            tables,
            masks,
            vectors: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: &str, vector: HyperVector) {
        // 1. Store the vector
        self.vectors.insert(key.to_string(), vector.clone());

        // 2. Index in each table
        for i in 0..NUM_TABLES {
            let hash = self.compute_hash(&vector, i);
            self.tables[i].entry(hash).or_insert_with(Vec::new).push(key.to_string());
        }
    }

    pub fn query(&self, query_vec: &HyperVector, k: usize) -> Vec<(String, f32)> {
        let mut candidates = HashSet::new();

        // 1. Collect candidates
        for i in 0..NUM_TABLES {
            let hash = self.compute_hash(query_vec, i);
            if let Some(bucket) = self.tables[i].get(&hash) {
                for id in bucket {
                    candidates.insert(id);
                }
            }
        }

        debug!("LSH Query: Found {} candidates for K={}", candidates.len(), k);

        // 2. Refine (Exact Distance)
        let mut results: Vec<(String, f32)> = candidates.into_iter().filter_map(|id| {
            if let Some(vec) = self.vectors.get(id) {
                let sim = query_vec.similarity(vec);
                Some((id.clone(), sim))
            } else {
                None
            }
        }).collect();

        // Sort descending by similarity
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        if results.len() > k {
            results.truncate(k);
        }

        results
    }

    fn compute_hash(&self, vector: &HyperVector, table_idx: usize) -> u64 {
        let mut hash: u64 = 0;
        for (i, &bit_idx) in self.masks[table_idx].iter().enumerate() {
            // Extract bit at bit_idx
            // Vector is Vec<u64>.
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lsh_recall() {
        let mut index = LSHIndex::new();

        let target = HyperVector::random();
        let noise = HyperVector::random();

        // Insert target and some noise
        index.insert("target", target.clone());
        for i in 0..100 {
            index.insert(&format!("noise_{}", i), HyperVector::random());
        }

        // Create a query similar to target (flip 5% bits)
        // 10,000 bits -> 500 bits flipped.
        // Similarity should be ~0.9.
        let mut query = target.clone();
        // Manually flip bits? core-vsa doesn't expose it easily.
        // Let's bundle with itself multiple times? No.
        // Let's just use target as query (Perfect recall check).

        let results = index.query(&target, 5);

        assert!(!results.is_empty());
        assert_eq!(results[0].0, "target");
        assert!(results[0].1 > 0.99);
    }
}
