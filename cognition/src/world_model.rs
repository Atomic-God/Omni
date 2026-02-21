use crate::{CognitionCore, RelationType};
use std::collections::{HashSet, VecDeque};

// World Model Layer
// Implements Ontology and Causal Reasoning

#[derive(Debug, Clone)]
pub struct WorldModel {
    // Entities and Concepts
    // For now, we use the graph itself, but we can add metadata here.
}

impl CognitionCore {
    // === Ontology ===

    /// Returns true if 'a' is a 'b' (transitive).
    pub fn is_a(&self, a: &str, b: &str) -> bool {
        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();
        queue.push_back(a.to_string());
        visited.insert(a.to_string());

        while let Some(current) = queue.pop_front() {
            if current == b { return true; }

            if let Some(relations) = self.relation_graph.get(&current) {
                for rel in relations {
                    if rel.relation_type == RelationType::Taxonomic { // "is a"
                        if !visited.contains(&rel.target) {
                            visited.insert(rel.target.clone());
                            queue.push_back(rel.target.clone());
                        }
                    }
                }
            }
        }
        false
    }

    // === Causal Engine ===

    /// Traces causal chains from a starting concept.
    /// Returns a list of paths (chains of causes).
    pub fn trace_causes(&self, start: &str, max_depth: usize) -> Vec<Vec<String>> {
        let mut results = Vec::new();
        let mut queue = VecDeque::new();

        // (Current, Path)
        queue.push_back((start.to_string(), vec![start.to_string()]));

        while let Some((current, path)) = queue.pop_front() {
            if path.len() >= max_depth {
                results.push(path);
                continue;
            }

            let mut found_next = false;
            if let Some(relations) = self.relation_graph.get(&current) {
                for rel in relations {
                    if let RelationType::Causal(_) = rel.relation_type { // "causes"
                        if !path.contains(&rel.target) {
                            found_next = true;
                            let mut new_path = path.clone();
                            new_path.push(rel.target.clone());
                            queue.push_back((rel.target.clone(), new_path));
                        }
                    }
                }
            }

            if !found_next && path.len() > 1 {
                results.push(path);
            }
        }
        results
    }

    /// Predicts effects of a concept based on causal relations.
    pub fn predict_effects(&self, cause: &str) -> Vec<String> {
        let mut effects = Vec::new();
        if let Some(relations) = self.relation_graph.get(cause) {
            for rel in relations {
                if let RelationType::Causal(_) = rel.relation_type {
                    effects.push(rel.target.clone());
                }
            }
        }
        effects
    }
}
