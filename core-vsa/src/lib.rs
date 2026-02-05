use rand::prelude::*;
use serde::{Serialize, Deserialize};

const DIMENSION: usize = 10_000;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HyperVector {
    values: Vec<i8>, // Stores ±1
}

impl HyperVector {
    /// Generates a random hypervector with ±1 values.
    pub fn random() -> Self {
        let mut rng = rand::thread_rng();
        let values: Vec<i8> = (0..DIMENSION)
            .map(|_| if rng.gen() { 1 } else { -1 })
            .collect();
        Self { values }
    }

    /// Binding operation (element-wise multiplication for bipolar vectors).
    /// Equivalent to XOR in binary space.
    pub fn bind(&self, other: &Self) -> Self {
        assert_eq!(self.values.len(), other.values.len(), "Dimension mismatch");
        let values: Vec<i8> = self.values.iter()
            .zip(other.values.iter())
            .map(|(a, b)| a * b)
            .collect();
        Self { values }
    }

    /// Bundling operation (element-wise addition with majority rule normalization).
    /// Breaks ties randomly to maintain ±1.
    pub fn bundle(&self, other: &Self) -> Self {
        assert_eq!(self.values.len(), other.values.len(), "Dimension mismatch");
        let mut rng = rand::thread_rng();
        let values: Vec<i8> = self.values.iter()
            .zip(other.values.iter())
            .map(|(&a, &b)| {
                let sum = a as i16 + b as i16;
                if sum > 0 {
                    1
                } else if sum < 0 {
                    -1
                } else {
                    if rng.gen() { 1 } else { -1 }
                }
            })
            .collect();
        Self { values }
    }

    /// Cosine similarity for bipolar vectors.
    /// Since magnitude is constant (sqrt(N)), this is just dot product / N.
    pub fn similarity(&self, other: &Self) -> f32 {
        assert_eq!(self.values.len(), other.values.len(), "Dimension mismatch");
        let dot_product: i32 = self.values.iter()
            .zip(other.values.iter())
            .map(|(&a, &b)| (a as i32) * (b as i32))
            .sum();

        dot_product as f32 / DIMENSION as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random_properties() {
        let hv = HyperVector::random();
        assert_eq!(hv.values.len(), DIMENSION);
        assert!(hv.values.iter().all(|&x| x == 1 || x == -1));
    }

    #[test]
    fn test_orthogonality() {
        let hv1 = HyperVector::random();
        let hv2 = HyperVector::random();
        let sim = hv1.similarity(&hv2);
        // Random vectors in high dim should be nearly orthogonal (sim ~ 0)
        assert!(sim.abs() < 0.05, "Similarity {} should be near 0", sim);
    }

    #[test]
    fn test_bind_inverse() {
        let a = HyperVector::random();
        let b = HyperVector::random();
        let bound = a.bind(&b);
        let recovered = bound.bind(&b); // (A * B) * B = A * (B * B) = A * 1 = A
        assert!(a.similarity(&recovered) > 0.99);
    }

    #[test]
    fn test_bundle_similarity() {
        let a = HyperVector::random();
        let b = HyperVector::random();
        let bundle = a.bundle(&b);

        // Bundle should be similar to both components.
        // For discrete bipolar vectors with random tie-breaking:
        // Match probability p = 0.75.
        // Similarity = 2p - 1 = 0.5.
        let sim_a = bundle.similarity(&a);
        let sim_b = bundle.similarity(&b);

        assert!(sim_a > 0.45 && sim_a < 0.55, "Sim A: {}", sim_a);
        assert!(sim_b > 0.45 && sim_b < 0.55, "Sim B: {}", sim_b);
    }
}
