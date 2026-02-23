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
    pub fn generate_industrial_report(core: &CognitionCore, query: &str, answer: &str, trace: Option<&cognition::ReasoningTrace>) -> String {
        let mut report = format!("--- INDUSTRIAL REASONING REPORT ---\n");
        report.push_str(&format!("QUERY:  {}\n", query));
        report.push_str(&format!("ANSWER: {}\n", answer));
        report.push_str(&format!("SYSTEM CONFIDENCE: {:.2}%\n", trace.as_ref().map_or(0.0, |t| t.final_confidence * 100.0)));

        if let Some(t) = trace {
            report.push_str("\nREASONING CHAIN:\n");
            for (i, step) in t.steps.iter().enumerate() {
                let arrow = if i == 0 { "" } else { " -> " };
                report.push_str(&format!("{}{}", arrow, step));
            }
            report.push_str("\n");

            // Causal analysis
            let causal_nodes: Vec<_> = t.steps.iter().filter(|s| core.knowledge_graph.facts.values().any(|f| f.triple.as_ref().map_or(false, |tr| tr.subject == **s && tr.predicate == "causes"))).collect();
            if !causal_nodes.is_empty() {
                report.push_str("\nCAUSAL ANCHORS DETECTED:\n");
                for node in causal_nodes {
                    report.push_str(&format!("  - {} (Active Driver)\n", node));
                }
            }
        }

        let entropy = core.compute_global_entropy();
        report.push_str(&format!("\nSYSTEM UNCERTAINTY: {:.4} (Industrial Threshold: 0.80)\n", entropy));
        if entropy > 0.6 {
            report.push_str("ADVISORY: High system entropy detected. Consider reinforcing core facts.\n");
        }

        report.push_str("----------------------------------");
        report
    }

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
