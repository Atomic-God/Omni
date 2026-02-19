use std::ops::{Add, Mul, Sub};
use rayon::prelude::*;
use rand::prelude::*;
use std::fmt;

#[derive(Clone, Debug)]
pub struct Tensor {
    pub data: Vec<f32>,
    pub shape: Vec<usize>,
}

impl Tensor {
    pub fn new(data: Vec<f32>, shape: Vec<usize>) -> Self {
        let size: usize = shape.iter().product();
        assert_eq!(data.len(), size, "Data length must match shape product");
        Self { data, shape }
    }

    pub fn zeros(shape: Vec<usize>) -> Self {
        let size: usize = shape.iter().product();
        Self {
            data: vec![0.0; size],
            shape,
        }
    }

    pub fn rand(shape: Vec<usize>) -> Self {
        let size: usize = shape.iter().product();
        let mut rng = rand::thread_rng();
        let data: Vec<f32> = (0..size).map(|_| rng.gen_range(-0.1..0.1)).collect();
        Self { data, shape }
    }

    pub fn shape(&self) -> &[usize] {
        &self.shape
    }

    pub fn matmul(&self, other: &Self) -> Self {
        // Assume 2D for simplicity in Phase 1
        assert_eq!(self.shape.len(), 2, "Matmul requires 2D tensors");
        assert_eq!(other.shape.len(), 2, "Matmul requires 2D tensors");

        let m = self.shape[0];
        let k = self.shape[1];
        let n = other.shape[1];

        assert_eq!(k, other.shape[0], "Dimension mismatch for matmul: {:?} x {:?}", self.shape, other.shape);

        let mut result_data = vec![0.0; m * n];

        // Parallel matrix multiplication
        result_data.par_chunks_mut(n).enumerate().for_each(|(i, row)| {
            for j in 0..n {
                let mut sum = 0.0;
                for p in 0..k {
                    sum += self.data[i * k + p] * other.data[p * n + j];
                }
                row[j] = sum;
            }
        });

        Self::new(result_data, vec![m, n])
    }

    pub fn transpose(&self) -> Self {
        assert_eq!(self.shape.len(), 2);
        let rows = self.shape[0];
        let cols = self.shape[1];
        let mut new_data = vec![0.0; rows * cols];

        for i in 0..rows {
            for j in 0..cols {
                new_data[j * rows + i] = self.data[i * cols + j];
            }
        }

        Self::new(new_data, vec![cols, rows])
    }
}

// Basic Ops
impl Add for &Tensor {
    type Output = Tensor;
    fn add(self, rhs: Self) -> Tensor {
        assert_eq!(self.shape, rhs.shape, "Shape mismatch for add");
        let data: Vec<f32> = self.data.par_iter().zip(&rhs.data).map(|(a, b)| a + b).collect();
        Tensor::new(data, self.shape.clone())
    }
}

impl Mul<f32> for &Tensor {
    type Output = Tensor;
    fn mul(self, scalar: f32) -> Tensor {
        let data: Vec<f32> = self.data.par_iter().map(|a| a * scalar).collect();
        Tensor::new(data, self.shape.clone())
    }
}

// For testing
impl PartialEq for Tensor {
    fn eq(&self, other: &Self) -> bool {
        if self.shape != other.shape { return false; }
        self.data.iter().zip(&other.data).all(|(a, b)| (a - b).abs() < 1e-5)
    }
}
