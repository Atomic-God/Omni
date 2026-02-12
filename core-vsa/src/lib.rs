use once_cell::sync::Lazy;
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::{Hasher, Hash};
use std::collections::hash_map::DefaultHasher;
use std::path::Path;

pub mod traits;

// 10,000 bits. 10000 / 64 = 156.25 -> 157 u64s.
pub const DIMENSION: usize = 10_000;
pub const NUM_WORDS: usize = (DIMENSION + 63) / 64;

/// Static hypervector representing the Subject role in an SVO structure.
pub static ROLE_SUBJECT: Lazy<HyperVector> = Lazy::new(|| HyperVector::deterministic(0x5001));
/// Static hypervector representing the Verb role in an SVO structure.
pub static ROLE_VERB: Lazy<HyperVector> = Lazy::new(|| HyperVector::deterministic(0x5002));
/// Static hypervector representing the Object role in an SVO structure.
pub static ROLE_OBJECT: Lazy<HyperVector> = Lazy::new(|| HyperVector::deterministic(0x5003));

/// A high-dimensional vector supporting VSA operations.
/// Implements a packed bit model (BSC) with dimension 10,000.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HyperVector {
    pub words: Vec<u64>,
}

impl Hash for HyperVector {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for word in &self.words {
            word.hash(state);
        }
    }
}

impl HyperVector {
    /// Generates a random hypervector (non-deterministic).
    pub fn random() -> Self {
        let mut rng = rand::thread_rng();
        let words: Vec<u64> = (0..NUM_WORDS).map(|_| rng.gen()).collect();
        Self { words }
    }

    /// Generates a deterministic hypervector from a seed.
    pub fn deterministic(seed: u64) -> Self {
        let mut rng = StdRng::seed_from_u64(seed);
        let words: Vec<u64> = (0..NUM_WORDS).map(|_| rng.gen()).collect();
        Self { words }
    }

    /// Binding operation (XOR).
    pub fn bind(&self, other: &Self) -> Self {
        let words = self
            .words
            .iter()
            .zip(other.words.iter())
            .map(|(a, b)| a ^ b)
            .collect();
        Self { words }
    }

    /// Bundling operation (Majority Rule) with Deterministic Tie-Breaking.
    pub fn bundle(&self, other: &Self) -> Self {
        // Hash the inputs to seed the tie-breaker
        let mut hasher = DefaultHasher::new();
        for w in &self.words { hasher.write_u64(*w); }
        for w in &other.words { hasher.write_u64(*w); }
        let seed = hasher.finish();
        let mut rng = StdRng::seed_from_u64(seed);

        let words = self
            .words
            .iter()
            .zip(other.words.iter())
            .map(|(&a, &b)| {
                let random_bits: u64 = rng.gen();
                let diff = a ^ b;
                // If diff bit is 1, take random. If 0, take a (or b, same).
                (a & !diff) | (random_bits & diff)
            })
            .collect();
        Self { words }
    }

    /// Permutation (Cyclic Shift).
    /// Shifts bits left by `shifts`.
    pub fn permute(&self, shifts: usize) -> Self {
        let mut new_words = vec![0u64; NUM_WORDS];
        let total_bits = NUM_WORDS * 64;
        let shift = shifts % total_bits;

        if shift == 0 {
            return self.clone();
        }

        // This is a slow bit-by-bit implementation for correctness.
        // Optimized SIMD/block shift should be used in Phase 8.
        // Copy to a bit vector
        let mut bits = Vec::with_capacity(total_bits);
        for word in &self.words {
            for i in 0..64 {
                bits.push((word >> i) & 1);
            }
        }

        // Rotate
        bits.rotate_left(shift);

        // Reconstruct
        for (i, bit) in bits.iter().enumerate() {
            if *bit == 1 {
                new_words[i / 64] |= 1 << (i % 64);
            }
        }

        Self { words: new_words }
    }

    /// Cosine similarity approximation via Hamming distance.
    /// Sim = 1 - 2 * (Hamming / Dim).
    /// Range: 1.0 (identical) to -1.0 (inverse). 0.0 (orthogonal).
    pub fn similarity(&self, other: &Self) -> f32 {
        let hamming: u32 = self
            .words
            .iter()
            .zip(other.words.iter())
            .map(|(a, b)| (a ^ b).count_ones())
            .sum();

        let total_bits = (NUM_WORDS * 64) as f32;
        1.0 - 2.0 * (hamming as f32 / total_bits)
    }
}

/// Represents a structured graph of symbols (Concepts).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolGraph {
    pub nodes: Vec<SymbolNode>,
    pub edges: Vec<SymbolEdge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolNode {
    pub id: String,
    pub vector: HyperVector,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolEdge {
    pub source: String,
    pub target: String,
    pub relation: String,
    pub weight: f32,
}

impl SymbolGraph {
    pub fn new() -> Self {
        Self { nodes: Vec::new(), edges: Vec::new() }
    }

    pub fn add_node(&mut self, id: &str, vector: HyperVector, metadata: HashMap<String, String>) {
        self.nodes.push(SymbolNode {
            id: id.to_string(),
            vector,
            metadata,
        });
    }

    pub fn add_edge(&mut self, source: &str, target: &str, relation: &str, weight: f32) {
        self.edges.push(SymbolEdge {
            source: source.to_string(),
            target: target.to_string(),
            relation: relation.to_string(),
            weight,
        });
    }
}
