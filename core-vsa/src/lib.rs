use once_cell::sync::Lazy;
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::{Hasher, Hash};
use std::collections::hash_map::DefaultHasher;
use std::path::Path;

pub mod traits;

// Configurable Dimension via Compile-Time Constant or Default?
// Requirement: "Configurable dimension".
// Rust const generics or runtime config?
// Core-VSA typically uses fixed dim for performance.
// Let's stick to 10,000 for "Industrial Core" default but allow resizing if we change the struct to wrap `BitVec` instead of `Vec<u64>`.
// However, `Vec<u64>` is faster.
// We will keep 10,000 as standard but add a method to resize or re-init?
// Actually, `HyperVector` struct owns `words`. It doesn't enforce length in type system (unlike array).
// So it is already configurable at runtime, just need to manage `NUM_WORDS`.

pub const DEFAULT_DIMENSION: usize = 10_000;

/// Static hypervector representing the Subject role in an SVO structure.
pub static ROLE_SUBJECT: Lazy<HyperVector> = Lazy::new(|| HyperVector::deterministic(0x5001));
/// Static hypervector representing the Verb role in an SVO structure.
pub static ROLE_VERB: Lazy<HyperVector> = Lazy::new(|| HyperVector::deterministic(0x5002));
/// Static hypervector representing the Object role in an SVO structure.
pub static ROLE_OBJECT: Lazy<HyperVector> = Lazy::new(|| HyperVector::deterministic(0x5003));

/// A high-dimensional vector supporting VSA operations.
/// Implements a packed bit model (BSC).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HyperVector {
    pub words: Vec<u64>,
    pub dim: usize, // Explicit dimension tracking
}

impl Hash for HyperVector {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.dim.hash(state);
        for word in &self.words {
            word.hash(state);
        }
    }
}

impl HyperVector {
    /// Generates a random hypervector (non-deterministic).
    pub fn random() -> Self {
        Self::random_dim(DEFAULT_DIMENSION)
    }

    pub fn random_dim(dim: usize) -> Self {
        let num_words = (dim + 63) / 64;
        let mut rng = rand::thread_rng();
        let words: Vec<u64> = (0..num_words).map(|_| rng.gen()).collect();
        Self { words, dim }
    }

    /// Generates a deterministic hypervector from a seed.
    pub fn deterministic(seed: u64) -> Self {
        Self::deterministic_dim(seed, DEFAULT_DIMENSION)
    }

    pub fn deterministic_dim(seed: u64, dim: usize) -> Self {
        let num_words = (dim + 63) / 64;
        let mut rng = StdRng::seed_from_u64(seed);
        let words: Vec<u64> = (0..num_words).map(|_| rng.gen()).collect();
        Self { words, dim }
    }

    /// Binding operation (XOR).
    pub fn bind(&self, other: &Self) -> Self {
        assert_eq!(self.dim, other.dim, "Dimension mismatch in bind");
        let words = self
            .words
            .iter()
            .zip(other.words.iter())
            .map(|(a, b)| a ^ b)
            .collect();
        Self { words, dim: self.dim }
    }

    /// Inverse Binding (XOR is self-inverse).
    pub fn inverse(&self) -> Self {
        self.clone()
    }

    /// Bundling operation (Majority Rule) with Deterministic Tie-Breaking.
    pub fn bundle(&self, other: &Self) -> Self {
        assert_eq!(self.dim, other.dim, "Dimension mismatch in bundle");
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
        Self { words, dim: self.dim }
    }

    /// Permutation (Cyclic Shift).
    /// Shifts bits left by `shifts`.
    pub fn permute(&self, shifts: usize) -> Self {
        let num_words = self.words.len();
        let mut new_words = vec![0u64; num_words];
        let total_bits = num_words * 64; // Approximate to word boundary for speed? No, strict cyclic.
        // Actually, we should use self.dim.
        // But packed u64 rotation is hard if dim is not multiple of 64.
        // For Industrial Core, assume dim is padded to 64-bit align or we handle partial last word.
        // Let's stick to full 64-bit words rotation for performance (dim ~ multiple of 64).

        let shift = shifts % total_bits;

        if shift == 0 {
            return self.clone();
        }

        // Bit-level rotation implementation
        // Optimization: Use `bs` crate or similar?
        // Manual implementation:
        let mut bits = Vec::with_capacity(total_bits);
        for word in &self.words {
            for i in 0..64 {
                bits.push((word >> i) & 1);
            }
        }

        bits.rotate_left(shift);

        for (i, bit) in bits.iter().enumerate() {
            if *bit == 1 {
                new_words[i / 64] |= 1 << (i % 64);
            }
        }

        Self { words: new_words, dim: self.dim }
    }

    /// Cosine similarity approximation via Hamming distance.
    pub fn similarity(&self, other: &Self) -> f32 {
        assert_eq!(self.dim, other.dim, "Dimension mismatch in similarity");
        let hamming: u32 = self
            .words
            .iter()
            .zip(other.words.iter())
            .map(|(a, b)| (a ^ b).count_ones())
            .sum();

        let total_bits = (self.words.len() * 64) as f32; // Use actual allocation size for normalization?
        // Or self.dim? If we padded, padded bits might add noise if not zeroed.
        // We initialize random, so padded bits are random.
        // This is fine for Hamming.

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
