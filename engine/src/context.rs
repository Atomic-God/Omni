use core_vsa::{HyperVector, traits::MemoryStore};
use memory::MemoryManager;
use cognition::Intent;
use std::collections::VecDeque;
use log::{info, warn};

/// Manages the active working context of the agent.
pub struct ContextManager {
    // Short-term sliding window of recent observations
    pub working_context: VecDeque<HyperVector>,
    pub max_working_size: usize,

    // Active Goals
    pub active_goals: Vec<crate::governance::Goal>, // Assuming existing Goal struct

    // Episodic Recall Buffer (retrieved memories relevant to current context)
    pub episodic_buffer: Vec<HyperVector>,
}

impl ContextManager {
    pub fn new() -> Self {
        Self {
            working_context: VecDeque::new(),
            max_working_size: 10,
            active_goals: Vec::new(),
            episodic_buffer: Vec::new(),
        }
    }

    /// Update context with new observation
    pub fn update(&mut self, observation: &HyperVector) {
        if self.working_context.len() >= self.max_working_size {
            self.working_context.pop_front();
        }
        self.working_context.push_back(observation.clone());
    }

    /// Retrieve relevant episodes from memory based on current working context
    pub fn recall_episodes(&mut self, memory: &MemoryManager) {
        // Form query vector from working context (e.g. bundle or last item)
        if let Some(current) = self.working_context.back() {
            // Query memory
            // Phase 6: We use `query_nearest` from MemoryStore trait
            let results = memory.query_nearest(current, 5);
            self.episodic_buffer.clear();
            for (id, _score) in results {
                if let Some(vec) = memory.retrieve(&id) {
                    self.episodic_buffer.push(vec);
                }
            }
            info!("Recalled {} relevant episodes.", self.episodic_buffer.len());
        }
    }

    /// Rank relevance of an item against current context
    pub fn rank_relevance(&self, item: &HyperVector) -> f32 {
        if let Some(current) = self.working_context.back() {
            current.similarity(item)
        } else {
            0.0
        }
    }

    /// Compress context into a single summary vector
    pub fn compress_context(&self) -> HyperVector {
        // Bundle all working context vectors?
        // Simple superposition.
        if self.working_context.is_empty() {
            return HyperVector::deterministic(0); // Zero-like
        }

        let mut summary = self.working_context[0].clone();
        for i in 1..self.working_context.len() {
            summary = summary.bundle(&self.working_context[i]);
        }
        summary
    }
}
