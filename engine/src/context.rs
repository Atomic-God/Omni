use cognition::planning::Goal;
use std::collections::VecDeque;

#[derive(Clone, Debug)]
pub struct TopicTracker {
    pub active_topics: VecDeque<String>,
}

impl TopicTracker {
    pub fn new() -> Self {
        Self {
            active_topics: VecDeque::new(),
        }
    }

    pub fn track(&mut self, text: &str) {
        // Naive topic extraction: first noun-like token?
        // Or just the whole query if short.
        if text.len() < 50 {
            self.active_topics.push_back(text.to_string());
            if self.active_topics.len() > 5 {
                self.active_topics.pop_front();
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct ContextManager {
    pub goals: VecDeque<Goal>, // Goal Stack
    pub semantic_memory: VecDeque<String>, // Active Semantic Concepts
    pub episodic_buffer: VecDeque<String>, // Recent Raw Interactions
    pub budget: usize,
    pub topic_tracker: TopicTracker, // Added
}

impl ContextManager {
    pub fn new() -> Self {
        Self {
            goals: VecDeque::new(),
            semantic_memory: VecDeque::new(),
            episodic_buffer: VecDeque::new(),
            budget: 7, // Miller's Law default
            topic_tracker: TopicTracker::new(),
        }
    }

    pub fn push_goal(&mut self, goal: Goal) {
        if goal.priority > 50 {
             self.goals.push_front(goal);
        } else {
             self.goals.push_front(goal); // Default LIFO
        }
    }

    pub fn pop_goal(&mut self) -> Option<Goal> {
        self.goals.pop_front()
    }

    pub fn activate_semantic(&mut self, concept: &str) {
        if !self.semantic_memory.contains(&concept.to_string()) {
            self.semantic_memory.push_back(concept.to_string());
            if self.semantic_memory.len() > self.budget {
                self.semantic_memory.pop_front();
            }
        } else {
            if let Some(pos) = self.semantic_memory.iter().position(|x| x == concept) {
                self.semantic_memory.remove(pos);
                self.semantic_memory.push_back(concept.to_string());
            }
        }
        // Also track topic
        self.topic_tracker.track(concept);
    }

    pub fn log_episodic(&mut self, text: &str) {
        self.episodic_buffer.push_back(text.to_string());
        if self.episodic_buffer.len() > 10 { // Short episodic window
            self.episodic_buffer.pop_front();
        }
    }

    pub fn get_active_context(&self) -> Vec<String> {
        self.semantic_memory.iter().cloned().collect()
    }
}
