pub mod traits;
pub mod decoder;
pub mod text_encoder;
pub mod tokenizer;
pub mod modality;

// Expose core traits and modality logic
pub use traits::{PerceptionModule, VFAInterface, ModalitySlot};
pub use modality::{ModalityRegistry, TextModality, VisionModality, AudioModality};

// Legacy Stubs (Deprecated)
pub mod vision_stub;
pub mod audio_stub;
