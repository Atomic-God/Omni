use cognition::CognitionCore;

pub enum AudienceModel {
    Child,
    Student,
    Expert,
}

pub struct TraceGenerator;

impl TraceGenerator {
    /// Generates a detailed audit log of the reasoning steps taken.
    pub fn generate_audit_trace(core: &cognition::CognitionCore, start: &str, end: &str) -> String {
        let mut trace = format!("Industrial Audit Trace: {} -> {}\n", start, end);
        if let Some(path) = core.find_path(start, end, 5) {
            for (i, step) in path.steps.iter().enumerate() {
                trace.push_str(&format!("  Step {}: {}\n", i, step));
            }
            trace.push_str(&format!("  Final Confidence: {:.4}\n", path.final_confidence));
            trace.push_str(&format!("  Entropy: {:.4}\n", core.compute_global_entropy()));
        } else {
            trace.push_str("  No logical path found between concepts.\n");
        }
        trace
    }
}

pub struct ExplanationEngine;

impl ExplanationEngine {
    pub fn simplify(core: &CognitionCore, concept: &str, audience: AudienceModel) -> String {
        let relations = match core.relation_graph.get(concept) {
            Some(r) => r,
            None => return format!("I don't know enough about {} to explain it.", concept),
        };

        match audience {
            AudienceModel::Child => {
                let mut explanation = format!("Imagine {}. ", concept);
                for rel in relations.iter().filter(|r| r.weight > 60.0).take(3) {
                    explanation.push_str(&format!("It is like {} because it relates to {}. ", concept, rel.target));
                }
                explanation
            },
            AudienceModel::Student => {
                let mut explanation = format!("{} is defined by: ", concept);
                let targets: Vec<String> = relations.iter().take(5).map(|r| r.target.clone()).collect();
                explanation.push_str(&targets.join(", "));
                explanation
            },
            AudienceModel::Expert => {
                let mut explanation = format!("Structural Analysis of {} (Uncertainty: {:.2}):\n", concept, core.compute_global_entropy());
                for rel in relations {
                    explanation.push_str(&format!("- [{:?}] -> {} (w={:.2}, c={:.2})\n", rel.relation_type, rel.target, rel.weight, rel.confidence));
                }

                let contradictions = core.detect_contradictions(concept);
                if !contradictions.is_empty() {
                    explanation.push_str("\nWARNING: Contradictions detected with: ");
                    explanation.push_str(&contradictions.join(", "));
                }

                explanation
            },
        }
    }
}
