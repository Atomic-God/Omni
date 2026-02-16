use std::collections::{HashMap, HashSet};
use core_vsa::HyperVector;
use log::info;
use crate::hierarchy::MemoryEntry;

pub struct PrototypeConsolidator;

impl PrototypeConsolidator {
    /// Merges highly similar memories into abstracted 'Prototypes'.
    /// Returns a list of (Key, PrototypeVector) to be updated in the semantic layer.
    pub fn generate_prototypes(
        metadata: &HashMap<String, MemoryEntry>,
        similarity_threshold: f32
    ) -> Vec<(String, HyperVector)> {
        let mut prototypes = Vec::new();
        let mut visited = HashSet::new();
        let keys: Vec<String> = metadata.keys().cloned().collect();

        for i in 0..keys.len() {
            let key_a = &keys[i];
            if visited.contains(key_a) { continue; }

            let entry_a = &metadata[key_a];
            let mut cluster = vec![entry_a.vector.clone()];
            visited.insert(key_a.clone());

            for j in (i+1)..keys.len() {
                let key_b = &keys[j];
                if visited.contains(key_b) { continue; }

                let entry_b = &metadata[key_b];
                let sim = entry_a.vector.similarity(&entry_b.vector);

                if sim > similarity_threshold {
                    cluster.push(entry_b.vector.clone());
                    visited.insert(key_b.clone());
                }
            }

            if cluster.len() > 3 { // Industrial Threshold: Only abstract if many examples
                info!("Memory Core: Consolidating {} instances into a new Semantic Prototype: {}", cluster.len(), key_a);
                let mut prototype = cluster[0].clone();
                for k in 1..cluster.len() {
                    prototype = prototype.bundle(&cluster[k]);
                }
                prototypes.push((key_a.clone(), prototype));
            }
        }

        prototypes
    }
}
