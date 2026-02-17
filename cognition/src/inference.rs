use crate::CognitionCore;
use tracing::debug;

pub struct UncertaintyScorer;

impl UncertaintyScorer {
    pub fn estimate_entropy(core: &CognitionCore) -> f32 {
        core.compute_global_entropy()
    }

    /// Combines multiple uncertainty sources: VSA similarity variance, KG entropy, and source reliability.
    pub fn calculate_industrial_uncertainty(confidence: f32, similarity: f32, kg_entropy: f32) -> f32 {
        // High uncertainty if confidence is low OR similarity is ambiguous OR KG is chaotic
        let sim_uncertainty = 1.0 - (similarity - 0.5).abs() * 2.0;
        let conf_uncertainty = 1.0 - confidence;

        // Weighted average
        sim_uncertainty * 0.4 + conf_uncertainty * 0.4 + kg_entropy * 0.2
    }
}

pub struct ReasoningValidator;

impl ReasoningValidator {
    pub fn validate_inference(core: &CognitionCore, subject: &str, object: &str) -> bool {
        let contradictions = core.detect_contradictions(subject);
        if contradictions.contains(&object.to_string()) {
            return false;
        }

        // Counter-reasoning loop stub
        // Try to prove the opposite: does subject have a relationship that precludes object?
        debug!("Self-verification: Validating path {} -> {}", subject, object);

        true
    }
}

pub struct LogicGate;

impl LogicGate {
    /// Only allows facts to pass if uncertainty is below threshold.
    pub fn threshold_gate(uncertainty: f32, threshold: f32) -> bool {
        uncertainty < threshold
    }
}
