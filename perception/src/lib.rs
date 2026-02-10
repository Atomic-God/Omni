pub mod traits;
pub mod decoder;
pub mod text_encoder;
pub mod tokenizer;
pub mod modality;
pub mod script;
pub mod clustering;
pub mod gpu_interfaces;
pub mod language; // Added
pub mod fabrication; // Added
pub mod polyglot;

// Expose core traits and modality logic
pub use traits::{PerceptionModule, VFAInterface, ModalitySlot};
pub use modality::{ModalityRegistry, TextModality, VisionModality, AudioModality};
pub use script::{ScriptNormalizer, UnknownLanguageHandler};
pub use clustering::SymbolClustering;
pub use gpu_interfaces::{InrInterface, ResonatorInterface, AudioVsaInterface, TensorBackend};
pub use language::{LanguageAbstractionLayer, UniversalLanguage};
pub use fabrication::{VisionFabricator, AudioFabricator, INRTrainer, ResonatorGPU, FabricationManifest};

// Legacy Stubs (Deprecated)
pub mod vision_stub;
pub mod audio_stub;
