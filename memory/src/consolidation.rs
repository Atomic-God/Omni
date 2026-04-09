use std::collections::{BTreeMap, HashSet};
use core_vsa::HyperVector;
use tracing::info;
use crate::hierarchy::MemoryEntry;

pub struct PrototypeConsolidator;

impl PrototypeConsolidator {
    /// Merges highly similar memories into abstracted 'Prototypes'.
    /// Returns a list of (PrototypeKey, PrototypeVector, Members) to be updated and linked in KG.
    pub fn generate_prototypes(
        metadata: &BTreeMap<String, MemoryEntry>,
        similarity_threshold: f32
    ) -> Vec<(String, HyperVector, Vec<String>)> {
        let mut prototypes = Vec::new();
        let mut visited = HashSet::new();
        let keys: Vec<String> = metadata.keys().cloned().collect();

        for i in 0..keys.len() {
            let key_a = &keys[i];
            if visited.contains(key_a) { continue; }

            let entry_a = &metadata[key_a];
            let mut cluster_vecs = vec![entry_a.vector.clone()];
            let mut cluster_keys = vec![key_a.clone()];
            visited.insert(key_a.clone());

            for j in (i+1)..keys.len() {
                let key_b = &keys[j];
                if visited.contains(key_b) { continue; }

                let entry_b = &metadata[key_b];
                let sim = entry_a.vector.similarity(&entry_b.vector);

                if sim > similarity_threshold {
                    cluster_vecs.push(entry_b.vector.clone());
                    cluster_keys.push(key_b.clone());
                    visited.insert(key_b.clone());
                }
            }

            if cluster_vecs.len() > 3 { // Industrial Threshold: Only abstract if many examples
                info!("Memory Core: Consolidating {} instances into a new Semantic Prototype: {}", cluster_vecs.len(), key_a);
                let mut prototype = cluster_vecs[0].clone();
                for k in 1..cluster_vecs.len() {
                    prototype = prototype.bundle(&cluster_vecs[k]);
                }

                prototypes.push((key_a.clone(), prototype, cluster_keys));
            }
        }

        prototypes
    }
}
