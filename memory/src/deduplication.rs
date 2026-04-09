use std::collections::BTreeMap;
use crate::hierarchy::MemoryEntry;
use tracing::info;

pub struct Deduplicator;

impl Deduplicator {
    /// Merges near-identical memories (similarity > threshold).
    pub fn merge_similar(entries: &mut BTreeMap<String, MemoryEntry>, threshold: f32) -> usize {
        let mut merged_count = 0;
        let keys: Vec<String> = entries.keys().cloned().collect();
        let mut to_remove = Vec::new();

        for i in 0..keys.len() {
            let key_a = &keys[i];
            if to_remove.contains(key_a) { continue; }

            for j in (i + 1)..keys.len() {
                let key_b = &keys[j];
                if to_remove.contains(key_b) { continue; }

                let (vec_a, vec_b) = {
                    let a = entries.get(key_a).unwrap();
                    let b = entries.get(key_b).unwrap();
                    (a.vector.clone(), b.vector.clone())
                };

                if vec_a.similarity(&vec_b) > threshold {
                    // Merge B into A
                    let entry_b = entries.get(key_b).unwrap().clone();
                    let entry_a = entries.get_mut(key_a).unwrap();

                    entry_a.reinforcement_count += entry_b.reinforcement_count;
                    entry_a.access_count += entry_b.access_count;
                    entry_a.confidence = (entry_a.confidence + entry_b.confidence) / 2.0;

                    // Probabilistic bundling for the vector
                    entry_a.vector = entry_a.vector.bundle(&entry_b.vector);

                    to_remove.push(key_b.clone());
                    merged_count += 1;
                }
            }
        }

        for key in to_remove {
            entries.remove(&key);
        }

        if merged_count > 0 {
            info!("Industrial Deduplication: Merged {} redundant cognitive entries.", merged_count);
        }
        merged_count
    }
}
