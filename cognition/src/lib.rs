use core_vsa::{HyperVector, SymbolGraph};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

pub mod sequence;
pub mod abductive;
pub mod intent;
pub mod planning;
pub mod world_model;
pub mod inference;
pub mod temporal;

pub use sequence::SequenceResonator;
pub use abductive::AbductiveReasoner;
pub use intent::{IntentResolver, Intent};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RelationType {
    Taxonomic, // "is a"
    Causal,    // "causes"
    Temporal,  // "before/after"
    Structural, // "part of"
    Contradictory, // Added
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relation {
    pub relation_type: RelationType,
    pub target: String,
    pub weight: f32,
    pub confidence: f32, // Added
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningTrace {
    pub steps: Vec<String>,
    pub final_confidence: f32,
}

pub struct CognitionCore {
    pub relation_graph: HashMap<String, Vec<Relation>>,
    pub trace_log: Vec<ReasoningTrace>, // Added
}

impl CognitionCore {
    pub fn new() -> Self {
        Self {
            relation_graph: HashMap::new(),
            trace_log: Vec::new(),
        }
    }

    pub fn compute_global_entropy(&self) -> f32 {
        let mut total_uncertainty = 0.0;
        let mut count = 0;
        for relations in self.relation_graph.values() {
            for rel in relations {
                total_uncertainty += 1.0 - rel.confidence;
                count += 1;
            }
        }
        if count == 0 { 0.0 } else { total_uncertainty / count as f32 }
    }

    pub fn add_relation(&mut self, source: &str, target: &str, rel_type: RelationType, weight: f32, confidence: f32) {
        let relations = self.relation_graph.entry(source.to_string()).or_insert_with(Vec::new);
        relations.push(Relation {
            relation_type: rel_type,
            target: target.to_string(),
            weight,
            confidence,
        });
    }

    pub fn detect_contradictions(&self, concept: &str) -> Vec<String> {
        let mut contradictions = Vec::new();
        if let Some(relations) = self.relation_graph.get(concept) {
            for rel in relations {
                if rel.relation_type == RelationType::Contradictory {
                    contradictions.push(rel.target.clone());
                }
            }
        }
        contradictions
    }

    pub fn link_multilingual_symbols(&mut self, source_id: &str, target_id: &str, confidence: f32) {
        self.add_relation(source_id, target_id, RelationType::Taxonomic, 1.0, confidence);
        self.add_relation(target_id, source_id, RelationType::Taxonomic, 1.0, confidence);
    }

    pub fn ingest_from_graph(&mut self, graph: &SymbolGraph) {
        for edge in &graph.edges {
            let rel_type = match edge.relation.as_str() {
                "is_a" | "has_field" => RelationType::Taxonomic,
                "causes" => RelationType::Causal,
                "contradicts" => RelationType::Contradictory,
                _ => RelationType::Structural,
            };
            self.add_relation(&edge.source, &edge.target, rel_type, edge.weight, edge.confidence);
        }
    }
}
