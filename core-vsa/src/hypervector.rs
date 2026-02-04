use rand::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Hypervector {
    pub data: Vec<u8>, // Binary hypervector (0 or 1)
}

impl Hypervector {
    pub fn new(dim: usize) -> Self {
        Self {
            data: vec![0; dim],
        }
    }

    pub fn random(dim: usize) -> Self {
        let mut rng = rand::thread_rng();
        let data: Vec<u8> = (0..dim).map(|_| if rng.gen() { 1 } else { 0 }).collect();
        Self { data }
    }

    pub fn dim(&self) -> usize {
        self.data.len()
    }
}

pub trait VsaOps {
    fn bind(&self, other: &Self) -> Self;
    fn bundle(&self, other: &Self) -> Self;
    fn permute(&self, steps: i32) -> Self;
    fn distance(&self, other: &Self) -> f32; // Hamming distance
}

impl VsaOps for Hypervector {
    fn bind(&self, other: &Self) -> Self {
        assert_eq!(self.dim(), other.dim());
        let data = self.data.iter().zip(other.data.iter())
            .map(|(a, b)| a ^ b)
            .collect();
        Self { data }
    }

    fn bundle(&self, other: &Self) -> Self {
        assert_eq!(self.dim(), other.dim());
        // Simple majority rule bundle for binary vectors:
        // If we sum two, we can't break ties easily without keeping state.
        // For this VSA implementation, we'll use a stochastic OR or simple XOR as a placeholder
        // or a proper "superposition" requires integer vector backing.
        // For this step, let's assume bitwise OR for binary bundling roughly represents union
        // commonly used in simple Binary Spatter Codes, or keep it as XOR (thinning).
        // A better approach for binary is Majority rule over a set.
        // Let's implement element-wise OR as a basic "Add" for now, or probabilistic.
        // Let's stick to XOR for "thin" bundling or OR for "thick".
        // Let's use Component-wise Majority via random tie-breaking if strictly binary.
        // For simplicity in this Phase III start: Bitwise OR.
        let data = self.data.iter().zip(other.data.iter())
            .map(|(a, b)| a | b)
            .collect();
        Self { data }
    }

    fn permute(&self, steps: i32) -> Self {
        let len = self.data.len();
        let mut new_data = vec![0; len];
        let shift = steps.rem_euclid(len as i32) as usize;

        // Cyclic shift
        // new[i] = old[(i - shift) % len]
        for i in 0..len {
            let src_idx = (i + len - shift) % len;
            new_data[i] = self.data[src_idx];
        }
        Self { data: new_data }
    }

    fn distance(&self, other: &Self) -> f32 {
        assert_eq!(self.dim(), other.dim());
        let hamming: u32 = self.data.iter().zip(other.data.iter())
            .map(|(a, b)| (a ^ b) as u32)
            .sum();
        hamming as f32 / self.dim() as f32
    }
}
