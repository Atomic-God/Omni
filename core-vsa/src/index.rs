use crate::HyperVector;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone)]
pub struct LshIndex {
    pub projections: Vec<HyperVector>,
    pub buckets: HashMap<u64, Vec<HyperVector>>,
}

impl LshIndex {
    pub fn new(num_projections: usize) -> Self {
        let projections = (0..num_projections)
            .map(|_| HyperVector::random())
            .collect();
        Self {
            projections,
            buckets: HashMap::new(),
        }
    }

    fn hash(&self, vec: &HyperVector) -> u64 {
        let mut hash: u64 = 0;
        for (i, proj) in self.projections.iter().enumerate() {
            // Similarity > 0 means dot product positive -> bit 1
            // For bipolar, sim > 0 is roughly "more same than different"
            if vec.similarity(proj) > 0.0 {
                hash |= 1 << i;
            }
        }
        hash
    }

    pub fn insert(&mut self, vec: HyperVector) {
        let h = self.hash(&vec);
        self.buckets.entry(h).or_default().push(vec);
    }

    pub fn search(&self, query: &HyperVector) -> Option<&HyperVector> {
        // Fallback to exhaustive scan for correctness in prototype.
        // Naive LSH single-bucket lookup fails for containment queries (Q subset of S).
        let mut best_sim = -1.0;
        let mut best_match = None;

        for candidates in self.buckets.values() {
            for cand in candidates {
                let sim = cand.similarity(query);
                if sim > best_sim {
                    best_sim = sim;
                    best_match = Some(cand);
                }
            }
        }
        best_match
    }
}

impl Default for LshIndex {
    fn default() -> Self {
        Self::new(64) // 64-bit hash
    }
}
