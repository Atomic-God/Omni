use serde::{Serialize, Deserialize};
use std::collections::VecDeque;
use tracing::{info, warn};
use rand;

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
    pub retries: u32,
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
            retries: 0,
        };
        info!("Task Loop: New goal added: {}", description);
        self.goals.push_back(goal);
    }

    /// Executes one step of the current top-level goal.
    pub fn step(&mut self) -> Option<String> {
        let goal = self.goals.front_mut()?;
        goal.status = TaskStatus::InProgress;

        info!("Task Loop: Executing step for goal: {}", goal.description);

        // Industrial failure simulation (e.g. random failure for testing recovery)
        let mut rng = rand::thread_rng();
        if rand::Rng::gen_bool(&mut rng, 0.05) {
             warn!("Task Loop: Transient failure detected for goal: {}", goal.description);
             if goal.retries < 3 {
                 goal.retries += 1;
                 goal.status = TaskStatus::Pending; // Backtrack to retry
                 return Some(format!("Recovering: Retrying goal (attempt {})", goal.retries));
             } else {
                 goal.status = TaskStatus::Failed("Max retries exceeded".to_string());
                 let failed = self.goals.pop_front().unwrap();
                 self.history.push(failed);
                 return Some("Goal Failed after multiple attempts".to_string());
             }
        }

        // Multi-step progress simulation
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
