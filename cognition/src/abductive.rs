use crate::knowledge::{KnowledgeGraph, KnowledgeFact};
use tracing::info;

pub struct AbductiveExplanation {
    pub hypothesis: String,
    pub confidence: f32,
    pub evidence_links: Vec<String>,
}

#[derive(Default)]
pub struct AbductiveReasoner;

impl AbductiveReasoner {
    pub fn new(_threshold: f32) -> Self {
        Self {}
    }

    pub fn explain(&self, graph: &KnowledgeGraph, observation: &str) -> Option<AbductiveExplanation> {
        info!("Abduction: Seeking explanation for '{}'", observation);

        let relevant: Vec<&KnowledgeFact> = graph.facts.values()
            .filter(|f| {
                if let Some(ref t) = f.triple {
                    t.subject.contains(observation) || t.object.contains(observation)
                } else {
                    false
                }
            })
            .collect();

        if relevant.is_empty() {
            return None;
        }

        for fact in &relevant {
            if let Some(ref t) = fact.triple {
                let is_causal = ["causality", "causes", "triggers", "leads to"].contains(&t.predicate.as_str());
                if is_causal {
                    if t.object.contains(observation) {
                        return Some(AbductiveExplanation {
                            hypothesis: t.subject.clone(),
                            confidence: fact.confidence * 0.9,
                            evidence_links: vec![fact.id.clone()],
                        });
                    }
                }
            }
        }

        if let Some(fact) = relevant.first() {
             if let Some(ref t) = fact.triple {
                 return Some(AbductiveExplanation {
                     hypothesis: format!("Related to {}", t.subject),
                     confidence: fact.confidence * 0.5,
                     evidence_links: vec![fact.id.clone()],
                 });
             }
        }

        None
    }

    pub fn find_analogy(&self, graph: &KnowledgeGraph, a: &str, b: &str) -> f32 {
        let preds_a: std::collections::HashSet<_> = graph.facts.values()
            .filter(|f| f.triple.as_ref().map_or(false, |t| t.subject == a))
            .filter_map(|f| f.triple.as_ref().map(|t| t.predicate.clone()))
            .collect();

        let preds_b: std::collections::HashSet<_> = graph.facts.values()
            .filter(|f| f.triple.as_ref().map_or(false, |t| t.subject == b))
            .filter_map(|f| f.triple.as_ref().map(|t| t.predicate.clone()))
            .collect();

        if preds_a.is_empty() || preds_b.is_empty() { return 0.0; }

        let intersection = preds_a.intersection(&preds_b).count();
        let union = preds_a.union(&preds_b).count();

        intersection as f32 / union as f32
    }
}
