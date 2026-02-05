pub mod decoder;
pub mod text_encoder;
pub mod tokenizer;
pub mod traits;
pub mod vision_stub;
pub mod audio_stub;

use core_vsa::HyperVector;
use std::collections::HashMap;

// Placeholder for loading external weights
pub fn load_encoder_weights(_path: &str) -> HashMap<String, HyperVector> {
    HashMap::new()
}
