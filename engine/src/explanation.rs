use cognition::CognitionCore;

pub struct ExplanationEngine;

impl ExplanationEngine {
    pub fn simplify(core: &CognitionCore, concept: &str, level: &str) -> String {
        let relations = match core.relation_graph.get(concept) {
            Some(r) => r,
            None => return format!("I don't know enough about {} to explain it.", concept),
        };

        // Simplification Logic:
        // ELI5 (Child): Only use 'is_a' or 'has_property' relations. Avoid 'causal' chains unless simple.
        // Student: Use all relations but summarize.
        // Expert: Show raw relations and confidence weights.

        let mut explanation = String::new();
        match level {
            "child" | "eli5" => {
                explanation.push_str(&format!("Imagine {}. ", concept));
                for rel in relations.iter().take(3) {
                    explanation.push_str(&format!("It is like {} because it relates to {}. ", concept, rel.target));
                }
            },
            "expert" => {
                explanation.push_str(&format!("Structural Analysis of {}:\n", concept));
                for rel in relations {
                    explanation.push_str(&format!("- [{:?}] -> {} (w={})\n", rel.relation_type, rel.target, rel.weight));
                }
            },
            _ => { // Student/Default
                explanation.push_str(&format!("{} is defined by: ", concept));
                let targets: Vec<String> = relations.iter().take(5).map(|r| r.target.clone()).collect();
                explanation.push_str(&targets.join(", "));
            }
        }
        explanation
    }
}
