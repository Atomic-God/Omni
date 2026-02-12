use core_vsa::{HyperVector, DIMENSION};
use cognition::abductive::AbductiveReasoner;
use cognition::sequence::SequenceResonator;
use std::collections::{BinaryHeap, VecDeque};
use std::cmp::Ordering;
use std::time::{SystemTime, UNIX_EPOCH};
use log::{info, warn, debug};

/// Represents a Goal or Task for the Active Inference Engine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Goal {
    pub id: String,
    pub description: String,
    pub priority: u32, // Higher is better
    pub created_at: u64,
    pub deadline: u64,
}

impl Ord for Goal {
    fn cmp(&self, other: &Self) -> Ordering {
        self.priority.cmp(&other.priority)
    }
}

impl PartialOrd for Goal {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// The OODA Loop Controller for Active Inference.
///
/// Cycle:
/// 1. Observe: Ingest new data (HyperVector).
/// 2. Orient: Compare Observation with Prediction (Surprise). Update World Model.
/// 3. Decide: Select Goal/Action to minimize future surprise or maximize utility.
/// 4. Act: Execute Action (Internal or External).
pub struct OODAController {
    // Cognitive Components
    pub resonator: SequenceResonator,
    pub reasoner: AbductiveReasoner,

    // State
    pub current_observation: Option<HyperVector>,
    pub predicted_observation: Option<HyperVector>,
    pub surprise_metric: f32,
    pub uncertainty_metric: f32,

    // Goals
    pub goal_queue: BinaryHeap<Goal>,

    // History
    pub surprise_history: VecDeque<f32>,
}

#[derive(Debug, Clone)]
pub enum Action {
    DoNothing,
    UpdateModel,
    GenerateHypothesis,
    Explore(String), // Search for specific concept
    Output(String),
}

impl OODAController {
    pub fn new() -> Self {
        Self {
            resonator: SequenceResonator::new(10),
            reasoner: AbductiveReasoner::new(0.5),
            current_observation: None,
            predicted_observation: None,
            surprise_metric: 0.0,
            uncertainty_metric: 0.0,
            goal_queue: BinaryHeap::new(),
            surprise_history: VecDeque::new(),
        }
    }

    /// 1. OBSERVE
    /// Ingests a new perception vector.
    pub fn observe(&mut self, input: HyperVector) {
        self.current_observation = Some(input.clone());
        // Auto-add to resonator to update context
        self.resonator.add(&input);
    }

    /// 2. ORIENT
    /// Calculates Surprise and Uncertainty.
    /// Updates internal metrics.
    pub fn orient(&mut self) {
        if let (Some(obs), Some(pred)) = (&self.current_observation, &self.predicted_observation) {
            // Surprise = 1 - Similarity (Cosine distance approximation)
            // Sim ranges -1 to 1.
            // We map it to 0 (perfect match) to 1 (complete mismatch/orthogonal).
            // Actually, Sim can be negative (opposite).
            // Distance = (1 - Sim) / 2? Or just 1 - Sim?
            // Let's use 1 - Sim. If Sim is 1, Surprise 0. If Sim 0, Surprise 1. If Sim -1, Surprise 2!
            let sim = obs.similarity(pred);
            self.surprise_metric = 1.0 - sim;

            // Uncertainty: Variance of recent surprise?
            // Or "Entropy" of the prediction bundle? (Binary VSA doesn't have easy entropy).
            // We'll use moving average of absolute surprise.
            self.surprise_history.push_back(self.surprise_metric);
            if self.surprise_history.len() > 20 {
                self.surprise_history.pop_front();
            }

            let avg_surprise: f32 = self.surprise_history.iter().sum::<f32>() / self.surprise_history.len() as f32;
            let variance: f32 = self.surprise_history.iter().map(|s| (s - avg_surprise).powi(2)).sum::<f32>() / self.surprise_history.len() as f32;
            self.uncertainty_metric = variance.sqrt();

            debug!("ORIENT: Surprise {:.3}, Uncertainty {:.3}", self.surprise_metric, self.uncertainty_metric);
        } else {
            // First run, no prediction
            self.surprise_metric = 1.0;
            self.uncertainty_metric = 1.0;
        }

        // Generate Next Prediction (Naive: Expect persistence or sequence continuation)
        // P_{t+1} = Resonator.predict_next() (Not implemented, let's use current context)
        // For now, predict "Change" or "Same"?
        // Let's assume prediction is "Context shifted forward".
        // P_{t+1} = Permute(CurrentContext)
        self.predicted_observation = Some(self.resonator.context_vector.permute(1));
    }

    /// 3. DECIDE
    /// Prioritizes goals based on Surprise and Uncertainty.
    /// Returns the selected Action.
    pub fn decide(&mut self) -> Action {
        // High Surprise -> Need to Update Model or Explore
        if self.surprise_metric > 0.6 {
            debug!("DECIDE: High Surprise! Triggering Hypothesis Generation.");
            return Action::GenerateHypothesis;
        }

        // Check Goal Queue
        if let Some(goal) = self.goal_queue.peek() {
            // Prioritize existing goal if high priority
            if goal.priority > 50 {
                return Action::Explore(goal.description.clone());
            }
        }

        // Default: Update Model (Learn)
        Action::UpdateModel
    }

    /// 4. ACT
    /// Executes the decided action.
    pub fn act(&mut self, action: Action) {
        match action {
            Action::DoNothing => {},
            Action::UpdateModel => {
                // Learning is implicit in "Observe" (Resonator update).
                // But we could "Reinforce" the memory here if surprise was low (Validation).
                if self.surprise_metric < 0.2 {
                    // "Reinforcement": Bundle again to strengthen?
                    // In Binary VSA, repeated bundling doesn't add weight unless we have integer counters.
                    // Pass.
                }
            },
            Action::GenerateHypothesis => {
                // Abductive reasoning: What caused this?
                // Hypothesis = Reasoner.induce(Observation, Rules)
                // For now, we just create a goal to "Investigate".
                self.add_goal(Goal {
                    id: format!("investigate_{}", self.get_time()),
                    description: "Resolve High Surprise".to_string(),
                    priority: 80,
                    created_at: self.get_time(),
                    deadline: self.get_time() + 100,
                });
            },
            Action::Explore(desc) => {
                info!("ACT: Exploring - {}", desc);
                // Reduce priority of the goal we just worked on
                if let Some(mut goal) = self.goal_queue.pop() {
                    goal.priority = goal.priority.saturating_sub(10);
                    if goal.priority > 0 {
                        self.goal_queue.push(goal);
                    }
                }
            },
            Action::Output(msg) => {
                info!("ACT: Outputting - {}", msg);
            }
        }

        // Decay all goals
        self.decay_priorities();
    }

    pub fn add_goal(&mut self, goal: Goal) {
        self.goal_queue.push(goal);
    }

    fn decay_priorities(&mut self) {
        // Rust's BinaryHeap doesn't support in-place mutation easily.
        // We pop all, decay, push back. (Inefficient for large queues, but industrial robust).
        let mut temp = Vec::new();
        while let Some(mut g) = self.goal_queue.pop() {
            g.priority = g.priority.saturating_sub(1);
            if g.priority > 0 {
                temp.push(g);
            }
        }
        for g in temp {
            self.goal_queue.push(g);
        }
    }

    fn get_time(&self) -> u64 {
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ooda_loop_basics() {
        let mut controller = OODAController::new();

        let obs1 = HyperVector::random();

        // 1. Observe
        controller.observe(obs1.clone());
        assert!(controller.current_observation.is_some());

        // 2. Orient
        controller.orient();
        // First run: Surprise should be 1.0 (no prediction)
        assert_eq!(controller.surprise_metric, 1.0);
        assert!(controller.predicted_observation.is_some());

        // 3. Decide
        let action = controller.decide();
        // Surprise > 0.6 -> GenerateHypothesis
        match action {
            Action::GenerateHypothesis => {},
            _ => panic!("Expected GenerateHypothesis on high surprise"),
        }

        // 4. Act
        controller.act(action);
        // Should have added a goal
        assert!(!controller.goal_queue.is_empty());
    }

    #[test]
    fn test_prediction_reduction() {
        let mut controller = OODAController::new();
        let obs1 = HyperVector::random();

        // Teach the resonator
        controller.observe(obs1.clone());
        controller.orient();

        // Pred = Permute(obs1)
        // If next obs is Permute(obs1), surprise should be 0.

        let obs2 = obs1.permute(1);
        controller.observe(obs2);
        controller.orient();

        assert!(controller.surprise_metric < 0.1, "Surprise should be low for correctly predicted sequence. Got {}", controller.surprise_metric);
    }
}
