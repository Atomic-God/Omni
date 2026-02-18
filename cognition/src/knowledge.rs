use std::collections::{HashMap, VecDeque};
use serde::{Serialize, Deserialize};
use tracing::{info, warn, debug};
use core_vsa::FactTriple;

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
    pub subject_index: HashMap<String, Vec<String>>,
    pub source_trust: HashMap<String, f32>, // Step 34
    pub exclusive_predicates: Vec<String>,
}

impl KnowledgeGraph {
    pub fn new() -> Self {
        Self {
            facts: HashMap::new(),
            contradictions: Vec::new(),
            subject_index: HashMap::new(),
            source_trust: HashMap::new(),
            exclusive_predicates: vec!["taxonomy".to_string(), "is_a".to_string(), "is_at".to_string(), "color".to_string()],
        }
    }

    pub fn add_fact(&mut self, id: &str, triple: Option<FactTriple>, reliability: f32) {
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();

        if let Some(ref t) = triple {
            let is_exclusive = self.exclusive_predicates.contains(&t.predicate);
            let conflicting: Vec<String> = self.facts.values()
                .filter(|f| f.triple.as_ref().map_or(false, |ft| {
                    if is_exclusive {
                        ft.subject == t.subject && ft.predicate == t.predicate && ft.object != t.object
                    } else {
                        false // Non-exclusive predicates can have multiple values
                    }
                }))
                .map(|f| f.id.clone())
                .collect();

            for conflict_id in conflicting {
                warn!("Conflict detected during ingestion: {} contradicts existing {}", id, conflict_id);
                self.contradictions.push((id.to_string(), conflict_id));
            }

            self.subject_index.entry(t.subject.clone()).or_insert_with(Vec::new).push(id.to_string());
        }

        let fact = self.facts.entry(id.to_string()).or_insert(KnowledgeFact {
            id: id.to_string(),
            triple,
            confidence: 0.5,
            source_reliability: reliability,
            reinforcement_count: 0,
            timestamp: now,
        });

        // Use trust scoring in confidence calculation
        let source_id = id.split(':').next().unwrap_or("unknown");
        let trust = self.source_trust.get(source_id).cloned().unwrap_or(1.0);
        let weighted_reliability = reliability * trust;

        let prior = fact.confidence;
        let n = fact.reinforcement_count as f32;
        fact.confidence = (prior * n + weighted_reliability) / (n + 1.0);
        fact.reinforcement_count += 1;
        fact.timestamp = now;
    }

    pub fn reinforce(&mut self, id: &str, evidence_reliability: f32) {
        if let Some(fact) = self.facts.get_mut(id) {
            // Step 2 & 3: Bayesian-like update
            // New Confidence = (Prior * Likelihood) / (Prior * Likelihood + (1-Prior)*(1-Likelihood))
            // Simplified for Phase-1 industrial logic:
            let prior = fact.confidence;
            let likelihood = evidence_reliability;

            let numerator = prior * likelihood;
            let denominator = numerator + (1.0 - prior) * (1.0 - likelihood);

            if denominator > 0.0 {
                fact.confidence = numerator / denominator;
            } else {
                fact.confidence = (prior + likelihood) / 2.0;
            }

            fact.reinforcement_count += 1;
            fact.timestamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
            debug!("Fact {} Bayesian reinforced to confidence {:.2}", id, fact.confidence);
        }
    }

    pub fn penalize(&mut self, id: &str, penalty_weight: f32) {
        let mut source_id_to_update = None;
        if let Some(fact) = self.facts.get_mut(id) {
            // Negative feedback: Dampen confidence and reliability
            fact.confidence *= (1.0 - penalty_weight).max(0.0);
            fact.source_reliability *= (1.0 - penalty_weight * 0.5).max(0.1);

            source_id_to_update = Some(id.split(':').next().unwrap_or("unknown").to_string());
            debug!("Fact {} penalized: new confidence {:.2}", id, fact.confidence);
        }

        if let Some(source_id) = source_id_to_update {
            self.update_source_trust(&source_id, -penalty_weight * 0.1);
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
            let (conf_a, source_a) = self.facts.get(&a_id).map(|f| (f.confidence, f.id.split(':').next().unwrap_or("unknown").to_string())).unwrap_or((0.0, "unknown".to_string()));
            let (conf_b, source_b) = self.facts.get(&b_id).map(|f| (f.confidence, f.id.split(':').next().unwrap_or("unknown").to_string())).unwrap_or((0.0, "unknown".to_string()));

            // Step 28: Complex resolution based on confidence and trust
            if conf_a > conf_b * 1.5 {
                info!("Resolving conflict: Favoring {} (high confidence) over {}", a_id, b_id);
                if let Some(f) = self.facts.get_mut(&b_id) { f.confidence *= 0.1; }
                self.update_source_trust(&source_a, 0.05);
                self.update_source_trust(&source_b, -0.1);
            } else if conf_b > conf_a * 1.5 {
                info!("Resolving conflict: Favoring {} (high confidence) over {}", b_id, a_id);
                if let Some(f) = self.facts.get_mut(&a_id) { f.confidence *= 0.1; }
                self.update_source_trust(&source_b, 0.05);
                self.update_source_trust(&source_a, -0.1);
            } else {
                // Ambiguous conflict: Dampen both
                if let Some(f) = self.facts.get_mut(&a_id) { f.confidence *= 0.8; }
                if let Some(f) = self.facts.get_mut(&b_id) { f.confidence *= 0.8; }
                self.update_source_trust(&source_a, -0.02);
                self.update_source_trust(&source_b, -0.02);
            }
        }
        self.contradictions.clear();
    }

    pub fn update_source_trust(&mut self, source_id: &str, delta: f32) {
        let trust = self.source_trust.entry(source_id.to_string()).or_insert(1.0);
        *trust = (*trust + delta).clamp(0.1, 2.0);
    }

    /// Temporal Truth: Retrieves the most recent fact about a subject/predicate pair.
    pub fn get_recent_truth(&self, subject: &str, predicate: &str) -> Option<&KnowledgeFact> {
        self.facts.values()
            .filter(|f| f.triple.as_ref().map_or(false, |t| t.subject == subject && t.predicate == predicate))
            .max_by_key(|f| f.timestamp)
    }
}

pub struct ReasoningEngine;

impl ReasoningEngine {
    /// Performs deep transitive inference with Bayesian confidence propagation.
    pub fn infer_transitive(graph: &KnowledgeGraph, start: &str, predicate: &str, max_depth: usize) -> Option<(String, f32)> {
        let mut queue = VecDeque::new();
        queue.push_back((start.to_string(), 1.0, 0));

        let mut visited = HashMap::new();
        visited.insert(start.to_string(), 1.0);

        while let Some((curr, conf, depth)) = queue.pop_front() {
            if depth >= max_depth { continue; }

            if let Some(fact_ids) = graph.subject_index.get(&curr) {
                for id in fact_ids {
                    if let Some(fact) = graph.facts.get(id) {
                        if let Some(ref triple) = fact.triple {
                            if triple.predicate == predicate {
                                // Bayesian Propagation: C(A->C) = C(A->B) * C(B->C)
                                let new_conf = conf * fact.confidence;
                                if !visited.contains_key(&triple.object) || visited[&triple.object] < new_conf {
                                    visited.insert(triple.object.clone(), new_conf);
                                    queue.push_back((triple.object.clone(), new_conf, depth + 1));
                                }
                            }
                        }
                    }
                }
            }
        }

        // Return best explanation with confidence above industrial threshold
        visited.into_iter()
            .filter(|(k, _)| k != start)
            .filter(|(_, v)| *v > 0.1)
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
    }

    pub fn verify_fact(graph: &KnowledgeGraph, triple: &FactTriple) -> (bool, f32) {
        for fact in graph.facts.values() {
            if let Some(ref ft) = fact.triple {
                if ft.subject == triple.subject && ft.predicate == triple.predicate && ft.object != triple.object {
                    return (false, fact.confidence);
                }
            }
        }
        (true, 1.0)
    }
}
