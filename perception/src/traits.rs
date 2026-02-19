use core_vsa::HyperVector;
use std::any::Any;

/// Legacy traits (aliased for compatibility or explicit text use)
pub trait Encoder {
    fn encode(&self, text: &str) -> HyperVector;
}

pub trait Decoder {
    fn decode(&self, vec: &HyperVector) -> String;
}

// New Multimodal Traits

/// Core trait for any module providing perceptual inputs to the Mind.
pub trait PerceptionModule: Send + Sync {
    /// Encodes raw input data (e.g., text, image bytes) into a VSA HyperVector.
    fn encode(&self, input: &[u8]) -> HyperVector;

    /// Decodes a HyperVector back into a human-readable or raw format.
    fn decode(&self, vec: &HyperVector) -> Vec<u8>;
}

/// Interface for Vector Function Architectures (VFA).
/// Allows mapping continuous values (time, space, audio) into high-dimensional space.
pub trait VFAInterface: Send + Sync {
    /// Maps a continuous scalar or coordinate to a HyperVector.
    fn map_coordinate(&self, value: f64) -> HyperVector;
}

/// Bridge for Resonator Networks to factorize composite vectors.
pub trait ResonatorBridge: Send + Sync {
    /// Factorizes a composite vector into its constituent factors (e.g., Position + Object).
    fn factorize(&self, composite: &HyperVector) -> Vec<HyperVector>;
}

/// A generic slot for a modality plugin (e.g., Vision, Audio).
pub trait ModalitySlot: Send + Sync {
    fn name(&self) -> &str;
    fn as_any(&self) -> &dyn Any;
    fn process_input(&self, input: &[u8]) -> Option<HyperVector>;
}

pub trait ModalEncoder: Send + Sync {
    fn encode_bytes(&self, data: &[u8]) -> HyperVector;
}

pub trait ModalDecoder: Send + Sync {
     fn decode_bytes(&self, hv: &HyperVector) -> Vec<u8>;
}
