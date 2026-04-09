use crate::OmniMind;
use tracing::info;
use memory::{MemoryLayer, HierarchicalMemory};
use core_vsa::traits::MemoryStore;

pub struct LearningEngine;

pub struct ContinuousLearningEngine;

impl ContinuousLearningEngine {
    /// Industrial Relevance Weighting: Adjusts result scores based on both global importance and current context.
    pub fn weight_relevance(
        mind: &OmniMind,
        results: &mut Vec<(String, f32)>,
        context: &core_vsa::HyperVector
    ) {
        for (key, score) in results.iter_mut() {
            let mut boost = 1.0;

            // 1. Contextual Similarity Boost
            let active_mem = mind.state_memory();
            if let Some(vec) = MemoryStore::retrieve(active_mem, key) {
                let context_sim = vec.similarity(context);
                if context_sim > 0.1 {
                    boost += context_sim * 0.5; // Up to 50% boost for contextual fit
                }
            }

            // 2. Global Importance Factor
            if let Some(entry) = mind.state_memory().metadata.get(key) {
                // Normalize importance 0-5 to a 0.0-0.2 boost
                boost += (entry.importance.min(5.0) / 5.0) * 0.2;
            }

            *score *= boost;
        }

        // Re-sort after weighting
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    }

    /// Incremental Learning: Reinforce high-confidence reasoning paths.
    pub fn autonomous_reinforcement(mind: &mut OmniMind) {
        let _prof = runtime::ProfileScope::new("ContinuousLearning::autonomous_reinforcement");

        let mut reinforcements = Vec::new();
        if let crate::LifecycleState::Forge(_, cog) = &mind.state {
            for fact in cog.knowledge_graph.facts.values() {
                if fact.get_composite_confidence() > 0.9 {
                    reinforcements.push(fact.id.clone());
                }
            }
        }

        for id in reinforcements {
            LearningEngine::process_feedback(mind, &id, 0.05); // Tiny autonomous boost
        }
    }
}

impl LearningEngine {
    pub fn process_feedback(mind: &mut OmniMind, key: &str, score: f32) {
        info!("Processing feedback for {}: score={}", key, score);
        match &mut mind.state {
            crate::LifecycleState::Forge(mem, cog) => {
                let mut found = false;
                if let Some(entry) = mem.metadata.get_mut(key) {
                    if score > 0.0 {
                        entry.reinforce(score);
                    } else {
                        entry.confidence *= (1.0 + score).max(0.1);
                        entry.importance *= (1.0 + score).max(0.1);
                    }
                    found = true;
                }

                if score > 0.0 {
                    cog.reinforce_knowledge(key, score);
                } else {
                    cog.penalize_knowledge(key, -score);
                }

                if !found && !cog.knowledge_graph.facts.contains_key(key) {
                    info!("Feedback: Key {} not found in memory or cognition.", key);
                }
            },
            crate::LifecycleState::Runtime(_, delta, cog) => {
                let mut found = false;
                if let Some(entry) = delta.metadata.get_mut(key) {
                    if score > 0.0 {
                        entry.reinforce(score);
                    } else {
                        entry.confidence *= (1.0 + score).max(0.1);
                        entry.importance *= (1.0 + score).max(0.1);
                    }
                    found = true;
                }

                if score > 0.0 {
                    cog.reinforce_knowledge(key, score);
                } else {
                    cog.penalize_knowledge(key, -score);
                }

                if !found && !cog.knowledge_graph.facts.contains_key(key) {
                    info!("Feedback: Key {} not found in memory or cognition.", key);
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
