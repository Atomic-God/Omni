use core_vsa::HyperVector;
use cognition::abductive::AbductiveReasoner;
use cognition::sequence::SequenceResonator;
use cognition::{IntentResolver, Intent};
use std::collections::{BinaryHeap, VecDeque};
use std::cmp::Ordering;
use std::time::{SystemTime, UNIX_EPOCH};
use log::{info, warn, debug};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Goal {
    pub id: String,
    pub description: String,
    pub priority: u32,
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

pub struct OODAController {
    pub resonator: SequenceResonator,
    pub reasoner: AbductiveReasoner,
    pub current_observation: Option<HyperVector>,
    pub predicted_observation: Option<HyperVector>,
    pub surprise_metric: f32,
    pub uncertainty_metric: f32,
    pub goal_queue: BinaryHeap<Goal>,
    pub surprise_history: VecDeque<f32>,
    pub energy_budget: f32,
    pub step_count: usize,
    pub current_intent: Intent, // New
}

#[derive(Debug, Clone)]
pub enum Action {
    DoNothing,
    UpdateModel,
    GenerateHypothesis,
    Explore(String),
    Output(String),
    Abort(String),
    ExecuteCommand(String), // New
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
            energy_budget: 100.0,
            step_count: 0,
            current_intent: Intent::Unknown,
        }
    }

    pub fn observe(&mut self, input: HyperVector, text_hint: Option<&str>) {
        self.current_observation = Some(input.clone());
        self.resonator.add(&input);

        // Resolve Intent
        self.current_intent = IntentResolver::resolve(&input, text_hint);
        debug!("Observed Intent: {:?}", self.current_intent);

        self.energy_budget -= 0.1;
    }

    pub fn orient(&mut self) {
        if let (Some(obs), Some(pred)) = (&self.current_observation, &self.predicted_observation) {
            let sim = obs.similarity(pred);
            self.surprise_metric = 1.0 - sim;

            self.surprise_history.push_back(self.surprise_metric);
            if self.surprise_history.len() > 20 {
                self.surprise_history.pop_front();
            }

            let avg_surprise: f32 = self.surprise_history.iter().sum::<f32>() / self.surprise_history.len() as f32;
            let variance: f32 = self.surprise_history.iter().map(|s| (s - avg_surprise).powi(2)).sum::<f32>() / self.surprise_history.len() as f32;
            self.uncertainty_metric = variance.sqrt();
        } else {
            self.surprise_metric = 1.0;
            self.uncertainty_metric = 1.0;
        }

        self.predicted_observation = Some(self.resonator.context_vector.permute(1));
        self.energy_budget -= 0.2;
    }

    pub fn decide(&mut self) -> Action {
        self.step_count += 1;
        if self.energy_budget <= 0.0 {
            return Action::Abort("Energy Depleted".to_string());
        }
        if self.step_count > 10000 {
            return Action::Abort("Max Steps Exceeded".to_string());
        }

        // Intent-driven logic
        match &self.current_intent {
            Intent::Command(cmd) => return Action::ExecuteCommand(cmd.clone()),
            Intent::Query(q) => return Action::Output(format!("Answering: {}", q)), // Stub for reasoning
            _ => {}
        }

        if self.surprise_metric > 0.6 {
            return Action::GenerateHypothesis;
        }

        if let Some(goal) = self.goal_queue.peek() {
            if goal.priority > 50 {
                return Action::Explore(goal.description.clone());
            }
        }

        Action::UpdateModel
    }

    pub fn act(&mut self, action: Action) {
        match action {
            Action::DoNothing => {},
            Action::UpdateModel => {},
            Action::GenerateHypothesis => {
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
                if let Some(mut goal) = self.goal_queue.pop() {
                    goal.priority = goal.priority.saturating_sub(10);
                    if goal.priority > 0 {
                        self.goal_queue.push(goal);
                    }
                }
            },
            Action::Output(msg) => info!("ACT: Outputting - {}", msg),
            Action::ExecuteCommand(cmd) => info!("ACT: Executing - {}", cmd),
            Action::Abort(reason) => warn!("ACT: Aborting - {}", reason),
        }

        self.decay_priorities();
        self.energy_budget -= 0.5;
    }

    pub fn add_goal(&mut self, goal: Goal) {
        self.goal_queue.push(goal);
    }

    fn decay_priorities(&mut self) {
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
