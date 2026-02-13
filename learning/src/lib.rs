use core_vsa::{HyperVector, traits::MemoryStore};
use memory::{MemoryManager, MemoryLayer, HierarchicalMemory};
use ingestion::UniversalIngestor;
use core_vsa::traits::Ingestor;
use std::path::Path;
use log::{info, warn};

pub struct RuntimeLearner {
    pub memory: MemoryManager, // Owned or Reference? For continuous loop, it owns the memory state.
    pub ingestor: UniversalIngestor,
    pub decay_rate: f32,
}

impl RuntimeLearner {
    pub fn new(memory_path: &Path) -> Self {
        Self {
            memory: MemoryManager::new(memory_path),
            ingestor: UniversalIngestor,
            decay_rate: 0.99, // 1% decay per cycle
        }
    }

    /// Primary Loop: Ingest -> Encode -> Integrate -> Consolidate
    pub fn process_input(&mut self, input_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        info!("Learning from {:?}", input_path);

        // 1. Ingest
        let graph = self.ingestor.ingest(input_path)?;

        // 2. Encode & Integrate
        for node in graph.nodes {
            // Compute importance based on metadata?
            // "Semantic significance" -> placeholder logic: Text length or edge count?
            // Let's assume importance is 1.0 initially.
            // Store in Working Memory
            self.memory.store_in_layer(&node.id, node.vector, MemoryLayer::Working)?;
        }

        // 3. Consolidate
        self.run_consolidation_cycle();

        Ok(())
    }

    pub fn run_consolidation_cycle(&mut self) {
        info!("Running consolidation cycle...");
        // Decay all items
        for entry in self.memory.metadata.values_mut() {
            entry.decay(self.decay_rate);
        }

        // Promote layers
        self.memory.consolidate_layers();

        // Pruning?
        // Remove items with importance < 0.1?
        self.memory.metadata.retain(|_, v| v.importance > 0.1);

        // Save snapshot periodically? (Handled by outer loop)
    }
}
