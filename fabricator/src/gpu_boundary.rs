// GPU Fabrication Interfaces (Future Phase)
// Defines traits for training and encoding on GPU.

pub trait FoundationTrainer {
    fn train_epoch(&mut self, data: &[u8]);
    fn save_checkpoint(&self, path: &str);
}

pub trait PerceptualEncoderTrainer {
    fn train_encoder(&mut self, modality: &str, data: &[u8]);
}

pub trait INRTrainer {
    fn train_inr(&mut self, target_signal: &[f32]);
}

pub trait SequenceResonatorTrainer {
    fn train_sequence(&mut self, sequence: &[u64]);
}

pub struct FabricationGPU;

impl FabricationGPU {
    pub fn new() -> Self {
        Self
    }
}
