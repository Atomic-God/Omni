use cognition::planning::Goal;
use std::collections::VecDeque;

#[derive(Clone, Debug)]
pub struct ContextManager {
    pub goals: VecDeque<Goal>, // Goal Stack (LIFO for subgoals)
    pub semantic_memory: VecDeque<String>, // Active Semantic Concepts
    pub episodic_buffer: VecDeque<String>, // Recent Raw Interactions
    pub budget: usize,
}

impl ContextManager {
    pub fn new() -> Self {
        Self {
            goals: VecDeque::new(),
            semantic_memory: VecDeque::new(),
            episodic_buffer: VecDeque::new(),
            budget: 7, // Miller's Law default
        }
    }

    pub fn push_goal(&mut self, mut goal: Goal) {
        // Priority logic: if high priority, push to front (active)
        // If lower, maybe push back?
        // Standard goal stack is usually LIFO for execution.
        // Let's assume strict LIFO for now, but sort by priority if needed.
        if goal.priority > 50 {
             self.goals.push_front(goal);
        } else {
             // For lower priority, maybe just append?
             // But a stack usually means "current focus".
             // Let's stick to LIFO but mark priority.
             self.goals.push_front(goal);
        }
    }

    pub fn pop_goal(&mut self) -> Option<Goal> {
        self.goals.pop_front()
    }

    pub fn activate_semantic(&mut self, concept: &str) {
        // Semantic Activation (Recency based)
        if !self.semantic_memory.contains(&concept.to_string()) {
            self.semantic_memory.push_back(concept.to_string());
            if self.semantic_memory.len() > self.budget {
                self.semantic_memory.pop_front();
            }
        } else {
            // Re-activate (move to end)
            // Ideally we'd remove and push, but VecDeque remove is O(N).
            // For small budget (7), O(N) is fine.
            if let Some(pos) = self.semantic_memory.iter().position(|x| x == concept) {
                self.semantic_memory.remove(pos);
                self.semantic_memory.push_back(concept.to_string());
            }
        }
    }

    pub fn log_episodic(&mut self, text: &str) {
        // Log Episodic Event
        self.episodic_buffer.push_back(text.to_string());
        if self.episodic_buffer.len() > 10 { // Short episodic window
            self.episodic_buffer.pop_front();
        }
    }

    pub fn get_active_context(&self) -> Vec<String> {
        self.semantic_memory.iter().cloned().collect()
    }
}
