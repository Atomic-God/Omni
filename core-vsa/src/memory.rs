use crate::hypervector::{Hypervector, VsaOps};
use std::collections::HashMap;

pub struct AssociativeMemory {
    store: HashMap<String, Hypervector>,
}

impl AssociativeMemory {
    pub fn new() -> Self {
        Self {
            store: HashMap::new(),
        }
    }

    pub fn add(&mut self, key: String, vector: Hypervector) {
        self.store.insert(key, vector);
    }

    pub fn query(&self, probe: &Hypervector) -> Option<(String, f32)> {
        let mut best_match: Option<(String, f32)> = None;
        let mut min_dist = 1.0;

        for (key, vec) in &self.store {
            let dist = probe.distance(vec);
            if dist < min_dist {
                min_dist = dist;
                best_match = Some((key.clone(), dist));
            }
        }
        best_match
    }
}
