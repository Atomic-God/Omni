use cognition::planning::Goal;
use std::collections::VecDeque;
use crate::governance::{SalienceScoring, MemoryTier};

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
    pub goals: VecDeque<Goal>,

    // Tiers
    pub short_term_memory: MemoryTier,
    pub long_term_buffer: MemoryTier,

    // Governance
    pub salience: SalienceScoring,
    pub topic_tracker: TopicTracker,
}

impl ContextManager {
    pub fn new() -> Self {
        Self {
            goals: VecDeque::new(),
            short_term_memory: MemoryTier::new(7), // Miller's Law
            long_term_buffer: MemoryTier::new(100), // Recent history
            salience: SalienceScoring::new(0.05), // 5% hourly decay
            topic_tracker: TopicTracker::new(),
        }
    }

    pub fn push_goal(&mut self, goal: Goal) {
        if goal.priority > 50 {
             self.goals.push_front(goal);
        } else {
             self.goals.push_front(goal);
        }
    }

    pub fn pop_goal(&mut self) -> Option<Goal> {
        self.goals.pop_front()
    }

    pub fn activate_semantic(&mut self, concept: &str) {
        // Boost Salience
        self.salience.score(concept, 1.0);

        // Add to STM, if evicted, try LTM
        if let Some(evicted) = self.short_term_memory.add(concept.to_string()) {
            self.long_term_buffer.add(evicted);
        }

        self.topic_tracker.track(concept);
    }

    pub fn log_episodic(&mut self, text: &str) {
        // Ephemeral log
        self.long_term_buffer.add(text.to_string());
    }

    pub fn get_active_context(&self) -> Vec<String> {
        // Return STM sorted by Salience?
        let mut context: Vec<String> = self.short_term_memory.items.iter().cloned().collect();
        // Maybe inject high salience LTM items?
        let top_ltm = self.salience.get_top(3);
        for item in top_ltm {
            if !context.contains(&item) {
                context.push(item);
            }
        }
        context
    }
}
