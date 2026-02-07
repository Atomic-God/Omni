use crate::traits::ModalitySlot;
use core_vsa::HyperVector;
use std::any::Any;
use std::collections::HashMap;
use log::info;

// --- Concrete Implementations ---

pub struct TextModality;
impl ModalitySlot for TextModality {
    fn name(&self) -> &str { "Language (Text)" }
    fn as_any(&self) -> &dyn Any { self }
    fn process_input(&self, input: &[u8]) -> Option<HyperVector> {
        // Assume UTF-8 text for now
        if let Ok(_text) = std::str::from_utf8(input) {
            // In a real implementation, this would call the tokenizer/encoder
            // For now, return a deterministic hash based vector as a placeholder
            // or delegate to `text_encoder` if accessible.
            // Since this is just the Abstraction Layer, we can return None or a placeholder.
            // But let's simulate "Processing"
            let mut sum_vec = HyperVector::deterministic(0);
            // Just a dummy hash-based summation for the abstraction proof
            for byte in input {
                 sum_vec = sum_vec.bundle(&HyperVector::deterministic(*byte as u64));
            }
            Some(sum_vec)
        } else {
            None
        }
    }
}

pub struct VisionModality;
impl ModalitySlot for VisionModality {
    fn name(&self) -> &str { "Vision (Symbolic Scaffold)" }
    fn as_any(&self) -> &dyn Any { self }
    fn process_input(&self, _input: &[u8]) -> Option<HyperVector> {
        // Vision would process image bytes.
        // Returning a placeholder for "Visual Scene"
        info!("Vision Modality received input. Generating symbolic scene vector.");
        Some(HyperVector::deterministic(0xCAFEBABE))
    }
}

pub struct AudioModality;
impl ModalitySlot for AudioModality {
    fn name(&self) -> &str { "Audio (Frequency-Symbol)" }
    fn as_any(&self) -> &dyn Any { self }
    fn process_input(&self, _input: &[u8]) -> Option<HyperVector> {
        // Audio would process PCM data.
        info!("Audio Modality received input. Mapping frequency to symbol.");
        Some(HyperVector::deterministic(0xDEADBEEF))
    }
}

// --- Registry ---

pub struct ModalityRegistry {
    modalities: HashMap<String, Box<dyn ModalitySlot>>,
}

impl Default for ModalityRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ModalityRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            modalities: HashMap::new(),
        };
        // Register default modalities
        registry.register(Box::new(TextModality));
        registry.register(Box::new(VisionModality));
        registry.register(Box::new(AudioModality));
        registry
    }

    pub fn register(&mut self, modality: Box<dyn ModalitySlot>) {
        let name = modality.name().to_string();
        info!("Registering Modality: {}", name);
        self.modalities.insert(name, modality);
    }

    pub fn get(&self, name: &str) -> Option<&dyn ModalitySlot> {
        self.modalities.get(name).map(|b| b.as_ref())
    }

    pub fn list_capabilities(&self) -> Vec<String> {
        self.modalities.keys().cloned().collect()
    }
}
