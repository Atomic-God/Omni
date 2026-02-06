use once_cell::sync::Lazy;
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use std::hash::Hasher;
use std::collections::hash_map::DefaultHasher;

pub mod index;

// 10,000 bits. 10000 / 64 = 156.25 -> 157 u64s.
const DIMENSION: usize = 10_000;
const NUM_WORDS: usize = (DIMENSION + 63) / 64;

/// Static hypervector representing the Subject role in an SVO structure.
/// Deterministically initialized.
pub static ROLE_SUBJECT: Lazy<HyperVector> = Lazy::new(|| HyperVector::deterministic(0x5001));
/// Static hypervector representing the Verb role in an SVO structure.
pub static ROLE_VERB: Lazy<HyperVector> = Lazy::new(|| HyperVector::deterministic(0x5002));
/// Static hypervector representing the Object role in an SVO structure.
pub static ROLE_OBJECT: Lazy<HyperVector> = Lazy::new(|| HyperVector::deterministic(0x5003));

/// A high-dimensional vector supporting VSA operations.
/// Implements a packed bit model (BSC) with dimension 10,000.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HyperVector {
    pub words: Vec<u64>,
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
                // Equal mask: !(a ^ b)
                // Result = (a & equal_mask) | (random & !equal_mask)
                let random_bits: u64 = rng.gen();
                let diff = a ^ b;
                (a & !diff) | (random_bits & diff)
            })
            .collect();
        Self { words }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random_properties() {
        let hv = HyperVector::random();
        assert_eq!(hv.words.len(), NUM_WORDS);
    }

    #[test]
    fn test_deterministic_roles() {
        let r1 = HyperVector::deterministic(123);
        let r2 = HyperVector::deterministic(123);
        assert_eq!(r1, r2);

        let r3 = HyperVector::deterministic(124);
        assert_ne!(r1, r3);
    }

    #[test]
    fn test_deterministic_bundle() {
        let a = HyperVector::deterministic(1);
        let b = HyperVector::deterministic(2);
        let c1 = a.bundle(&b);
        let c2 = a.bundle(&b);
        assert_eq!(c1, c2);
    }

    #[test]
    fn test_orthogonality() {
        let hv1 = HyperVector::random();
        let hv2 = HyperVector::random();
        let sim = hv1.similarity(&hv2);
        assert!(sim.abs() < 0.05, "Similarity {} should be near 0", sim);
    }

    #[test]
    fn test_bind_inverse() {
        let a = HyperVector::random();
        let b = HyperVector::random();
        let bound = a.bind(&b);
        let recovered = bound.bind(&b);
        assert!(a.similarity(&recovered) > 0.99);
    }

    #[test]
    fn test_bundle_similarity() {
        let a = HyperVector::random();
        let b = HyperVector::random();
        let bundle = a.bundle(&b);

        let sim_a = bundle.similarity(&a);
        let sim_b = bundle.similarity(&b);

        assert!(sim_a > 0.45 && sim_a < 0.55, "Sim A: {}", sim_a);
        assert!(sim_b > 0.45 && sim_b < 0.55, "Sim B: {}", sim_b);
    }
}
