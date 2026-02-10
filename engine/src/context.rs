use cognition::planning::Goal;
use std::collections::{VecDeque, HashMap};
use crate::governance::{SalienceScoring, MemoryTier};
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
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

// Arbitration Layers

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EpisodicMemory {
    pub buffer: MemoryTier,
    // Future: Could store full Episode objects with timestamps
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GoalMemory {
    pub active: VecDeque<Goal>,
    pub completed: Vec<Goal>, // History
}

impl GoalMemory {
    pub fn new() -> Self {
        Self { active: VecDeque::new(), completed: Vec::new() }
    }

    pub fn push(&mut self, goal: Goal) {
        // Priority sort? For now, just push.
        self.active.push_front(goal);
    }

    pub fn complete(&mut self, target_state: &str) {
        // Move to completed
        let mut still_active = VecDeque::new();
        while let Some(mut g) = self.active.pop_front() {
            if g.target_state == target_state {
                g.completed = true;
                self.completed.push(g);
            } else {
                still_active.push_back(g);
            }
        }
        self.active = still_active;
    }

    pub fn compact(&mut self) {
        // Remove low priority completed goals
        self.completed.retain(|g| g.priority > 10);
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContextManager {
    // Arbitrated Memory Stores
    pub episodic: EpisodicMemory,
    pub goals: GoalMemory,
    pub structural_focus: MemoryTier, // Pointers to knowledge graph

    // Governance
    pub salience: SalienceScoring,
    pub topic_tracker: TopicTracker,
}

impl ContextManager {
    pub fn new() -> Self {
        Self {
            episodic: EpisodicMemory { buffer: MemoryTier::new(100) },
            goals: GoalMemory::new(),
            structural_focus: MemoryTier::new(20), // Focus on 20 concepts max
            salience: SalienceScoring::new(0.05),
            topic_tracker: TopicTracker::new(),
        }
    }

    pub fn push_goal(&mut self, goal: Goal) {
        self.goals.push(goal);
    }

    pub fn complete_goal(&mut self, target_state: &str) {
        self.goals.complete(target_state);
    }

    pub fn activate_semantic(&mut self, concept: &str) {
        self.salience.score(concept, 1.0);

        // Add to structural focus
        self.structural_focus.add(concept.to_string());

        self.topic_tracker.track(concept);
    }

    pub fn log_episodic(&mut self, text: &str) {
        self.episodic.buffer.add(text.to_string());
    }

    pub fn get_active_context(&self) -> Vec<String> {
        // Combine Focus + Top Salience + Active Goals
        let mut context = Vec::new();

        // 1. High Salience (Structural)
        let top_concepts = self.salience.get_top(5);
        context.extend(top_concepts);

        // 2. Recent Focus
        for item in &self.structural_focus.items {
            if !context.contains(item) {
                context.push(item.clone());
            }
        }

        // 3. Goals
        for goal in &self.goals.active {
            let desc = format!("Goal: {}", goal.description);
            if !context.contains(&desc) {
                context.push(desc);
            }
        }

        context
    }

    pub fn compact(&mut self) {
        self.goals.compact();
        // Decay salience
        self.salience.update_decay();
    }
}
