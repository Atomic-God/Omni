use crate::CognitionCore;
use tracing::warn;

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
    /// Deep validation of an inference path against global knowledge consistency.
    pub fn validate_inference(core: &CognitionCore, subject: &str, object: &str) -> bool {
        // 1. Direct contradiction check
        let contradictions = core.detect_contradictions(subject);
        if contradictions.contains(&object.to_string()) {
            warn!("Inference Blocked: Direct contradiction found for {}", subject);
            return false;
        }

        // 2. Transitive contradiction check
        if let Some(path) = core.find_path(subject, object, 4) {
             for step in &path.steps {
                 let local_contradictions = core.detect_contradictions(step);
                 if local_contradictions.iter().any(|c| path.steps.contains(c)) {
                     warn!("Inference Blocked: Indirect contradiction found in path step {}", step);
                     return false;
                 }
             }
        }

        // 3. Confidence Thresholding
        if let Some(path) = core.find_path(subject, object, 4) {
            if path.final_confidence < 0.2 {
                warn!("Inference Blocked: Confidence too low ({:.4})", path.final_confidence);
                return false;
            }
        }

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
