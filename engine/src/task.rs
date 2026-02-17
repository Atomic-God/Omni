use serde::{Serialize, Deserialize};
use std::collections::VecDeque;
use tracing::{info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    InProgress,
    Completed,
    Failed(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Goal {
    pub id: String,
    pub description: String,
    pub status: TaskStatus,
    pub confidence: f32, // Step 97
}

pub struct TaskLoop {
    pub goals: VecDeque<Goal>,
    pub history: Vec<Goal>,
}

impl TaskLoop {
    pub fn new() -> Self {
        Self {
            goals: VecDeque::new(),
            history: Vec::new(),
        }
    }

    pub fn add_goal(&mut self, description: &str) {
        let goal = Goal {
            id: uuid::Uuid::new_v4().to_string(),
            description: description.to_string(),
            status: TaskStatus::Pending,
            confidence: 0.0,
        };
        info!("Task Loop: New goal added: {}", description);
        self.goals.push_back(goal);
    }

    /// Executes one step of the current top-level goal.
    pub fn step(&mut self) -> Option<String> {
        let goal = self.goals.front_mut()?;
        goal.status = TaskStatus::InProgress;

        info!("Task Loop: Executing step for goal: {}", goal.description);

        // Multi-step simulation for Phase-1
        // In reality, this would hook into cognition/planning
        if goal.confidence < 0.9 {
            goal.confidence += 0.3;
            return Some(format!("Working on: {}", goal.description));
        }

        goal.status = TaskStatus::Completed;
        let finished = self.goals.pop_front().unwrap();
        self.history.push(finished);
        Some("Goal Completed".to_string())
    }
}
