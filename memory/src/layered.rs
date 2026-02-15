use core_vsa::HyperVector;
use std::sync::Arc;
use log::info;
use crate::{MemoryManager, MemoryStore, HierarchicalMemory, MemoryLayer};
use std::error::Error;

/// A Layered Memory Store that composes a Read-Only Base with a Read-Write Delta.
pub struct LayeredMemory {
    pub base: Arc<MemoryManager>, // Read-Only Base Mind
    pub delta: MemoryManager,     // Read-Write Personal Delta
}

impl LayeredMemory {
    pub fn new(base: Arc<MemoryManager>, delta_path: &std::path::Path) -> Self {
        let delta = MemoryManager::new(delta_path);
        Self { base, delta }
    }

    /// Saves ONLY the delta layer.
    pub fn save_delta(&self, name: &str) -> Result<(), Box<dyn Error>> {
        info!("Saving Delta Snapshot: {}", name);
        self.delta.save_snapshot(name)
    }

    /// Computes diff stats (how much has the delta diverged).
    pub fn diff_stats(&self) -> String {
        let new_items = self.delta.metadata.len();
        format!("Delta contains {} new/modified items.", new_items)
    }
}

// Delegate MemoryStore to Delta (Write) or Delta->Base (Read)
impl MemoryStore for LayeredMemory {
    fn store(&mut self, key: &str, vector: HyperVector) -> Result<(), Box<dyn Error>> {
        // Always write to Delta
        self.delta.store(key, vector)
    }

    fn retrieve(&self, key: &str) -> Option<HyperVector> {
        // Check Delta first (Overwrites/New)
        if let Some(v) = self.delta.retrieve(key) {
            return Some(v);
        }
        // Fallback to Base
        self.base.retrieve(key)
    }

    fn query_nearest(&self, query: &HyperVector, k: usize) -> Vec<(String, f32)> {
        // Query both, merge results
        let mut results = self.delta.query_nearest(query, k);
        let base_results = self.base.query_nearest(query, k);

        results.extend(base_results);
        // Deduplicate and sort
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        // Simple dedup by key
        let mut seen = std::collections::HashSet::new();
        let mut final_res = Vec::new();
        for res in results {
            if seen.insert(res.0.clone()) {
                final_res.push(res);
            }
            if final_res.len() >= k { break; }
        }
        final_res
    }
}

// Delegate HierarchicalMemory
impl HierarchicalMemory for LayeredMemory {
    fn store_in_layer(&mut self, key: &str, vector: HyperVector, layer: MemoryLayer) -> Result<(), Box<dyn Error>> {
        self.delta.store_in_layer(key, vector, layer)
    }

    fn retrieve_with_metrics(&mut self, key: &str) -> Option<HyperVector> {
        // If in Delta, update metrics there
        if self.delta.metadata.contains_key(key) {
            return self.delta.retrieve_with_metrics(key);
        }
        // If in Base, we technically "cow" (copy-on-write) metadata to delta?
        // Or just read without updating metrics in immutable base?
        // "Personal adaptation" implies we should track usage even of base items.
        // So we should promote base item metadata to delta if accessed frequently.

        if let Some(v) = self.base.retrieve(key) {
            // Implicit promotion to Working Memory in Delta for tracking?
            // Let's do it lazily or just return value for now.
            // For Phase 1, we just return.
            return Some(v);
        }
        None
    }

    fn consolidate_layers(&mut self) {
        self.delta.consolidate_layers();
    }
}
