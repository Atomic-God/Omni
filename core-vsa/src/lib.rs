#![deny(warnings)]
//! # OmniForge Core VSA Engine
//!
//! This crate implements Vector Symbolic Architectures (VSA), specifically using
//! Binary Spatter Codes (BSC). HyperVectors are high-dimensional (typically 10,000 bits)
//! representations that allow for symbolic reasoning through algebraic operations.
//!
//! ## Key Operations:
//! - **Binding**: XOR-based binding. Combines two vectors into a new one,
//!   orthogonal to both. Reversible via unbinding (applying XOR again).
//! - **Bundling**: Bitwise majority rule (implemented with probabilistic merging).
//!   Merges vectors into a set-like representation that remains similar to its components.
//! - **Permutation**: Cyclic shift. Used for sequence encoding and mapping structural roles.
use once_cell::sync::Lazy;
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::{Hasher, Hash};

pub mod traits;

pub const DEFAULT_DIMENSION: usize = 10_000;
pub const DIMENSION: usize = DEFAULT_DIMENSION;

pub static ROLE_SUBJECT: Lazy<HyperVector> = Lazy::new(|| HyperVector::deterministic(0x5001));
pub static ROLE_VERB: Lazy<HyperVector> = Lazy::new(|| HyperVector::deterministic(0x5002));
pub static ROLE_OBJECT: Lazy<HyperVector> = Lazy::new(|| HyperVector::deterministic(0x5003));

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HyperVector {
    pub words: Vec<u64>,
    pub dim: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_noise_resilience() {
        let v1 = HyperVector::random();
        let v2 = v1.add_noise(0.05); // 5% noise
        let sim = v1.similarity(&v2);
        println!("Similarity with 5% noise: {}", sim);
        assert!(sim > 0.8); // 5% noise should keep similarity high

        let v3 = v1.add_noise(0.25); // 25% noise
        let sim2 = v1.similarity(&v3);
        println!("Similarity with 25% noise: {}", sim2);
        assert!(sim2 > 0.0); // Should still be positively correlated
        assert!(sim2 < sim);
    }

    #[test]
    fn test_binding_unbinding() {
        let a = HyperVector::random();
        let b = HyperVector::random();
        let c = a.bind(&b);
        let a_approx = c.bind(&b); // c XOR b = a
        assert!(a.similarity(&a_approx) > 0.99);
    }
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
    pub fn random() -> Self {
        Self::random_dim(DEFAULT_DIMENSION)
    }

    pub fn random_dim(dim: usize) -> Self {
        let num_words = (dim + 63) / 64;
        let mut rng = rand::thread_rng();
        let mut words: Vec<u64> = (0..num_words).map(|_| rng.gen()).collect();

        // Mask last word
        if dim % 64 != 0 {
            if let Some(last) = words.last_mut() {
                *last &= (1 << (dim % 64)) - 1;
            }
        }

        Self { words, dim }
    }

    pub fn deterministic(seed: u64) -> Self {
        Self::deterministic_dim(seed, DEFAULT_DIMENSION)
    }

    pub fn deterministic_dim(seed: u64, dim: usize) -> Self {
        let num_words = (dim + 63) / 64;
        let mut rng = StdRng::seed_from_u64(seed);
        let mut words: Vec<u64> = (0..num_words).map(|_| rng.gen()).collect();

        // Mask last word
        if dim % 64 != 0 {
            if let Some(last) = words.last_mut() {
                *last &= (1 << (dim % 64)) - 1;
            }
        }

        Self { words, dim }
    }

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

    pub fn inverse(&self) -> Self {
        self.clone()
    }

    pub fn bundle(&self, other: &Self) -> Self {
        assert_eq!(self.dim, other.dim, "Dimension mismatch in bundle");
        let mut h = seahash::SeaHasher::new();
        for w in &self.words { h.write_u64(*w); }
        for w in &other.words { h.write_u64(*w); }
        let seed = h.finish();
        let mut rng = StdRng::seed_from_u64(seed);

        let words = self
            .words
            .iter()
            .zip(other.words.iter())
            .map(|(&a, &b)| {
                let random_bits: u64 = rng.gen();
                let diff = a ^ b;
                (a & !diff) | (random_bits & diff)
            })
            .collect();
        Self { words, dim: self.dim }
    }

    pub fn permute(&self, shifts: usize) -> Self {
        let shift = shifts % self.dim;

        if shift == 0 {
            return self.clone();
        }

        let mut bits = Vec::with_capacity(self.dim);
        for (i, word) in self.words.iter().enumerate() {
            for j in 0..64 {
                if i * 64 + j < self.dim {
                    bits.push((word >> j) & 1);
                }
            }
        }

        bits.rotate_left(shift);

        let mut new_words = vec![0u64; self.words.len()];
        for (i, bit) in bits.iter().enumerate() {
            if *bit == 1 {
                new_words[i / 64] |= 1 << (i % 64);
            }
        }

        Self { words: new_words, dim: self.dim }
    }

    /// Computes the cosine-like similarity between two HyperVectors using Hamming distance.
    /// Result is in range [-1, 1], where 1 is identical, 0 is orthogonal (random), and -1 is inverse.
    pub fn similarity(&self, other: &Self) -> f32 {
        assert_eq!(self.dim, other.dim, "Dimension mismatch in similarity");
        let mut hamming: u32 = 0;

        for (a, b) in self.words.iter().zip(other.words.iter()) {
            hamming += (a ^ b).count_ones();
        }

        // Adjust for bits beyond self.dim in the last word
        let total_bits = self.dim as f32;
        1.0 - 2.0 * (hamming as f32 / total_bits)
    }

    /// Adds noise to the HyperVector by flipping a percentage of bits.
    pub fn add_noise(&self, noise_level: f32) -> Self {
        let mut rng = rand::thread_rng();
        let words = self.words.iter().map(|&w| {
            let mut word = w;
            for i in 0..64 {
                if rng.gen::<f32>() < noise_level {
                    word ^= 1 << i;
                }
            }
            word
        }).collect();
        Self { words, dim: self.dim }
    }

    pub fn truncate(&self, new_dim: usize) -> Self {
        assert!(new_dim <= self.dim);
        let num_words = (new_dim + 63) / 64;
        let mut words = self.words[..num_words].to_vec();

        if new_dim % 64 != 0 {
            let mask = (1 << (new_dim % 64)) - 1;
            if let Some(last) = words.last_mut() {
                *last &= mask;
            }
        }

        Self { words, dim: new_dim }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct FactTriple {
    pub subject: String,
    pub predicate: String,
    pub object: String,
}

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
    pub confidence: f32,
    pub source_reliability: f32,
    pub reinforcement_count: u32,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolEdge {
    pub source: String,
    pub target: String,
    pub relation: String,
    pub weight: f32,
    pub confidence: f32,
    pub source_reliability: f32,
    pub reinforcement_count: u32,
    pub timestamp: u64,
}

impl SymbolGraph {
    pub fn new() -> Self {
        Self { nodes: Vec::new(), edges: Vec::new() }
    }

    pub fn add_node(&mut self, id: &str, vector: HyperVector, metadata: HashMap<String, String>) {
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
        self.nodes.push(SymbolNode {
            id: id.to_string(),
            vector,
            metadata,
            confidence: 1.0,
            source_reliability: 1.0,
            reinforcement_count: 1,
            timestamp: now,
        });
    }

    pub fn add_node_with_confidence(&mut self, id: &str, vector: HyperVector, metadata: HashMap<String, String>, confidence: f32) {
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
        self.nodes.push(SymbolNode {
            id: id.to_string(),
            vector,
            metadata,
            confidence,
            source_reliability: 1.0,
            reinforcement_count: 1,
            timestamp: now,
        });
    }

    pub fn add_edge(&mut self, source: &str, target: &str, relation: &str, weight: f32) {
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
        self.edges.push(SymbolEdge {
            source: source.to_string(),
            target: target.to_string(),
            relation: relation.to_string(),
            weight,
            confidence: 1.0,
            source_reliability: 1.0,
            reinforcement_count: 1,
            timestamp: now,
        });
    }

    pub fn add_edge_with_confidence(&mut self, source: &str, target: &str, relation: &str, weight: f32, confidence: f32) {
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
        self.edges.push(SymbolEdge {
            source: source.to_string(),
            target: target.to_string(),
            relation: relation.to_string(),
            weight,
            confidence,
            source_reliability: 1.0,
            reinforcement_count: 1,
            timestamp: now,
        });
    }
}
