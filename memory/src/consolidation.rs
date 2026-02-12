use core_vsa::HyperVector;
use std::collections::HashMap;

/// Handles memory consolidation based on relevance (simulating neural attention/importance).
pub struct Consolidator;

impl Consolidator {
    /// Filters vectors based on a relevance score threshold.
    /// In a full system, relevance comes from activation frequency or neural attention weights.
    pub fn consolidate(
        memory_buffer: &HashMap<String, HyperVector>,
        relevance_scores: &HashMap<String, f32>,
        threshold: f32
    ) -> HashMap<String, HyperVector> {
        let mut consolidated = HashMap::new();

        for (id, vec) in memory_buffer {
            let score = relevance_scores.get(id).cloned().unwrap_or(0.0);
            if score >= threshold {
                consolidated.insert(id.clone(), vec.clone());
            }
        }

        consolidated
    }

    /// Merges similar vectors (clustering).
    /// Simple greedy clustering for Phase 1.
    pub fn cluster_merge(
        vectors: &HashMap<String, HyperVector>,
        similarity_threshold: f32
    ) -> HashMap<String, HyperVector> {
        let mut merged = HashMap::new();
        let mut visited = std::collections::HashSet::new();

        // This is O(N^2), so only suitable for small consolidation batches (e.g. working memory).
        let keys: Vec<_> = vectors.keys().collect();

        for i in 0..keys.len() {
            if visited.contains(keys[i]) { continue; }

            let mut cluster = vec![vectors[keys[i]].clone()];
            visited.insert(keys[i].clone());

            for j in (i+1)..keys.len() {
                if visited.contains(keys[j]) { continue; }

                let sim = vectors[keys[i]].similarity(&vectors[keys[j]]);
                if sim > similarity_threshold {
                    cluster.push(vectors[keys[j]].clone());
                    visited.insert(keys[j].clone());
                }
            }

            // Bundle cluster
            let mut centroid = cluster[0].clone();
            for k in 1..cluster.len() {
                centroid = centroid.bundle(&cluster[k]);
            }

            merged.insert(keys[i].clone(), centroid);
        }

        merged
    }
}
