use crate::CognitionCore;

pub struct UncertaintyScorer;

impl UncertaintyScorer {
    pub fn estimate_entropy(core: &CognitionCore) -> f32 {
        core.compute_global_entropy()
    }

    pub fn calculate_uncertainty(confidence: f32, similarity: f32) -> f32 {
        // High uncertainty if confidence is low OR similarity is mid-range (ambiguous)
        let sim_uncertainty = 1.0 - (similarity - 0.5).abs() * 2.0;
        let conf_uncertainty = 1.0 - confidence;
        (sim_uncertainty + conf_uncertainty) / 2.0
    }
}

pub struct ReasoningValidator;

impl ReasoningValidator {
    pub fn validate_inference(core: &CognitionCore, subject: &str, object: &str) -> bool {
        // Check if there is a known contradiction between subject and object
        let contradictions = core.detect_contradictions(subject);
        if contradictions.contains(&object.to_string()) {
            return false;
        }
        true
    }
}

pub struct GoalGenerator;

impl GoalGenerator {
    pub fn suggest_goals(core: &CognitionCore) -> Vec<String> {
        let mut goals = Vec::new();
        if core.relation_graph.len() < 5 {
            goals.push("Expand Knowledge Graph".to_string());
        }
        goals
    }
}
