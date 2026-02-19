// Active Inference Components (Structural)

use crate::CognitionCore;

pub struct SurpriseEstimator;

impl SurpriseEstimator {
    pub fn estimate_entropy(core: &CognitionCore) -> f32 {
        core.compute_global_entropy()
    }
}

pub struct GoalGenerator;

impl GoalGenerator {
    pub fn suggest_goals(core: &CognitionCore) -> Vec<String> {
        // Suggest exploring low-confidence areas
        let mut goals = Vec::new();
        if core.relation_graph.len() < 5 {
            goals.push("Expand Knowledge Graph".to_string());
        }
        goals
    }
}

pub struct ExplorationPolicy;

impl ExplorationPolicy {
    pub fn next_move() -> String {
        "explore_random".to_string()
    }
}
