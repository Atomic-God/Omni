use serde::{Serialize, Deserialize};
use std::path::Path;
use std::fs::File;
use log::info;
use neural::Tensor;

#[derive(Serialize, Deserialize)]
pub struct Checkpoint {
    pub epoch: usize,
    pub model_state: Vec<Tensor>, // Simplified: list of tensor data
    pub optimizer_state: Vec<Tensor>,
}

pub struct CheckpointManager {
    pub checkpoint_dir: std::path::PathBuf,
}

impl CheckpointManager {
    pub fn new(dir: &Path) -> Self {
        std::fs::create_dir_all(dir).unwrap();
        Self { checkpoint_dir: dir.to_path_buf() }
    }

    pub fn save(&self, epoch: usize, model_params: &[&mut Tensor]) {
        // Collect data (clone)
        let model_state: Vec<Tensor> = model_params.iter().map(|t| (*t).clone()).collect();
        let optimizer_state = Vec::new(); // Placeholder

        let cp = Checkpoint { epoch, model_state, optimizer_state };
        let path = self.checkpoint_dir.join(format!("checkpoint_epoch_{}.json", epoch));

        let f = File::create(path).unwrap();
        serde_json::to_writer(f, &cp).unwrap();
        info!("Saved checkpoint for epoch {}", epoch);
    }
}
