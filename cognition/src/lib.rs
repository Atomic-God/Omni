use core_vsa::HyperVector;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod traits;

use core_vsa::index::LshIndex;
use perception::tokenizer;
use traits::{PerceptionModule, ReasoningModule};

/// The core cognitive engine implementing BEAGLE-style learning and VSA reasoning.
#[derive(Serialize, Deserialize, Clone)]
pub struct CognitionCore {
    /// Static random ID for each word (Environmental Vector).
    pub index_memory: HashMap<String, HyperVector>,
    /// Learned meaning (Contextual Vector) derived from co-occurrence.
    pub semantic_memory: HashMap<String, HyperVector>,
    /// Explicit relation graph for symbolic reasoning (e.g., "is-a" relationships).
    pub relation_graph: HashMap<String, Vec<String>>,
    /// Episodic memory storing structural sentence vectors via LSH Index.
    pub sentence_memory: LshIndex,
}

impl Default for CognitionCore {
    fn default() -> Self {
        Self::new()
    }
}

impl PerceptionModule for CognitionCore {
    fn learn_text(&mut self, text: &str) {
        self.learn_text_internal(text);
    }
}

impl ReasoningModule for CognitionCore {
    fn infer(&self, start: &str, target: &str) -> Option<Vec<String>> {
        self.infer(start, target)
    }

    fn query(&self, query_str: &str) -> String {
        self.query_internal(query_str)
    }

    fn sentence_vector(&self, sentence: &str) -> Option<HyperVector> {
        self.sentence_vector_internal(sentence)
    }
}

impl CognitionCore {
    /// Creates a new, empty CognitionCore.
    pub fn new() -> Self {
        Self {
            index_memory: HashMap::new(),
            semantic_memory: HashMap::new(),
            relation_graph: HashMap::new(),
            sentence_memory: LshIndex::default(),
        }
    }

    fn learn_text_internal(&mut self, text: &str) {
        let words = tokenizer::tokenize(text);

        // 1. Index Learning
        for word in &words {
            if !self.index_memory.contains_key(word) {
                let vec = HyperVector::random();
                self.index_memory.insert(word.clone(), vec.clone());
                self.semantic_memory
                    .insert(word.clone(), HyperVector::random());
            }
        }

        // 2. Graph Learning (X is Y)
        for i in 0..words.len() {
            if words[i] == "is" && i > 0 && i + 1 < words.len() {
                let subject = words[i - 1].clone();
                let object = words[i + 1].clone();
                self.relation_graph.entry(subject).or_default().push(object);
            }
            if i + 1 < words.len()
                && words[i] != "is"
                && words[i + 1] != "is"
                && words[i] != "the"
                && words[i] != "an"
            {
                let a = words[i].clone();
                let b = words[i + 1].clone();
                self.relation_graph.entry(a).or_default().push(b);
            }
        }

        // 3. Semantic Context Learning
        for (i, target_word) in words.iter().enumerate() {
            let mut context_bundle: Option<HyperVector> = None;
            for (j, context_word) in words.iter().enumerate() {
                if i == j {
                    continue;
                }
                let context_vec = self.index_memory.get(context_word).unwrap();
                context_bundle = match context_bundle {
                    Some(b) => Some(b.bundle(context_vec)),
                    None => Some(context_vec.clone()),
                };
            }
            if let Some(ctx) = context_bundle {
                if let Some(current_semantic) = self.semantic_memory.get(target_word) {
                    let new_semantic = current_semantic.bundle(&ctx);
                    self.semantic_memory
                        .insert(target_word.clone(), new_semantic);
                }
            }
        }

        // 4. Episodic Memory
        if let Some(sv) = self.sentence_vector(text) {
            self.sentence_memory.insert(sv);
        }
    }

    fn query_internal(&self, query_str: &str) -> String {
        let words = tokenizer::tokenize(query_str);

        if words.len() < 3 {
            return "Query too short.".to_string();
        }

        if words[0] == "what" && words[1] == "does" && words.len() >= 4 {
            let subject = &words[2];
            let verb = &words[3];
            let objects = self.query_subject_action(subject, verb);
            if !objects.is_empty() {
                return objects[0].clone();
            } else {
                return "Unknown".to_string();
            }
        }

        let subject = &words[1];
        let target = words.last().unwrap();

        if let Some(path) = self.infer(subject, target) {
            format!("Yes. Reasoning: {}", path.join(" -> "))
        } else {
            "No connection found.".to_string()
        }
    }

    pub fn infer(&self, start: &str, target: &str) -> Option<Vec<String>> {
        let threshold = 0.02;
        // Queue: (CurrentNode, PathSoFar)
        let mut queue = std::collections::VecDeque::new();
        let mut visited = std::collections::HashSet::new();

        queue.push_back((start.to_string(), vec![start.to_string()]));
        visited.insert(start.to_string());

        while let Some((current, path)) = queue.pop_front() {
            if current == target {
                return Some(path);
            }

            // Limit depth to avoid explosion in prototype
            if path.len() > 5 { continue; }

            // 1. Explicit
            if let Some(neighbors) = self.relation_graph.get(&current) {
                for neighbor in neighbors {
                    if !visited.contains(neighbor) {
                        visited.insert(neighbor.clone());
                        let mut new_path = path.clone();
                        new_path.push(neighbor.clone());
                        queue.push_back((neighbor.clone(), new_path));
                    }
                }
            }

            // 2. Implicit
            let similar_words = self.most_similar(&current);
            for (word, score) in similar_words {
                if score > threshold && !visited.contains(&word) {
                    visited.insert(word.clone());
                    let mut new_path = path.clone();
                    new_path.push(word.clone());
                    queue.push_back((word.clone(), new_path));
                }
            }
        }
        None
    }

    fn sentence_vector_internal(&self, sentence: &str) -> Option<HyperVector> {
        let words = tokenizer::tokenize(sentence);

        if words.len() >= 3 {
            let s_vec = self.semantic_memory.get(&words[0])?;
            let v_vec = self.semantic_memory.get(&words[1])?;
            let o_vec = self.semantic_memory.get(&words[2])?;

            let s_bound = s_vec.bind(&core_vsa::ROLE_SUBJECT);
            let v_bound = v_vec.bind(&core_vsa::ROLE_VERB);
            let o_bound = o_vec.bind(&core_vsa::ROLE_OBJECT);

            Some(s_bound.bundle(&v_bound).bundle(&o_bound))
        } else {
            let mut bundle: Option<HyperVector> = None;
            for word in words {
                if let Some(vec) = self.semantic_memory.get(&word) {
                    bundle = match bundle {
                        Some(b) => Some(b.bundle(vec)),
                        None => Some(vec.clone()),
                    };
                }
            }
            bundle
        }
    }

    fn query_subject_action(&self, subject: &str, verb: &str) -> Vec<String> {
        let s_vec = match self.semantic_memory.get(subject) {
            Some(v) => v,
            None => return Vec::new(),
        };
        let v_vec = match self.semantic_memory.get(verb) {
            Some(v) => v,
            None => return Vec::new(),
        };

        let query = s_vec
            .bind(&core_vsa::ROLE_SUBJECT)
            .bundle(&v_vec.bind(&core_vsa::ROLE_VERB));

        if let Some(sent) = self.sentence_memory.search(&query) {
            let object_guess = sent.bind(&core_vsa::ROLE_OBJECT);
            let mut results: Vec<(String, f32)> = self
                .semantic_memory
                .iter()
                .map(|(k, v)| (k.clone(), object_guess.similarity(v)))
                .collect();

            results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            results.into_iter().take(5).map(|(k, _)| k).collect()
        } else {
            Vec::new()
        }
    }

    pub fn most_similar(&self, word: &str) -> Vec<(String, f32)> {
        let target_vec = match self.semantic_memory.get(word) {
            Some(v) => v,
            None => return Vec::new(),
        };
        let mut results: Vec<(String, f32)> = self
            .semantic_memory
            .iter()
            .map(|(k, v)| (k.clone(), target_vec.similarity(v)))
            .collect();
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results.into_iter().take(5).collect()
    }
}
