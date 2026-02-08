use cognition::CognitionCore;

pub enum AudienceModel {
    Child,
    Student,
    Expert,
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
                // ELI5: Use analogies ("like X") and simple attributes
                let mut explanation = format!("Imagine {}. ", concept);
                for rel in relations.iter().filter(|r| r.weight > 60).take(3) {
                    explanation.push_str(&format!("It is like {} because it relates to {}. ", concept, rel.target));
                }
                explanation
            },
            AudienceModel::Student => {
                // Structural definition
                let mut explanation = format!("{} is defined by: ", concept);
                let targets: Vec<String> = relations.iter().take(5).map(|r| r.target.clone()).collect();
                explanation.push_str(&targets.join(", "));
                explanation
            },
            AudienceModel::Expert => {
                // Full graph dump with confidence
                let mut explanation = format!("Structural Analysis of {}:\n", concept);
                for rel in relations {
                    explanation.push_str(&format!("- [{:?}] -> {} (w={})\n", rel.relation_type, rel.target, rel.weight));
                }
                explanation
            },
        }
    }
}
