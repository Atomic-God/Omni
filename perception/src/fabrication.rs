// Multimodal Fabrication Interfaces
// CPU-Safe definitions for GPU Handoff

pub trait VisionFabricator: Send + Sync {
    fn ingest_images(&mut self, folder: &str) -> Result<(), String>;
    fn train_inr(&mut self) -> Result<String, String>; // Returns path to weights
}

pub trait AudioFabricator: Send + Sync {
    fn ingest_audio(&mut self, folder: &str) -> Result<(), String>;
    fn train_vsa(&mut self) -> Result<String, String>;
}

pub trait INRTrainer: Send + Sync {
    fn optimize(&mut self, epochs: usize) -> f32; // loss
}

pub trait ResonatorGPU: Send + Sync {
    fn factorize_batch(&self, vectors: &[Vec<f32>]) -> Vec<Vec<f32>>;
}

#[derive(Debug)]
pub struct FabricationManifest {
    pub vision_weights: Option<String>,
    pub audio_weights: Option<String>,
    pub target_platform: String,
}
