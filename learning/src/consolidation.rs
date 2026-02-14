use std::collections::{HashMap, HashSet};
use core_vsa::HyperVector;
use log::{info, warn};

pub struct ConsolidationEngine;

impl ConsolidationEngine {
    /// Merges similar vectors in the working memory to form stable concepts.
    /// Returns a list of merged concepts.
    pub fn consolidate(
        items: &mut HashMap<String, HyperVector>,
        similarity_threshold: f32
    ) -> Vec<(String, HyperVector)> {
        let mut merged = Vec::new();
        let mut visited = HashSet::new();
        let keys: Vec<String> = items.keys().cloned().collect();

        for i in 0..keys.len() {
            if visited.contains(&keys[i]) { continue; }

            let mut cluster = vec![items[&keys[i]].clone()];
            visited.insert(keys[i].clone());

            for j in (i+1)..keys.len() {
                if visited.contains(&keys[j]) { continue; }

                let sim = items[&keys[i]].similarity(&items[&keys[j]]);
                if sim > similarity_threshold {
                    cluster.push(items[&keys[j]].clone());
                    visited.insert(keys[j].clone());
                }
            }

            if cluster.len() > 1 {
                // Bundle cluster
                let mut centroid = cluster[0].clone();
                for k in 1..cluster.len() {
                    centroid = centroid.bundle(&cluster[k]);
                }
                merged.push((keys[i].clone(), centroid));
            } else {
                // Keep singleton? Or consider it noise if low importance?
                // For now, keep singletons as is (no merge).
            }
        }

        // Update items with merged centroids?
        // Real implementation would replace individual items with the concept.
        for (key, centroid) in &merged {
            items.insert(key.clone(), centroid.clone());
        }

        merged
    }

    pub fn decay_and_prune(
        items: &mut HashMap<String, crate::MemoryEntry>,
        decay_rate: f32,
        prune_threshold: f32
    ) {
        // Decay
        for entry in items.values_mut() {
            entry.decay(decay_rate);
        }

        // Prune
        let initial_count = items.len();
        items.retain(|_, v| v.importance > prune_threshold);
        let final_count = items.len();

        if initial_count > final_count {
            info!("Pruned {} low-relevance items.", initial_count - final_count);
        }
    }
}
