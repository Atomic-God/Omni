use core_vsa::HyperVector;
use crate::hierarchy::{MemoryEntry, MemoryLayer};

pub struct LifecycleManager;

impl LifecycleManager {
    /// Determines if a memory entry should be promoted to a higher layer.
    pub fn evaluate_promotion(entry: &MemoryEntry) -> Option<MemoryLayer> {
        let current = match entry.layer.as_str() {
            "working" => MemoryLayer::Working,
            "episodic" => MemoryLayer::Episodic,
            "semantic" => MemoryLayer::Semantic,
            _ => return None,
        };

        match current {
            MemoryLayer::Working => {
                // Working -> Episodic: Needs reinforcement or high access
                if entry.reinforcement_count >= 3 || entry.access_count > 5 {
                    return Some(MemoryLayer::Episodic);
                }
            }
            MemoryLayer::Episodic => {
                // Episodic -> Semantic: Needs high confidence and stability
                if entry.confidence > 0.8 && entry.reinforcement_count > 10 {
                    return Some(MemoryLayer::Semantic);
                }
            }
            MemoryLayer::Semantic => return None,
        }
        None
    }

    /// Determines if a memory entry should be evicted (deleted) due to low importance.
    pub fn evaluate_eviction(entry: &MemoryEntry, threshold: f32) -> bool {
        entry.importance < threshold && entry.layer != "semantic"
    }
}

pub struct EpisodicEncoder {
    pub time_seed: HyperVector,
}

impl EpisodicEncoder {
    pub fn new() -> Self {
        Self {
            time_seed: HyperVector::deterministic(0x71337),
        }
    }

    /// Binds an event with its temporal context.
    pub fn encode_event(&self, event: &HyperVector, timestamp: u64) -> HyperVector {
        let time_context = self.time_seed.permute((timestamp % 10000) as usize);
        event.bind(&time_context)
    }
}
