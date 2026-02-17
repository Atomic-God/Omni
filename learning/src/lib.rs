#![deny(warnings)]
use memory::{MemoryManager, MemoryLayer, HierarchicalMemory};
use ingestion::UniversalIngestor;
use core_vsa::traits::Ingestor;
use std::path::Path;
use tracing::info;
use crate::consolidation::ConsolidationEngine;

pub mod consolidation;

pub struct ConfidenceTuner {
    pub learning_rate: f32,
    pub stability_count: u32,
}

impl ConfidenceTuner {
    pub fn new() -> Self {
        Self {
            learning_rate: 0.1,
            stability_count: 0,
        }
    }

    pub fn adjust(&mut self, success: bool) {
        if success {
            self.learning_rate *= 0.95; // Become more conservative as we succeed
            self.stability_count += 1;
        } else {
            self.learning_rate *= 1.2; // Become more radical as we fail
            self.stability_count = 0;
        }
        self.learning_rate = self.learning_rate.clamp(0.01, 1.0);
    }
}

pub struct RuntimeLearner {
    pub memory: MemoryManager,
    pub ingestor: UniversalIngestor,
    pub decay_rate: f32,
    pub similarity_threshold: f32,
    pub tuner: ConfidenceTuner,
}

impl RuntimeLearner {
    pub fn new(memory_path: &Path) -> Self {
        Self {
            memory: MemoryManager::new(memory_path),
            ingestor: UniversalIngestor::new(),
            decay_rate: 0.99,
            similarity_threshold: 0.85,
            tuner: ConfidenceTuner::new(),
        }
    }

    pub fn process_input(&mut self, input_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        info!("Learning from {:?}", input_path);
        let graph = self.ingestor.ingest(input_path).map_err(|e| e.to_string())?;

        for node in graph.nodes {
            let _ = self.memory.store_in_layer(&node.id, node.vector, MemoryLayer::Working);
        }

        self.run_consolidation_cycle();
        Ok(())
    }

    pub fn feedback_loop(&mut self, success: bool) {
        self.tuner.adjust(success);
        info!("Confidence Tuner: Learning Rate now {:.4}", self.tuner.learning_rate);
    }

    pub fn run_consolidation_cycle(&mut self) {
        info!("Running consolidation cycle...");
        ConsolidationEngine::decay_and_prune(&mut self.memory.metadata, self.decay_rate, 0.1);
        self.memory.consolidate_layers();
    }
}
