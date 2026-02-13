pub mod tensor;
pub mod autograd;
pub mod layers;
pub mod optimizer;
pub mod loss;
pub mod generator; // Placeholder for Phase 2

pub use tensor::Tensor;
pub use layers::{Layer, DenseLayer, RecurrentStateLayer, ReLU, Tanh};
pub use optimizer::{Optimizer, SGD, Adam};
pub use loss::CrossEntropyLoss;
