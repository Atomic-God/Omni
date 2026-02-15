use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use log::{info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeFact {
    pub id: String,
    pub confidence: f32,
    pub source_reliability: f32,
    pub reinforcement_count: u32,
    pub timestamp: u64,
}

pub struct KnowledgeGraph {
    pub facts: HashMap<String, KnowledgeFact>,
    pub contradictions: Vec<(String, String)>,
}

impl KnowledgeGraph {
    pub fn new() -> Self {
        Self {
            facts: HashMap::new(),
            contradictions: Vec::new(),
        }
    }

    pub fn reinforce(&mut self, id: &str, evidence_reliability: f32) {
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
        let fact = self.facts.entry(id.to_string()).or_insert(KnowledgeFact {
            id: id.to_string(),
            confidence: 0.5, // Initial prior
            source_reliability: evidence_reliability,
            reinforcement_count: 0,
            timestamp: now,
        });

        // Bayesian-inspired update: P(H|E) = P(E|H)P(H) / P(E)
        // Simplified: confidence = (prior * count + evidence) / (count + 1)
        let prior = fact.confidence;
        let n = fact.reinforcement_count as f32;

        fact.confidence = (prior * n + evidence_reliability) / (n + 1.0);
        fact.reinforcement_count += 1;
        fact.timestamp = now;

        info!("Fact {} reinforced: confidence={:.2}, count={}", id, fact.confidence, fact.reinforcement_count);
    }

    pub fn apply_temporal_decay(&mut self, decay_rate: f32, now: u64) {
        for fact in self.facts.values_mut() {
            let age = now.saturating_sub(fact.timestamp);
            if age > 3600 { // Decay every hour of inactivity
                let periods = (age / 3600) as f32;
                fact.confidence *= decay_rate.powf(periods);
            }
        }
    }

    pub fn detect_contradiction(&mut self, a_id: &str, b_id: &str, is_contradictory: bool) {
        if is_contradictory {
            warn!("Contradiction detected between {} and {}", a_id, b_id);
            self.contradictions.push((a_id.to_string(), b_id.to_string()));
            self.resolve_contradiction(a_id, b_id);
        }
    }

    fn resolve_contradiction(&mut self, a_id: &str, b_id: &str) {
        let conf_a = self.facts.get(a_id).map(|f| f.confidence).unwrap_or(0.0);
        let conf_b = self.facts.get(b_id).map(|f| f.confidence).unwrap_or(0.0);

        if conf_a > conf_b {
            info!("Resolving conflict: Favoring {} (conf={:.2}) over {} (conf={:.2})", a_id, conf_a, b_id, conf_b);
            if let Some(fact) = self.facts.get_mut(b_id) {
                fact.confidence *= 0.1; // Aggressive penalization for contradicted fact
            }
        } else {
            info!("Resolving conflict: Favoring {} (conf={:.2}) over {} (conf={:.2})", b_id, conf_b, a_id, conf_a);
            if let Some(fact) = self.facts.get_mut(a_id) {
                fact.confidence *= 0.1;
            }
        }
    }
}
