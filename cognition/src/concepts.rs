use std::collections::{HashMap, HashSet};
use crate::knowledge::{KnowledgeGraph};
use core_vsa::HyperVector;

#[derive(Debug, Clone)]
pub struct Topic {
    pub id: String,
    pub label: String,
    pub members: Vec<String>, // Fact IDs
    pub centroid: HyperVector,
    pub weight: f32,
}

pub struct ConceptFormationEngine;

impl ConceptFormationEngine {
    /// Detects emerging topics based on Knowledge Graph connectivity.
    /// Facts that share the same subject or object are clustered.
    pub fn detect_topics(graph: &KnowledgeGraph) -> Vec<Topic> {
        let mut topics = Vec::new();
        let mut visited_facts = HashSet::new();

        let fact_ids: Vec<String> = graph.facts.keys().cloned().collect();

        for fact_id in fact_ids {
            if visited_facts.contains(&fact_id) { continue; }

            let mut cluster = Vec::new();
            let mut to_process = vec![fact_id.clone()];
            visited_facts.insert(fact_id.clone());

            while let Some(current_id) = to_process.pop() {
                cluster.push(current_id.clone());

                if let Some(fact) = graph.facts.get(&current_id) {
                    if let Some(ref triple) = fact.triple {
                        // Find all facts sharing subject or object
                        for other_fact in graph.facts.values() {
                            if visited_facts.contains(&other_fact.id) { continue; }

                            if let Some(ref other_triple) = other_fact.triple {
                                if other_triple.subject == triple.subject || other_triple.object == triple.object
                                   || other_triple.subject == triple.object || other_triple.object == triple.subject {
                                    visited_facts.insert(other_fact.id.clone());
                                    to_process.push(other_fact.id.clone());
                                }
                            }
                        }
                    }
                }
            }

            if cluster.len() >= 3 {
                let topic = Self::create_topic_from_cluster(cluster, graph);
                topics.push(topic);
            }
        }

        topics
    }

    fn create_topic_from_cluster(members: Vec<String>, graph: &KnowledgeGraph) -> Topic {
        let mut subjects = HashMap::new();

        for id in &members {
            if let Some(fact) = graph.facts.get(id) {
                if let Some(ref triple) = fact.triple {
                    *subjects.entry(triple.subject.clone()).or_insert(0) += 1;
                    *subjects.entry(triple.object.clone()).or_insert(0) += 1;
                }

                // We'd need a vector for the fact.
                // Facts in KG don't currently store vectors directly, but they correspond to memory entries.
                // For Phase-1, we'll use a deterministic vector based on the label.
            }
        }

        // Label is the most frequent entity in the cluster
        let label = subjects.into_iter()
            .max_by_key(|&(_, count)| count)
            .map(|(s, _)| s)
            .unwrap_or_else(|| "General Topic".to_string());

        Topic {
            id: format!("topic:{}", seahash::hash(label.as_bytes())),
            label: label.clone(),
            members,
            centroid: HyperVector::deterministic(seahash::hash(label.as_bytes())),
            weight: 1.0,
        }
    }

    /// Groups related topics into higher-level domains.
    pub fn abstract_domains(topics: &[Topic]) -> Vec<Domain> {
        let mut domains = Vec::new();
        let mut visited = HashSet::new();

        for i in 0..topics.len() {
            if visited.contains(&i) { continue; }

            let mut domain_members = vec![topics[i].clone()];
            visited.insert(i);

            for j in (i+1)..topics.len() {
                if visited.contains(&j) { continue; }

                // Similarity between topics based on shared entities (simple heuristic)
                if topics[i].centroid.similarity(&topics[j].centroid) > 0.5 {
                    domain_members.push(topics[j].clone());
                    visited.insert(j);
                }
            }

            if !domain_members.is_empty() {
                domains.push(Domain {
                    id: format!("domain:{}", seahash::hash(topics[i].label.as_bytes())),
                    label: format!("{} Domain", topics[i].label),
                    topics: domain_members,
                });
            }
        }
        domains
    }
}

#[derive(Debug, Clone)]
pub struct Domain {
    pub id: String,
    pub label: String,
    pub topics: Vec<Topic>,
}
