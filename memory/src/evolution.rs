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

pub struct ForgettingEngine;

impl ForgettingEngine {
    /// Applies Ebbinghaus forgetting curve and importance-based pruning.
    pub fn prune_fading_memories(entries: &mut std::collections::HashMap<String, MemoryEntry>, base_decay: f32) {
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
        let mut to_remove = Vec::new();

        for (id, entry) in entries.iter_mut() {
            if entry.layer == "semantic" { continue; }

            let time_since_last = now.saturating_sub(entry.last_access) as f32;

            // Retrievability R = exp(-t / S)
            let retrievability = (-time_since_last / (entry.stability * 3600.0)).exp();

            entry.importance *= retrievability * base_decay;

            if entry.importance < 0.01 {
                to_remove.push(id.clone());
            }
        }

        for id in to_remove {
            entries.remove(&id);
        }
    }

    /// Prunes entries with low industrial utility (low access + low reinforcement).
    pub fn prune_by_utility(entries: &mut std::collections::HashMap<String, MemoryEntry>, threshold: f32) {
        let mut to_remove = Vec::new();
        for (id, entry) in entries.iter() {
            if entry.layer == "semantic" { continue; }

            // Utility U = (Access * 0.4) + (Reinforcement * 0.6)
            let utility = (entry.access_count as f32 * 0.4) + (entry.reinforcement_count as f32 * 0.6);
            if utility < threshold {
                to_remove.push(id.clone());
            }
        }
        for id in to_remove {
            entries.remove(&id);
        }
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
