#![deny(warnings)]
use core_vsa::HyperVector;
use serde::{Serialize, Deserialize};

pub mod vision;
pub mod audio;
pub mod bridge;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Modality {
    Text,
    Vision,
    Audio,
    CrossModal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModalityTag {
    pub modality: Modality,
    pub confidence: f32,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticVector {
    pub vector: HyperVector,
    pub tag: ModalityTag,
    pub label: String,
}

impl SemanticVector {
    pub fn new(vector: HyperVector, modality: Modality, label: String, source: &str) -> Self {
        Self {
            vector,
            tag: ModalityTag {
                modality,
                confidence: 1.0,
                source: source.to_string(),
            },
            label,
        }
    }
}
