pub mod neural; // Stubs
pub mod mapper; // New
pub mod text_encoder; // Existing
pub mod decoder; // Existing

pub use neural::{NeuralEncoder, VisionEncoder, AudioEncoder, SyntaxEncoder, Tensor};
pub use mapper::NeuralMapper;
