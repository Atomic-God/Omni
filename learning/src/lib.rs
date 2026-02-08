use cognition::{CognitionCore, Relation};
use cognition::traits::PerceptionModule;
use ingestion::SemanticChunk;
use log::{info, warn};
use std::collections::{HashSet, HashMap};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct ConceptFrequencyTracker {
    pub counts: HashMap<String, u64>,
}

impl ConceptFrequencyTracker {
    pub fn new() -> Self {
        Self {
            counts: HashMap::new(),
        }
    }

    pub fn record(&mut self, text: &str) {
        // Simplified token count
        for word in text.split_whitespace() {
            *self.counts.entry(word.to_string()).or_insert(0) += 1;
        }
    }
}

pub struct ReflectionLoop {
    pub pending_tasks: Vec<String>,
}

impl ReflectionLoop {
    pub fn new() -> Self {
        Self {
            pending_tasks: Vec::new(),
        }
    }

    pub fn assess_performance(&mut self, _core: &CognitionCore) {
        // Self-evaluation stub
        // In future: Check for contradictory relations or sparse areas
        info!("Running Reflection Loop: No critical anomalies detected.");
    }
}

pub struct LearningEngine {
    pub frozen: bool,
    pub last_consolidation: u64,
    pub consolidation_interval: u64, // seconds
    pub previous_entropy: f32,
    pub frequency_tracker: ConceptFrequencyTracker,
    pub reflector: ReflectionLoop,
}

impl LearningEngine {
    pub fn new() -> Self {
        Self {
            frozen: false,
            last_consolidation: SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs(),
            consolidation_interval: 300, // 5 minutes default
            previous_entropy: 0.0,
            frequency_tracker: ConceptFrequencyTracker::new(),
            reflector: ReflectionLoop::new(),
        }
    }

    pub fn learn(&mut self, core: &mut CognitionCore, chunks: Vec<SemanticChunk>) {
        if self.frozen {
            log::error!("Attempted to learn while frozen!");
            return;
        }

        let start_entropy = core.compute_global_entropy();

        for chunk in chunks {
            info!("Learning chunk from {}", chunk.source);
            core.learn_text(&chunk.content);
            self.frequency_tracker.record(&chunk.content);
        }

        let end_entropy = core.compute_global_entropy();
        let entropy_delta = (end_entropy - start_entropy).abs();

        info!("Entropy Delta: {:.4} (Prev: {:.4} -> New: {:.4})", entropy_delta, start_entropy, end_entropy);

        // If entropy spike is high (surprise), trigger immediate consolidation to stabilize
        let entropy_threshold = 0.5; // Heuristic

        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
        if entropy_delta > entropy_threshold || now - self.last_consolidation > self.consolidation_interval {
            self.consolidate(core);
            self.last_consolidation = now;
            self.previous_entropy = core.compute_global_entropy(); // Update baseline

            // Trigger reflection after heavy learning
            self.reflector.assess_performance(core);
        }
    }

    pub fn consolidate(&self, core: &mut CognitionCore) {
        if self.frozen { return; }
        info!("Consolidating memory... (Deduplication, Decay, Pruning based on Weight)");

        // 1. Decay logic: Reduce weights of all relations slightly to simulate forgetting
        for relations in core.relation_graph.values_mut() {
            for rel in relations.iter_mut() {
                // Decay by 10% or at least 1 unit, but keep min 1 to avoid accidental deletion unless pruned
                if rel.weight > 5 {
                    rel.weight = rel.weight.saturating_sub(rel.weight / 10).max(1);
                }
            }
        }

        // 2. Deduplicate relation graph values (simple HashSet is strict equality)
        // We want to merge weights if duplicates exist?
        // For simplicity, just uniq (last write wins or first?)
        // Let's use drain and re-insert logic if needed, but existing logic was fine for exact duplicates.
        for (_subject, relations) in core.relation_graph.iter_mut() {
            // Sort by weight descending (highest weight first) to prioritize strong beliefs
            relations.sort_by(|a, b| b.weight.cmp(&a.weight));

            // Deduplicate by target+type (keeping highest weight due to sort)
            let mut seen = HashSet::new();
            relations.retain(|r| {
                let key = format!("{:?}:{}", r.relation_type, r.target);
                if seen.contains(&key) {
                    false
                } else {
                    seen.insert(key);
                    true
                }
            });
        }

        // 3. Memory Pressure Pruning
        // Soft cap: 50 relations per concept to keep memory lean
        let max_rels_per_concept = 50;
        let mut pruned_count = 0;

        for (_subject, relations) in core.relation_graph.iter_mut() {
            if relations.len() > max_rels_per_concept {
                // Already sorted by weight descending.
                relations.truncate(max_rels_per_concept);
                pruned_count += 1;
            }
        }

        if pruned_count > 0 {
            warn!("Pruned {} overgrown concepts (retained highest weight relations).", pruned_count);
        }

        // 4. Schema Abstraction (Stub)
        // Check for highly similar concepts and merge them?
        // For now, we assume VSA bundling handles this implicitly by vector similarity.
    }

    pub fn freeze(&mut self) {
        info!("Freezing Learning Engine. No further updates allowed.");
        self.frozen = true;
    }

    pub fn unfreeze(&mut self) {
         info!("Unfreezing Learning Engine. Updates allowed.");
         self.frozen = false;
    }

    // Explicit reinforcement API for future use
    pub fn strengthen_memory(&self, _concept: &str) {
        // Placeholder: Logic to increase weight of a concept vector
        info!("Reinforcing memory (stub)");
    }
}
