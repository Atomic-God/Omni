use crate::OmniMind;
use tracing::info;
use memory::{MemoryLayer, HierarchicalMemory};

pub struct LearningEngine;

impl LearningEngine {
    pub fn process_feedback(mind: &mut OmniMind, key: &str, score: f32) {
        info!("Processing feedback for {}: score={}", key, score);
        match &mut mind.state {
            crate::LifecycleState::Forge(mem, cog) => {
                if let Some(entry) = mem.metadata.get_mut(key) {
                    if score > 0.0 {
                        entry.reinforce(score);
                        cog.reinforce_knowledge(key, score);
                    } else {
                        entry.confidence *= (1.0 + score).max(0.1);
                        entry.importance *= (1.0 + score).max(0.1);
                    }
                }
            },
            crate::LifecycleState::Runtime(_, delta, cog) => {
                if let Some(entry) = delta.metadata.get_mut(key) {
                    if score > 0.0 {
                        entry.reinforce(score);
                        cog.reinforce_knowledge(key, score);
                    } else {
                        entry.confidence *= (1.0 + score).max(0.1);
                        entry.importance *= (1.0 + score).max(0.1);
                    }
                }
            }
        }
    }

    pub fn incremental_update(mind: &mut OmniMind, key: &str, new_data: &str) {
        info!("Incremental learning update for {}", key);
        let new_vector = mind.encode_text(new_data);

        match &mut mind.state {
            crate::LifecycleState::Forge(mem, _) => {
                if let Some(entry) = mem.metadata.get_mut(key) {
                    entry.vector = entry.vector.bundle(&new_vector);
                    entry.reinforce(0.5);
                } else {
                    let _ = mem.store_in_layer(key, new_vector, MemoryLayer::Working);
                }
            },
            crate::LifecycleState::Runtime(_, delta, _) => {
                if let Some(entry) = delta.metadata.get_mut(key) {
                    entry.vector = entry.vector.bundle(&new_vector);
                    entry.reinforce(0.5);
                } else {
                    let _ = delta.store_in_layer(key, new_vector, MemoryLayer::Working);
                }
            }
        }
    }

    pub fn reward_usage(mind: &mut OmniMind, key: &str) {
        Self::process_feedback(mind, key, 1.0);
    }

    pub fn penalize_usage(mind: &mut OmniMind, key: &str) {
        Self::process_feedback(mind, key, -0.2);
    }
}
