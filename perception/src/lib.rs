use core_vsa::HyperVector;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use std::fs::File;
use std::io::BufReader;

#[derive(Serialize, Deserialize)]
pub struct Beagle {
    // Static random ID for each word (Environmental Vector)
    pub index_memory: HashMap<String, HyperVector>,
    // Learned meaning (Contextual Vector)
    pub semantic_memory: HashMap<String, HyperVector>,
    // Explicit relation graph for symbolic reasoning
    pub relation_graph: HashMap<String, Vec<String>>,
}

impl Beagle {
    pub fn new() -> Self {
        Self {
            index_memory: HashMap::new(),
            semantic_memory: HashMap::new(),
            relation_graph: HashMap::new(),
        }
    }

    pub fn sentence_vector(&self, sentence: &str) -> Option<HyperVector> {
        let words: Vec<String> = sentence
            .split_whitespace()
            .map(|s| s.to_lowercase().replace(|c: char| !c.is_alphanumeric(), ""))
            .filter(|s| !s.is_empty())
            .collect();

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

    fn get_or_create_index(&mut self, word: &str) -> HyperVector {
        if let Some(vec) = self.index_memory.get(word) {
            return vec.clone();
        }
        let vec = HyperVector::random();
        self.index_memory.insert(word.to_string(), vec.clone());
        // Initialize semantic memory as empty (zeros) or random?
        // Usually initialized as random or zero.
        // For our HyperVector (always +/-1), we initialize as a copy of index or random.
        // Let's initialize with random for now to act as "identity" before learning.
        self.semantic_memory.insert(word.to_string(), HyperVector::random());
        vec
    }

    pub fn learn_sentence(&mut self, sentence: &str) {
        let words: Vec<String> = sentence
            .split_whitespace()
            .map(|s| s.to_lowercase().replace(|c: char| !c.is_alphanumeric(), ""))
            .filter(|s| !s.is_empty())
            .collect();

        // 1. Ensure all words exist in index
        for word in &words {
            self.get_or_create_index(word);
        }

        // 1.5. Graph Learning (Simple Heuristic)
        // If sentence matches "A is B" or "A B" (e.g. "animal breathes"), link them.
        for i in 0..words.len() {
            if words[i] == "is" && i > 0 && i + 1 < words.len() {
                let subject = words[i-1].clone();
                let object = words[i+1].clone();
                self.relation_graph.entry(subject).or_insert_with(Vec::new).push(object);
            }
            // For "animal breathes" (noun verb), assume adjacent implies potential relation if no "is"
            // This is very simple; real BEAGLE doesn't do this, but Omni Forge v5 requirements ask for it.
            // Let's rely on explicit "is" or specific training structure for now to keep it clean,
            // or allow "subject verb" linking.
            if i + 1 < words.len() && words[i] != "is" && words[i+1] != "is" && words[i] != "the" && words[i] != "an" {
                 // Potentially link neighbors? Let's stick to explicit "is" and "noun verb" if strictly asked.
                 // The prompt: "Every learned sentence like 'dog is animal' should record: dog -> animal"
                 // And example "animal breathes" -> animal -> breathes.
                 let a = words[i].clone();
                 let b = words[i+1].clone();
                 self.relation_graph.entry(a).or_insert_with(Vec::new).push(b);
            }
        }

        // 2. Context Learning (Bag-of-Words Context)
        for (i, target_word) in words.iter().enumerate() {
            // Build context bundle from all OTHER words in the sentence
            let mut context_bundle: Option<HyperVector> = None;

            for (j, context_word) in words.iter().enumerate() {
                if i == j { continue; }

                let context_vec = self.index_memory.get(context_word).unwrap();

                context_bundle = match context_bundle {
                    Some(b) => Some(b.bundle(context_vec)),
                    None => Some(context_vec.clone()),
                };
            }

            if let Some(ctx) = context_bundle {
                // Update semantic memory: Semantic = Semantic + Context
                // Since our HyperVector is immutable and bundle creates new,
                // we replace the old one.
                if let Some(current_semantic) = self.semantic_memory.get(target_word) {
                    let new_semantic = current_semantic.bundle(&ctx);
                    self.semantic_memory.insert(target_word.clone(), new_semantic);
                }
            }
        }
    }

    pub fn similarity(&self, word1: &str, word2: &str) -> Option<f32> {
        let v1 = self.semantic_memory.get(word1)?;
        let v2 = self.semantic_memory.get(word2)?;
        Some(v1.similarity(v2))
    }

    pub fn most_similar(&self, word: &str) -> Vec<(String, f32)> {
        let target_vec = match self.semantic_memory.get(word) {
            Some(v) => v,
            None => return Vec::new(),
        };

        let mut results: Vec<(String, f32)> = self.semantic_memory.iter()
            .map(|(k, v)| (k.clone(), target_vec.similarity(v)))
            .collect();

        // Sort descending by score
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Take top 5
        results.into_iter().take(5).collect()
    }

    pub fn save(&self, path: &str) -> Result<(), std::io::Error> {
        let file = File::create(path)?;
        serde_json::to_writer(file, self)?;
        Ok(())
    }

    pub fn load(path: &str) -> Result<Self, std::io::Error> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let beagle = serde_json::from_reader(reader)?;
        Ok(beagle)
    }

    pub fn infer_relation(&self, start: &str, target: &str) -> bool {
        let threshold = 0.02;
        let mut queue = std::collections::VecDeque::new();
        let mut visited = std::collections::HashSet::new();

        queue.push_back(start.to_string());
        visited.insert(start.to_string());

        while let Some(current) = queue.pop_front() {
            if current == target {
                return true;
            }

            // 1. Check Explicit Graph Neighbors
            if let Some(neighbors) = self.relation_graph.get(&current) {
                for neighbor in neighbors {
                    if !visited.contains(neighbor) {
                        visited.insert(neighbor.clone());
                        queue.push_back(neighbor.clone());
                    }
                }
            }

            // 2. Check Implicit Similarity Neighbors (Soft Links)
            // This is expensive (scan all memory), so limiting to "most_similar" logic
            let similar_words = self.most_similar(&current);
            for (word, score) in similar_words {
                if score > threshold && !visited.contains(&word) {
                    visited.insert(word.clone());
                    queue.push_back(word);
                }
            }
        }
        false
    }

    pub fn answer(&self, query: &str) -> String {
        // Simple parser for "Does X Y?" or "Is X Y?"
        // Remove punctuation
        let clean_query = query.replace("?", "").to_lowercase();
        let words: Vec<&str> = clean_query.split_whitespace().collect();

        if words.len() < 3 {
            return "Query too short.".to_string();
        }

        // Logic: Extract Subject and Object/Verb
        // "Does dog breathe" -> Subject: dog, Target: breathe
        // "Is dog animal" -> Subject: dog, Target: animal
        let subject = words[1];
        let target = words.last().unwrap(); // Simple assumption

        if self.infer_relation(subject, target) {
            "Yes".to_string()
        } else {
            "No".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_beagle_learning() {
        let mut beagle = Beagle::new();

        // Train on simple corpus where "cat" and "dog" share context ("the", "barks"/"meows" might differ)
        // "The cat runs"
        // "The dog runs"
        // "The fish swims"

        beagle.learn_sentence("The cat runs");
        beagle.learn_sentence("The dog runs");
        beagle.learn_sentence("The fish swims");

        // "cat" context: {The, runs}
        // "dog" context: {The, runs}
        // "fish" context: {The, swims}

        // Cat and Dog should be very similar (identical context in this small corpus)
        let sim_cat_dog = beagle.similarity("cat", "dog").unwrap();

        // Cat and Fish share "The" but differ on "runs" vs "swims".
        // Similarity should be lower.
        let sim_cat_fish = beagle.similarity("cat", "fish").unwrap();

        println!("Sim Cat-Dog: {}", sim_cat_dog);
        println!("Sim Cat-Fish: {}", sim_cat_fish);

        assert!(sim_cat_dog > sim_cat_fish, "Cat and Dog should be more similar than Cat and Fish");
    }
}
