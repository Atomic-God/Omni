use core_vsa::{HyperVector, traits::MemoryStore};
use memory::{MemoryManager, MemoryLayer, HierarchicalMemory};
use ingestion::{UniversalIngestor, ingest_graph};
use core_vsa::traits::Ingestor;
use std::path::Path;
use log::{info, warn};
use crate::consolidation::ConsolidationEngine;

pub mod consolidation;

pub struct RuntimeLearner {
    pub memory: MemoryManager,
    pub ingestor: UniversalIngestor,
    pub decay_rate: f32,
    pub similarity_threshold: f32,
}

impl RuntimeLearner {
    pub fn new(memory_path: &Path) -> Self {
        Self {
            memory: MemoryManager::new(memory_path),
            ingestor: UniversalIngestor,
            decay_rate: 0.99,
            similarity_threshold: 0.85,
        }
    }

    pub fn process_input(&mut self, input_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        info!("Learning from {:?}", input_path);
        let graph = self.ingestor.ingest(input_path)?;

        for node in graph.nodes {
            self.memory.store_in_layer(&node.id, node.vector, MemoryLayer::Working)?;
        }

        self.run_consolidation_cycle();
        Ok(())
    }

    pub fn run_consolidation_cycle(&mut self) {
        info!("Running consolidation cycle...");

        // 1. Decay & Prune Metadata
        ConsolidationEngine::decay_and_prune(&mut self.memory.metadata, self.decay_rate, 0.1);

        // 2. Consolidate Vectors (Clustering) - Requires extracting vectors from storage?
        // MemoryManager keeps storage separate.
        // We can only consolidate what's in 'metadata' if it stores vectors?
        // Yes, MemoryEntry stores vector.

        // Extract map for consolidation
        let mut vector_map = std::collections::HashMap::new();
        for (k, v) in &self.memory.metadata {
            vector_map.insert(k.clone(), v.vector.clone());
        }

        let merged = ConsolidationEngine::consolidate(&mut vector_map, self.similarity_threshold);

        // Write back merged vectors
        for (k, v) in merged {
            // Update vector in memory entry
            if let Some(entry) = self.memory.metadata.get_mut(&k) {
                entry.vector = v;
                // Boost importance of merged concept
                entry.importance += 1.0;
                info!("Merged concept updated: {}", k);
            }
        }

        // 3. Promote Layers
        self.memory.consolidate_layers();
    }
}
