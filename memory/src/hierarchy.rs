use core_vsa::HyperVector;
use std::error::Error;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MemoryLayer {
    Working,
    Episodic,
    Semantic, // Renamed from Invariant
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct MemoryEntry {
    pub vector: HyperVector,
    pub timestamp: u64,
    pub access_count: u32,
    pub last_access: u64,
    pub layer: String,
    pub importance: f32,
    pub confidence: f32,
    pub source_reliability: f32,
    pub reinforcement_count: u32,
}

impl MemoryEntry {
    pub fn new(vector: HyperVector, layer: MemoryLayer) -> Self {
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
        let layer_str = match layer {
            MemoryLayer::Working => "working",
            MemoryLayer::Episodic => "episodic",
            MemoryLayer::Semantic => "semantic",
        };
        Self {
            vector,
            timestamp: now,
            access_count: 1,
            last_access: now,
            layer: layer_str.to_string(),
            importance: 1.0,
            confidence: 0.5,
            source_reliability: 1.0,
            reinforcement_count: 1,
        }
    }

    pub fn update_access(&mut self) {
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
        self.access_count += 1;
        self.last_access = now;
        self.importance += 0.1;
    }

    pub fn reinforce(&mut self, reliability: f32) {
        let n = self.reinforcement_count as f32;
        self.confidence = (self.confidence * n + reliability) / (n + 1.0);
        self.reinforcement_count += 1;
        self.importance += 0.5;
    }

    pub fn decay(&mut self, rate: f32) {
        self.importance *= rate;
        if self.reinforcement_count < 2 {
            self.confidence *= rate;
        }
    }
}

pub trait HierarchicalMemory {
    fn store_in_layer(&mut self, key: &str, vector: HyperVector, layer: MemoryLayer) -> Result<(), Box<dyn Error>>;
    fn retrieve_with_metrics(&mut self, key: &str) -> Option<HyperVector>;
    fn consolidate_layers(&mut self);
}
