#![deny(warnings)]
use core_vsa::SymbolGraph;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

pub mod sequence;
pub mod abductive;
pub mod intent;
pub mod planning;
pub mod world_model;
pub mod inference;
pub mod temporal;
pub mod knowledge;

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
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningTrace {
    pub steps: Vec<String>,
    pub final_confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitionCore {
    pub relation_graph: HashMap<String, Vec<Relation>>,
    pub trace_log: Vec<ReasoningTrace>,
    pub knowledge_graph: knowledge::KnowledgeGraph,
}

impl CognitionCore {
    pub fn new() -> Self {
        Self {
            relation_graph: HashMap::new(),
            trace_log: Vec::new(),
            knowledge_graph: knowledge::KnowledgeGraph::new(),
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

    pub fn reinforce_knowledge(&mut self, id: &str, reliability: f32) {
        self.knowledge_graph.reinforce(id, reliability);
    }

    pub fn penalize_knowledge(&mut self, id: &str, penalty: f32) {
        self.knowledge_graph.penalize(id, penalty);
    }

    pub fn resolve_contradictions(&mut self) {
        self.knowledge_graph.resolve_all_contradictions();
    }

    pub fn apply_knowledge_decay(&mut self, decay_rate: f32) {
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
        self.knowledge_graph.apply_temporal_decay(decay_rate, now);
    }

    pub fn add_fact_triple(&mut self, triple: core_vsa::FactTriple, confidence: f32) {
        let rel_type = match triple.predicate.as_str() {
            "taxonomy" | "is_a" | "es" | "son" | "ist" | "sind" => RelationType::Taxonomic,
            "causality" | "causes" | "triggers" | "causa" | "verursacht" => RelationType::Causal,
            "contradicts" | "contradice" | "widerspricht" => RelationType::Contradictory,
            "means" | "significa" | "bedeutet" => {
                self.link_multilingual_symbols(&triple.subject, &triple.object, confidence);
                RelationType::Taxonomic
            },
            _ => RelationType::Structural,
        };
        self.add_relation(&triple.subject, &triple.object, rel_type, 1.0, confidence);

        let fact_id = format!("{}-{}-{}", triple.subject, triple.predicate, triple.object);
        self.knowledge_graph.add_fact(&fact_id, Some(triple), confidence);
    }

    pub fn ingest_from_graph(&mut self, graph: &SymbolGraph) {
        for edge in &graph.edges {
            let triple = core_vsa::FactTriple {
                subject: edge.source.clone(),
                predicate: edge.relation.clone(),
                object: edge.target.clone(),
            };
            self.add_fact_triple(triple, edge.confidence);
        }
    }

    /// Performs a path-finding search between concepts and returns a ReasoningTrace.
    pub fn find_path(&self, start: &str, end: &str, max_depth: usize) -> Option<ReasoningTrace> {
        let mut queue = std::collections::VecDeque::new();
        queue.push_back((start.to_string(), vec![start.to_string()], 1.0));

        let mut visited = HashMap::new();

        while let Some((curr, path, conf)) = queue.pop_front() {
            if curr == end {
                return Some(ReasoningTrace {
                    steps: path,
                    final_confidence: conf,
                });
            }

            if path.len() > max_depth { continue; }

            if let Some(relations) = self.relation_graph.get(&curr) {
                for rel in relations {
                    let next_conf = conf * rel.confidence;
                    if !visited.contains_key(&rel.target) || visited[&rel.target] < next_conf {
                        visited.insert(rel.target.clone(), next_conf);
                        let mut next_path = path.clone();
                        next_path.push(rel.target.clone());
                        queue.push_back((rel.target.clone(), next_path, next_conf));
                    }
                }
            }
        }
        None
    }
}
