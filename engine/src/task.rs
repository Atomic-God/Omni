use serde::{Serialize, Deserialize};
use std::collections::VecDeque;
use tracing::{info, warn};
use rand;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskStatus {
    Pending,
    InProgress,
    Completed,
    Failed(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryStrategy {
    pub max_retries: u32,
    pub backoff_factor: f32,
}

impl Default for RetryStrategy {
    fn default() -> Self {
        Self { max_retries: 3, backoff_factor: 2.0 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Goal {
    pub id: String,
    pub description: String,
    pub status: TaskStatus,
    pub priority: Priority,
    pub confidence: f32,
    pub retries: u32,
    pub strategy: RetryStrategy,
    pub created_at: u64,
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

    pub fn add_goal(&mut self, description: &str, priority: Priority) {
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
        let goal = Goal {
            id: uuid::Uuid::new_v4().to_string(),
            description: description.to_string(),
            status: TaskStatus::Pending,
            priority,
            confidence: 0.0,
            retries: 0,
            strategy: RetryStrategy::default(),
            created_at: now,
        };
        info!("Task Loop: New goal added [{:?}]: {}", goal.priority, description);
        self.goals.push_back(goal);

        // Sort goals by priority
        let mut sorted_goals: Vec<Goal> = self.goals.drain(..).collect();
        sorted_goals.sort_by(|a, b| {
            let p_a = match a.priority { Priority::Critical => 4, Priority::High => 3, Priority::Medium => 2, Priority::Low => 1 };
            let p_b = match b.priority { Priority::Critical => 4, Priority::High => 3, Priority::Medium => 2, Priority::Low => 1 };
            p_b.cmp(&p_a) // Higher priority first
        });
        for g in sorted_goals { self.goals.push_back(g); }
    }

    /// Executes one step of the current top-level goal with industrial recovery logic.
    /// Incorporates uncertainty to decide between Observation and Action.
    pub fn step(&mut self, current_uncertainty: f32) -> Option<String> {
        // Confidence Scoring Loop: Evaluate if current knowledge supports goal completion
        if let Some(goal) = self.goals.front_mut() {
            Self::update_goal_confidence(goal, current_uncertainty);
        }

        let goal = self.goals.front_mut()?;

        if current_uncertainty > 0.7 && goal.status != TaskStatus::Pending {
             info!("Task Loop: HIGH UNCERTAINTY ({:.2}). Switching to Observation Mode.", current_uncertainty);
             return Some(format!("Observing: Gathering more data for goal: {}", goal.description));
        }

        if goal.confidence < 0.1 && goal.status == TaskStatus::InProgress {
            warn!("Task Loop: Goal confidence CRITICALLY LOW ({:.2}). Re-evaluating strategy.", goal.confidence);
            return self.handle_failure();
        }

        // Handle Backoff if retrying
        if goal.retries > 0 && goal.status == TaskStatus::Pending {
            let delay = (goal.strategy.backoff_factor.powi(goal.retries as i32) * 1.0) as u64;
            info!("Task Loop: Backoff active for goal: {} (delay units: {})", goal.description, delay);
            // In a real async loop, we would sleep. Here we just log and proceed.
        }

        goal.status = TaskStatus::InProgress;
        info!("Task Loop: [PRIORITY: {:?}] Executing step for goal: {}", goal.priority, goal.description);

        // Industrial failure simulation
        let mut rng = rand::thread_rng();
        let failure_probability = 0.05;

        if rand::Rng::gen_bool(&mut rng, failure_probability) {
             warn!("Task Loop: INDUSTRIAL FAILURE detected: {}", goal.description);
             return self.handle_failure();
        }

        // Execution logic: Increase confidence as we perform "symbolic actions"
        let progress = rand::Rng::gen_range(&mut rng, 0.1..0.4);
        goal.confidence += progress;

        if goal.confidence < 1.0 {
            Some(format!("Progress: {:.0}% on goal: {}", goal.confidence * 100.0, goal.description))
        } else {
            goal.status = TaskStatus::Completed;
            info!("Task Loop: Goal COMPLETED successfully: {}", goal.description);
            let finished = self.goals.pop_front().unwrap();
            self.history.push(finished);
            Some("Goal reached Industrial Completion.".to_string())
        }
    }

    fn update_goal_confidence(goal: &mut Goal, uncertainty: f32) {
        // Industrial Confidence Scoring Loop:
        // C_new = C_old * (1 - Uncertainty) + (SuccessRate * 0.1)
        let decay = 1.0 - uncertainty;
        goal.confidence = (goal.confidence * decay).clamp(0.0, 1.0);

        // Reinforce if uncertainty is low
        if uncertainty < 0.3 {
            goal.confidence = (goal.confidence + 0.05).min(1.0);
        }
    }

    fn handle_failure(&mut self) -> Option<String> {
        let goal = self.goals.front_mut()?;
        if goal.retries < goal.strategy.max_retries {
            goal.retries += 1;
            goal.status = TaskStatus::Pending;

            // Industrial Recovery: Pivot strategy based on retry count
            if goal.retries == 1 {
                info!("Recovery Logic [Tier 1]: Increasing observation priority.");
                goal.confidence = 0.2;
                return Some(format!("Recovery: Strategy shift to High Observation for: {}", goal.description));
            } else if goal.retries == 2 {
                info!("Recovery Logic [Tier 2]: Broadening semantic search.");
                goal.confidence = 0.1;
                return Some(format!("Recovery: Strategy shift to Broad Semantic Search for: {}", goal.description));
            }

            goal.confidence *= 0.5; // Backtrack
            Some(format!("Recovery Loop: Initiated Retry {}/{} for: {}", goal.retries, goal.strategy.max_retries, goal.description))
        } else {
            info!("Task Loop: UNRECOVERABLE FAILURE for goal: {}", goal.description);
            goal.status = TaskStatus::Failed("Industrial Timeout / Max Retries Exceeded".to_string());
            let failed = self.goals.pop_front().unwrap();
            self.history.push(failed);
            Some("Task aborted: Max retries exceeded.".to_string())
        }
    }
}
