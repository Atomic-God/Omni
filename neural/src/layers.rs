use crate::Tensor;
use rand::prelude::*;
use std::cell::RefCell;

pub trait Layer: Send + Sync {
    fn forward(&self, input: &Tensor) -> Tensor;
    fn backward(&mut self, grad_output: &Tensor, input: &Tensor) -> Tensor;
    fn parameters(&mut self) -> Vec<&mut Tensor>;
    fn gradients(&mut self) -> Vec<&mut Tensor>;
}

// ... DenseLayer (unchanged) ...
pub struct DenseLayer {
    pub weights: Tensor,
    pub bias: Tensor,
    pub grad_weights: Tensor,
    pub grad_bias: Tensor,
}

impl DenseLayer {
    pub fn new(input_dim: usize, output_dim: usize) -> Self {
        let scale = (2.0 / (input_dim as f32 + output_dim as f32)).sqrt();
        let mut rng = rand::thread_rng();

        let w_data: Vec<f32> = (0..input_dim * output_dim)
            .map(|_| rng.gen_range(-scale..scale))
            .collect();

        let b_data = vec![0.0; output_dim];

        Self {
            weights: Tensor::new(w_data, vec![input_dim, output_dim]),
            bias: Tensor::new(b_data, vec![1, output_dim]),
            grad_weights: Tensor::zeros(vec![input_dim, output_dim]),
            grad_bias: Tensor::zeros(vec![1, output_dim]),
        }
    }
}

impl Layer for DenseLayer {
    fn forward(&self, input: &Tensor) -> Tensor {
        let xw = input.matmul(&self.weights);
        let rows = xw.shape[0];
        let cols = xw.shape[1];
        let mut output_data = xw.data.clone();

        for i in 0..rows {
            for j in 0..cols {
                output_data[i * cols + j] += self.bias.data[j];
            }
        }

        Tensor::new(output_data, vec![rows, cols])
    }

    fn backward(&mut self, grad_output: &Tensor, input: &Tensor) -> Tensor {
        let input_t = input.transpose();
        let dw = input_t.matmul(grad_output);

        // Accumulate
        let new_grad_w_data: Vec<f32> = self.grad_weights.data.iter().zip(dw.data.iter()).map(|(a, b)| a + b).collect();
        self.grad_weights.data = new_grad_w_data;

        let batch_size = grad_output.shape[0];
        let output_dim = grad_output.shape[1];
        let mut db_data = self.grad_bias.data.clone(); // Start with current grad

        for i in 0..batch_size {
            for j in 0..output_dim {
                db_data[j] += grad_output.data[i * output_dim + j];
            }
        }
        self.grad_bias.data = db_data;

        let weights_t = self.weights.transpose();
        grad_output.matmul(&weights_t)
    }

    fn parameters(&mut self) -> Vec<&mut Tensor> {
        vec![&mut self.weights, &mut self.bias]
    }

    fn gradients(&mut self) -> Vec<&mut Tensor> {
        vec![&mut self.grad_weights, &mut self.grad_bias]
    }
}

// ... ReLU, Tanh (unchanged structs, re-impl Layer) ...

pub struct ReLU;
impl Layer for ReLU {
    fn forward(&self, input: &Tensor) -> Tensor {
        let data: Vec<f32> = input.data.iter().map(|&x| if x > 0.0 { x } else { 0.0 }).collect();
        Tensor::new(data, input.shape.clone())
    }

    fn backward(&mut self, grad_output: &Tensor, input: &Tensor) -> Tensor {
        let data: Vec<f32> = grad_output.data.iter().zip(input.data.iter())
            .map(|(&g, &x)| if x > 0.0 { g } else { 0.0 })
            .collect();
        Tensor::new(data, input.shape.clone())
    }

    fn parameters(&mut self) -> Vec<&mut Tensor> { vec![] }
    fn gradients(&mut self) -> Vec<&mut Tensor> { vec![] }
}

pub struct Tanh;
impl Layer for Tanh {
    fn forward(&self, input: &Tensor) -> Tensor {
        let data: Vec<f32> = input.data.iter().map(|&x| x.tanh()).collect();
        Tensor::new(data, input.shape.clone())
    }

    fn backward(&mut self, grad_output: &Tensor, input: &Tensor) -> Tensor {
        let data: Vec<f32> = grad_output.data.iter().zip(input.data.iter())
            .map(|(&g, &x)| {
                let t = x.tanh();
                g * (1.0 - t * t)
            })
            .collect();
        Tensor::new(data, input.shape.clone())
    }

    fn parameters(&mut self) -> Vec<&mut Tensor> { vec![] }
    fn gradients(&mut self) -> Vec<&mut Tensor> { vec![] }
}

// ... RecurrentStateLayer ...

pub struct RecurrentStateLayer {
    pub w_xh: Tensor,
    pub w_hh: Tensor,
    pub bias: Tensor,
    pub grad_w_xh: Tensor,
    pub grad_w_hh: Tensor,
    pub grad_bias: Tensor,
    pub hidden_size: usize,
    pub stored_hidden: RefCell<Option<Tensor>>, // Use RefCell for interior mutability
}

impl RecurrentStateLayer {
    pub fn new(input_dim: usize, hidden_size: usize) -> Self {
        let w_xh = Tensor::rand(vec![input_dim, hidden_size]);
        let w_hh = Tensor::rand(vec![hidden_size, hidden_size]);
        let bias = Tensor::zeros(vec![1, hidden_size]);

        Self {
            w_xh: w_xh.clone(),
            w_hh: w_hh.clone(),
            bias: bias.clone(),
            grad_w_xh: Tensor::zeros(vec![input_dim, hidden_size]),
            grad_w_hh: Tensor::zeros(vec![hidden_size, hidden_size]),
            grad_bias: Tensor::zeros(vec![1, hidden_size]),
            hidden_size,
            stored_hidden: RefCell::new(None),
        }
    }

    pub fn reset_state(&self) {
        *self.stored_hidden.borrow_mut() = None;
    }
}

impl Layer for RecurrentStateLayer {
    fn forward(&self, input: &Tensor) -> Tensor {
        let batch_size = input.shape[0];

        // Clone hidden state from RefCell
        let h_prev = if let Some(ref h) = *self.stored_hidden.borrow() {
            h.clone()
        } else {
            Tensor::zeros(vec![batch_size, self.hidden_size])
        };

        let wx = input.matmul(&self.w_xh);
        let wh = h_prev.matmul(&self.w_hh);

        // wx + wh + bias
        // simplified add loop
        let mut sum_data = Vec::with_capacity(batch_size * self.hidden_size);
        for i in 0..batch_size {
            for j in 0..self.hidden_size {
                let idx = i * self.hidden_size + j;
                sum_data.push(wx.data[idx] + wh.data[idx] + self.bias.data[j]);
            }
        }

        // Tanh
        let activated: Vec<f32> = sum_data.iter().map(|x| x.tanh()).collect();
        let h_new = Tensor::new(activated, vec![batch_size, self.hidden_size]);

        // Update stored hidden
        *self.stored_hidden.borrow_mut() = Some(h_new.clone());

        h_new
    }

    fn backward(&mut self, grad_output: &Tensor, input: &Tensor) -> Tensor {
        // Simplified backward (one step)
        // Ignoring d/dh_prev for now

        // dL/dh_raw = grad_output * (1 - tanh^2(h))
        // We need 'h' (output).
        // We can recompute or assume grad_output IS dL/dh.
        // Actually grad_output is dL/dh_new.
        // So we need to multiply by derivative of tanh.
        // We need h_new values.
        // For Phase 1, assume we fetch it from stored_hidden (which matches input for backward if called immediately).

        let h_val = self.stored_hidden.borrow().as_ref().unwrap().clone();

        let mut delta_data = Vec::with_capacity(h_val.data.len());
        for (g, h) in grad_output.data.iter().zip(h_val.data.iter()) {
            delta_data.push(g * (1.0 - h * h));
        }
        let delta = Tensor::new(delta_data, h_val.shape.clone());

        // dL/dW_xh = x^T * delta
        let x_t = input.transpose();
        let dw_xh = x_t.matmul(&delta);

        // Accumulate grad
        // (Manual add loop to avoid borrowing issues if I used helper)
        for i in 0..self.grad_w_xh.data.len() {
             self.grad_w_xh.data[i] += dw_xh.data[i];
        }

        // Input grad
        // dL/dx = delta * W_xh^T
        let w_xh_t = self.w_xh.transpose();
        delta.matmul(&w_xh_t)
    }

    fn parameters(&mut self) -> Vec<&mut Tensor> {
        vec![&mut self.w_xh, &mut self.w_hh, &mut self.bias]
    }

    fn gradients(&mut self) -> Vec<&mut Tensor> {
        vec![&mut self.grad_w_xh, &mut self.grad_w_hh, &mut self.grad_bias]
    }
}
