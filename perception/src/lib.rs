pub mod traits;
pub mod decoder;
pub mod text_encoder;
pub mod tokenizer;
pub mod modality;
pub mod script;

// Expose core traits and modality logic
pub use traits::{PerceptionModule, VFAInterface, ModalitySlot};
pub use modality::{ModalityRegistry, TextModality, VisionModality, AudioModality};
pub use script::{ScriptNormalizer, UnknownLanguageHandler};

// Legacy Stubs (Deprecated)
pub mod vision_stub;
pub mod audio_stub;
