use crate::Tensor;
use crate::layers::Layer;

pub trait Optimizer: Send + Sync {
    fn step(&mut self, layer: &mut dyn Layer);
}

pub struct SGD {
    pub learning_rate: f32,
    pub momentum: f32,
    pub velocities: Vec<Tensor>,
}

impl SGD {
    pub fn new(lr: f32, momentum: f32) -> Self {
        Self {
            learning_rate: lr,
            momentum,
            velocities: Vec::new(),
        }
    }
}

impl Optimizer for SGD {
    fn step(&mut self, layer: &mut dyn Layer) {
        let (mut params, grads) = layer.params_and_grads();

        if self.velocities.is_empty() {
             for p in &params {
                 self.velocities.push(Tensor::zeros(p.shape.clone()));
             }
        }

        for i in 0..params.len() {
            let param = &mut params[i];
            let grad = &grads[i];

            let update = &**grad * self.learning_rate;
            let mut new_data = Vec::with_capacity(param.data.len());
            for (p_val, u_val) in param.data.iter().zip(update.data.iter()) {
                new_data.push(p_val - u_val);
            }

            param.data = new_data;
        }
    }
}

pub struct Adam {
    pub learning_rate: f32,
    pub beta1: f32,
    pub beta2: f32,
    pub epsilon: f32,
    pub m: Vec<Tensor>,
    pub v: Vec<Tensor>,
    pub t: usize,
}

impl Adam {
    pub fn new(lr: f32) -> Self {
        Self {
            learning_rate: lr,
            beta1: 0.9,
            beta2: 0.999,
            epsilon: 1e-8,
            m: Vec::new(),
            v: Vec::new(),
            t: 0,
        }
    }
}

impl Optimizer for Adam {
    fn step(&mut self, layer: &mut dyn Layer) {
        let (mut params, grads) = layer.params_and_grads();

        self.t += 1;

        if self.m.is_empty() {
            for p in &params {
                self.m.push(Tensor::zeros(p.shape.clone()));
                self.v.push(Tensor::zeros(p.shape.clone()));
            }
        }

        for i in 0..params.len() {
            let param = &mut params[i];
            let grad = &grads[i];

            for j in 0..param.data.len() {
                let g = grad.data[j];

                self.m[i].data[j] = self.beta1 * self.m[i].data[j] + (1.0 - self.beta1) * g;
                self.v[i].data[j] = self.beta2 * self.v[i].data[j] + (1.0 - self.beta2) * g * g;

                let m_hat = self.m[i].data[j] / (1.0 - self.beta1.powi(self.t as i32));
                let v_hat = self.v[i].data[j] / (1.0 - self.beta2.powi(self.t as i32));

                param.data[j] -= self.learning_rate * m_hat / (v_hat.sqrt() + self.epsilon);
            }
        }
    }
}
