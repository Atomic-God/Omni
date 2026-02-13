use core_vsa::HyperVector;
use std::collections::HashMap;
use std::error::Error;

/// Defines the layers of memory in the cognitive architecture.
pub enum MemoryLayer {
    Working,   // High-speed, low-capacity, transient
    Episodic,  // Event-based, medium-capacity
    Invariant, // Fact-based, high-capacity, immutable/slow-decay
}

/// A structured memory entry with importance metrics.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct MemoryEntry {
    pub vector: HyperVector,
    pub timestamp: u64,
    pub access_count: u32,
    pub last_access: u64,
    pub layer: String, // "working", "episodic", "invariant"
    pub importance: f32, // Computed score
}

impl MemoryEntry {
    pub fn new(vector: HyperVector, layer: MemoryLayer) -> Self {
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
        let layer_str = match layer {
            MemoryLayer::Working => "working",
            MemoryLayer::Episodic => "episodic",
            MemoryLayer::Invariant => "invariant",
        };
        Self {
            vector,
            timestamp: now,
            access_count: 1,
            last_access: now,
            layer: layer_str.to_string(),
            importance: 1.0,
        }
    }

    pub fn update_access(&mut self) {
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
        self.access_count += 1;
        self.last_access = now;
        // Simple importance boost
        self.importance += 0.1;
    }

    pub fn decay(&mut self, rate: f32) {
        self.importance *= rate;
    }
}

pub trait HierarchicalMemory {
    fn store_in_layer(&mut self, key: &str, vector: HyperVector, layer: MemoryLayer) -> Result<(), Box<dyn Error>>;
    fn retrieve_with_metrics(&mut self, key: &str) -> Option<HyperVector>;
    fn consolidate_layers(&mut self);
}
