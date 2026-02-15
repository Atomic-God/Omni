use core_vsa::{HyperVector, traits::MemoryStore};
use memory::MemoryManager;
use std::collections::VecDeque;
use log::{info, warn};
use runtime::Goal;

pub struct ContextManager {
    pub working_context: VecDeque<HyperVector>,
    pub max_working_size: usize,
    pub active_goals: Vec<Goal>,
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

    pub fn add_goal(&mut self, goal: Goal) {
        info!("Arbitration: New goal added - {} (Priority: {})", goal.description, goal.priority);
        self.active_goals.push(goal);
        self.prioritize_goals();
    }

    fn prioritize_goals(&mut self) {
        // Sort by priority descending, then by creation time
        self.active_goals.sort_by(|a, b| {
            b.priority.cmp(&a.priority).then(a.created_at.cmp(&b.created_at))
        });
    }

    pub fn resolve_conflicts(&mut self) {
        // If two goals have high similarity but different intents, reduce priority of the newer one
        // For Phase 1, we use simple priority-based preemption
        if self.active_goals.len() > 5 {
            warn!("Goal overload detected. Pruning lowest priority goals.");
            self.active_goals.truncate(5);
        }
    }

    pub fn update(&mut self, observation: &HyperVector) {
        if self.working_context.len() >= self.max_working_size {
            self.working_context.pop_front();
        }
        self.working_context.push_back(observation.clone());
    }

    pub fn recall_episodes(&mut self, memory: &MemoryManager) {
        if let Some(current) = self.working_context.back() {
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

    pub fn rank_relevance(&self, item: &HyperVector) -> f32 {
        if let Some(current) = self.working_context.back() {
            current.similarity(item)
        } else {
            0.0
        }
    }

    pub fn compress_context(&self) -> HyperVector {
        if self.working_context.is_empty() {
            return HyperVector::deterministic(0);
        }

        let mut summary = self.working_context[0].clone();
        for i in 1..self.working_context.len() {
            summary = summary.bundle(&self.working_context[i]);
        }
        summary
    }
}
