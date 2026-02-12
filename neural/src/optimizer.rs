use crate::Tensor;
use crate::layers::Layer;

pub trait Optimizer: Send + Sync {
    fn step(&mut self, layer: &mut dyn Layer);
}

pub struct SGD {
    pub learning_rate: f32,
    pub momentum: f32,
    pub velocities: Vec<Tensor>, // Momentum buffer
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
        let params = layer.parameters();
        let grads = layer.gradients();

        // Ensure velocity buffer is initialized
        if self.velocities.is_empty() {
             for p in &params {
                 self.velocities.push(Tensor::zeros(p.shape.clone()));
             }
        }

        // Update
        for i in 0..params.len() {
            let param = &mut params[i];
            let grad = &grads[i];

            // v = m * v - lr * g
            // p = p + v

            // Simple SGD w/o momentum for Phase 1 to ensure correctness first
            // p = p - lr * g

            let update = &**grad * self.learning_rate;
            let mut new_data = Vec::with_capacity(param.data.len());
            for (p_val, u_val) in param.data.iter().zip(update.data.iter()) {
                new_data.push(p_val - u_val);
            }

            // Assign back
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
        let params = layer.parameters();
        let grads = layer.gradients();

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

            // m = b1*m + (1-b1)*g
            // v = b2*v + (1-b2)*g^2
            // m_hat = m / (1-b1^t)
            // v_hat = v / (1-b2^t)
            // p = p - lr * m_hat / (sqrt(v_hat) + eps)

            // Simplified CPU loop
            for j in 0..param.data.len() {
                let g = grad.data[j];

                // Update m
                self.m[i].data[j] = self.beta1 * self.m[i].data[j] + (1.0 - self.beta1) * g;
                // Update v
                self.v[i].data[j] = self.beta2 * self.v[i].data[j] + (1.0 - self.beta2) * g * g;

                let m_hat = self.m[i].data[j] / (1.0 - self.beta1.powi(self.t as i32));
                let v_hat = self.v[i].data[j] / (1.0 - self.beta2.powi(self.t as i32));

                param.data[j] -= self.learning_rate * m_hat / (v_hat.sqrt() + self.epsilon);
            }
        }
    }
}
