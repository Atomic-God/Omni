use cognition::planning::Goal;
use std::collections::VecDeque;

#[derive(Clone, Debug)]
pub struct ContextManager {
    pub goals: VecDeque<Goal>,
    pub working_memory: VecDeque<String>, // Active concepts
    pub budget: usize,
}

impl ContextManager {
    pub fn new() -> Self {
        Self {
            goals: VecDeque::new(),
            working_memory: VecDeque::new(),
            budget: 7, // Miller's Law default
        }
    }

    pub fn add_goal(&mut self, goal: Goal) {
        self.goals.push_back(goal);
    }

    pub fn activate(&mut self, concept: &str) {
        // Simple FIFO working memory
        if self.working_memory.contains(&concept.to_string()) {
            return; // Already active
        }

        self.working_memory.push_back(concept.to_string());
        if self.working_memory.len() > self.budget {
            self.working_memory.pop_front();
        }
    }

    pub fn get_active_context(&self) -> Vec<String> {
        self.working_memory.iter().cloned().collect()
    }
}
