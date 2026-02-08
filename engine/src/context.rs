use cognition::planning::Goal;
use std::collections::VecDeque;

#[derive(Clone, Debug)]
pub struct ContextManager {
    pub goals: VecDeque<Goal>, // Goal Stack
    pub working_memory: VecDeque<String>, // Active Semantic Concepts
    pub episodic_buffer: VecDeque<String>, // Recent Raw Interactions
    pub budget: usize,
}

impl ContextManager {
    pub fn new() -> Self {
        Self {
            goals: VecDeque::new(),
            working_memory: VecDeque::new(),
            episodic_buffer: VecDeque::new(),
            budget: 7, // Miller's Law default
        }
    }

    pub fn push_goal(&mut self, goal: Goal) {
        // Higher priority should be at the front?
        // Goals are usually LIFO (stack) for sub-tasks.
        self.goals.push_front(goal);
    }

    pub fn pop_goal(&mut self) -> Option<Goal> {
        self.goals.pop_front()
    }

    pub fn activate(&mut self, concept: &str) {
        // Activate Semantic Memory
        if !self.working_memory.contains(&concept.to_string()) {
            self.working_memory.push_back(concept.to_string());
            if self.working_memory.len() > self.budget {
                self.working_memory.pop_front();
            }
        }
    }

    pub fn log_interaction(&mut self, text: &str) {
        // Log Episodic Event
        self.episodic_buffer.push_back(text.to_string());
        if self.episodic_buffer.len() > 10 { // Short episodic window
            self.episodic_buffer.pop_front();
        }
    }

    pub fn get_active_context(&self) -> Vec<String> {
        self.working_memory.iter().cloned().collect()
    }
}
