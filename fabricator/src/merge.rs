use crate::facade::OmniForge;
use engine::OmniMind;
use memory::{load_mind, save_mind, MindPack};
use std::sync::{Arc, Mutex};

// Merging Logic
pub fn merge_minds(primary_path: &str, secondary_path: &str, output_path: &str) -> Result<(), std::io::Error> {
    let primary_pack = load_mind(primary_path)?;
    let secondary_pack = load_mind(secondary_path)?;

    // We modify primary, merging secondary into it
    let mut merged_core = primary_pack.memory.core.clone();
    let secondary_core = secondary_pack.memory.core;

    // Merge Index Memory (Union)
    // In production, we'd need to reconcile divergent vectors for same keys if non-deterministic.
    // Since we are deterministic (hash-based), identical keys have identical vectors.
    // We only need to add missing keys.
    for (k, v) in secondary_core.index_memory {
        merged_core.index_memory.entry(k).or_insert(v);
    }

    // Merge Semantic Memory (Bundle)
    // For same keys, we bundle them to average the meaning.
    for (k, v) in secondary_core.semantic_memory {
        merged_core.semantic_memory.entry(k)
            .and_modify(|existing| *existing = existing.bundle(&v))
            .or_insert(v);
    }

    // Merge Relations (Union)
    for (k, mut v) in secondary_core.relation_graph {
        let entry = merged_core.relation_graph.entry(k).or_default();
        entry.append(&mut v);
        // Deduplication happens at next consolidation or here?
        // Let's rely on LearningEngine::consolidate if run later, or just simple dedupe here.
        // Doing strictly naive append is safer than complex merging logic for now.
    }

    // Create new pack
    let mut new_pack = primary_pack.clone();
    new_pack.memory.core = merged_core;
    // Update metadata?
    new_pack.metadata.source = format!("Merged: {} + {}", primary_path, secondary_path);

    save_mind(&new_pack, output_path)
}
