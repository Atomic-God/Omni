use crate::Tensor;
use rand::prelude::*;

pub trait Layer: Send + Sync {
    fn forward(&self, input: &Tensor) -> Tensor;
    fn backward(&mut self, grad_output: &Tensor, input: &Tensor) -> Tensor;
    fn parameters(&mut self) -> Vec<&mut Tensor>;
    fn gradients(&mut self) -> Vec<&mut Tensor>;
    fn params_and_grads(&mut self) -> (Vec<&mut Tensor>, Vec<&mut Tensor>);
}

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

        self.grad_weights = &self.grad_weights + &dw;

        let batch_size = grad_output.shape[0];
        let output_dim = grad_output.shape[1];
        let mut db_data = vec![0.0; output_dim];

        for i in 0..batch_size {
            for j in 0..output_dim {
                db_data[j] += grad_output.data[i * output_dim + j];
            }
        }
        let db = Tensor::new(db_data, vec![1, output_dim]);
        self.grad_bias = &self.grad_bias + &db;

        let weights_t = self.weights.transpose();
        grad_output.matmul(&weights_t)
    }

    fn parameters(&mut self) -> Vec<&mut Tensor> {
        vec![&mut self.weights, &mut self.bias]
    }

    fn gradients(&mut self) -> Vec<&mut Tensor> {
        vec![&mut self.grad_weights, &mut self.grad_bias]
    }

    fn params_and_grads(&mut self) -> (Vec<&mut Tensor>, Vec<&mut Tensor>) {
        (
            vec![&mut self.weights, &mut self.bias],
            vec![&mut self.grad_weights, &mut self.grad_bias]
        )
    }
}

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
    fn params_and_grads(&mut self) -> (Vec<&mut Tensor>, Vec<&mut Tensor>) { (vec![], vec![]) }
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
    fn params_and_grads(&mut self) -> (Vec<&mut Tensor>, Vec<&mut Tensor>) { (vec![], vec![]) }
}

pub struct RecurrentStateLayer {
    pub w_xh: Tensor,
    pub w_hh: Tensor,
    pub bias: Tensor,
    pub grad_w_xh: Tensor,
    pub grad_w_hh: Tensor,
    pub grad_bias: Tensor,
    pub hidden_size: usize,
    pub stored_hidden: Option<Tensor>,
}

impl RecurrentStateLayer {
    pub fn new(input_dim: usize, hidden_size: usize) -> Self {
        let w_xh = Tensor::rand(vec![input_dim, hidden_size]);
        let w_hh = Tensor::rand(vec![hidden_size, hidden_size]);
        let bias = Tensor::zeros(vec![1, hidden_size]);

        Self {
            w_xh,
            w_hh,
            bias,
            grad_w_xh: Tensor::zeros(vec![input_dim, hidden_size]),
            grad_w_hh: Tensor::zeros(vec![hidden_size, hidden_size]),
            grad_bias: Tensor::zeros(vec![1, hidden_size]),
            hidden_size,
            stored_hidden: None,
        }
    }

    pub fn reset_state(&mut self) {
        self.stored_hidden = None;
    }
}

impl Layer for RecurrentStateLayer {
    fn forward(&self, input: &Tensor) -> Tensor {
        let batch_size = input.shape[0];

        let h_prev = if let Some(ref h) = self.stored_hidden {
            h.clone()
        } else {
            Tensor::zeros(vec![batch_size, self.hidden_size])
        };

        let wx = input.matmul(&self.w_xh);
        let wh = h_prev.matmul(&self.w_hh);
        let sum = &wx + &wh;

        let mut data = sum.data.clone();
        for i in 0..batch_size {
            for j in 0..self.hidden_size {
                data[i * self.hidden_size + j] += self.bias.data[j];
            }
        }

        let activated: Vec<f32> = data.iter().map(|x| x.tanh()).collect();
        Tensor::new(activated, vec![batch_size, self.hidden_size])
    }

    fn backward(&mut self, _grad_output: &Tensor, input: &Tensor) -> Tensor {
        Tensor::zeros(input.shape.clone())
    }

    fn parameters(&mut self) -> Vec<&mut Tensor> {
        vec![&mut self.w_xh, &mut self.w_hh, &mut self.bias]
    }

    fn gradients(&mut self) -> Vec<&mut Tensor> {
        vec![&mut self.grad_w_xh, &mut self.grad_w_hh, &mut self.grad_bias]
    }

    fn params_and_grads(&mut self) -> (Vec<&mut Tensor>, Vec<&mut Tensor>) {
        (
            vec![&mut self.w_xh, &mut self.w_hh, &mut self.bias],
            vec![&mut self.grad_w_xh, &mut self.grad_w_hh, &mut self.grad_bias]
        )
    }
}
