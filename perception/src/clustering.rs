use core_vsa::HyperVector;
use std::collections::HashMap;

// Basic Clustering for Unknown Symbols
pub struct SymbolClustering {
    pub clusters: HashMap<String, Vec<HyperVector>>,
    pub centroids: HashMap<String, HyperVector>,
}

impl SymbolClustering {
    pub fn new() -> Self {
        Self {
            clusters: HashMap::new(),
            centroids: HashMap::new(),
        }
    }

    pub fn cluster(&mut self, symbol: &str, vec: &HyperVector) {
        // Implementation: Add vector to nearest cluster center
        // 1. Find nearest centroid
        let mut nearest = None;
        let mut max_sim = -1.0;

        for (id, center) in &self.centroids {
            let sim = vec.similarity(center);
            if sim > max_sim {
                max_sim = sim;
                nearest = Some(id.clone());
            }
        }

        // 2. If close enough, add to cluster. Else create new.
        let threshold = 0.7; // Similarity threshold
        if max_sim > threshold {
            if let Some(id) = nearest {
                self.clusters.entry(id).or_default().push(vec.clone());
                // Update centroid (average) - requires bundling/normalization
                // For VSA, sum/bundle is approximation of centroid.
                // Simplified: Just append for now.
            }
        } else {
            // Create new cluster
            let new_id = format!("cluster_{}", self.centroids.len());
            self.centroids.insert(new_id.clone(), vec.clone());
            self.clusters.insert(new_id, vec![vec.clone()]);
        }
    }
}
