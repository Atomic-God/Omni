use rand::prelude::*;
use serde::{Serialize, Deserialize};
use once_cell::sync::Lazy;

// 10,000 bits. 10000 / 64 = 156.25 -> 157 u64s.
const DIMENSION: usize = 10_000;
const NUM_WORDS: usize = (DIMENSION + 63) / 64;

/// Static hypervector representing the Subject role in an SVO structure.
pub static ROLE_SUBJECT: Lazy<HyperVector> = Lazy::new(|| HyperVector::random());
/// Static hypervector representing the Verb role in an SVO structure.
pub static ROLE_VERB: Lazy<HyperVector> = Lazy::new(|| HyperVector::random());
/// Static hypervector representing the Object role in an SVO structure.
pub static ROLE_OBJECT: Lazy<HyperVector> = Lazy::new(|| HyperVector::random());

/// A high-dimensional vector supporting VSA operations.
/// Implements a packed bit model (BSC) with dimension 10,000.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HyperVector {
    pub words: Vec<u64>,
}

impl HyperVector {
    /// Generates a random hypervector.
    pub fn random() -> Self {
        let mut rng = rand::thread_rng();
        let words: Vec<u64> = (0..NUM_WORDS).map(|_| rng.gen()).collect();
        Self { words }
    }

    /// Binding operation (XOR).
    pub fn bind(&self, other: &Self) -> Self {
        let words = self.words.iter()
            .zip(other.words.iter())
            .map(|(a, b)| a ^ b)
            .collect();
        Self { words }
    }

    /// Bundling operation (Majority Rule).
    /// For 2 vectors A, B:
    /// If bits agree (00 or 11), result is that bit.
    /// If bits disagree (01 or 10), result is random.
    /// Logic: (A & B) | (A & R) | (B & R) ??
    /// Simpler: majority(A, B, Random)
    pub fn bundle(&self, other: &Self) -> Self {
        let mut rng = rand::thread_rng();
        let words = self.words.iter()
            .zip(other.words.iter())
            .map(|(&a, &b)| {
                // If bits equal, keep them. If different, random.
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
        let hamming: u32 = self.words.iter()
            .zip(other.words.iter())
            .map(|(a, b)| (a ^ b).count_ones())
            .sum();

        // Adjust for potential padding bits in last word if DIMENSION % 64 != 0
        // But for random vectors, padding shouldn't skew much if consistent.
        // We treat it as full 157*64 = 10048 dim effectively, or mask last word.
        // Let's assume standard behavior for now.

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

        // Expected similarity ~ 0.5 for A+B (random tie break)
        let sim_a = bundle.similarity(&a);
        let sim_b = bundle.similarity(&b);

        assert!(sim_a > 0.45 && sim_a < 0.55, "Sim A: {}", sim_a);
        assert!(sim_b > 0.45 && sim_b < 0.55, "Sim B: {}", sim_b);
    }
}
