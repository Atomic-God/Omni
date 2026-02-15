use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use log::{info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactTriple {
    pub subject: String,
    pub predicate: String,
    pub object: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeFact {
    pub id: String,
    pub triple: Option<FactTriple>,
    pub confidence: f32,
    pub source_reliability: f32,
    pub reinforcement_count: u32,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

    pub fn add_fact(&mut self, id: &str, triple: Option<FactTriple>, reliability: f32) {
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();

        if let Some(ref t) = triple {
            let conflicting: Vec<String> = self.facts.values()
                .filter(|f| f.triple.as_ref().map_or(false, |ft|
                    ft.subject == t.subject && ft.predicate == t.predicate && ft.object != t.object
                ))
                .map(|f| f.id.clone())
                .collect();

            for conflict_id in conflicting {
                warn!("Conflict detected during ingestion: {} contradicts existing {}", id, conflict_id);
                self.contradictions.push((id.to_string(), conflict_id));
            }
        }

        let fact = self.facts.entry(id.to_string()).or_insert(KnowledgeFact {
            id: id.to_string(),
            triple,
            confidence: 0.5,
            source_reliability: reliability,
            reinforcement_count: 0,
            timestamp: now,
        });

        let prior = fact.confidence;
        let n = fact.reinforcement_count as f32;
        fact.confidence = (prior * n + reliability) / (n + 1.0);
        fact.reinforcement_count += 1;
        fact.timestamp = now;
    }

    pub fn reinforce(&mut self, id: &str, evidence_reliability: f32) {
        if let Some(fact) = self.facts.get_mut(id) {
            let prior = fact.confidence;
            let n = fact.reinforcement_count as f32;
            fact.confidence = (prior * n + evidence_reliability) / (n + 1.0);
            fact.reinforcement_count += 1;
            fact.timestamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
            info!("Fact {} reinforced to confidence {:.2}", id, fact.confidence);
        }
    }

    pub fn apply_temporal_decay(&mut self, decay_rate: f32, now: u64) {
        for fact in self.facts.values_mut() {
            let age = now.saturating_sub(fact.timestamp);
            if age > 3600 {
                let periods = (age / 3600) as f32;
                fact.confidence *= decay_rate.powf(periods);
            }
        }
    }

    pub fn resolve_all_contradictions(&mut self) {
        let conflicts = self.contradictions.clone();
        for (a_id, b_id) in conflicts {
            let conf_a = self.facts.get(&a_id).map(|f| f.confidence).unwrap_or(0.0);
            let conf_b = self.facts.get(&b_id).map(|f| f.confidence).unwrap_or(0.0);

            if conf_a > conf_b + 0.1 {
                info!("Resolving conflict: Favoring {} over {}", a_id, b_id);
                if let Some(f) = self.facts.get_mut(&b_id) { f.confidence *= 0.5; }
            } else if conf_b > conf_a + 0.1 {
                info!("Resolving conflict: Favoring {} over {}", b_id, a_id);
                if let Some(f) = self.facts.get_mut(&a_id) { f.confidence *= 0.5; }
            }
        }
    }

    pub fn get_uncertainty(&self, id: &str) -> f32 {
        self.facts.get(id).map(|f| 1.0 - f.confidence).unwrap_or(1.0)
    }
}
