use cognition::CognitionCore;
use cognition::traits::PerceptionModule;
use ingestion::SemanticChunk;
use log::{info, warn};
use std::collections::HashSet;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct LearningEngine {
    pub frozen: bool,
    pub last_consolidation: u64,
    pub consolidation_interval: u64, // seconds
}

impl LearningEngine {
    pub fn new() -> Self {
        Self {
            frozen: false,
            last_consolidation: SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs(),
            consolidation_interval: 300, // 5 minutes default
        }
    }

    pub fn learn(&mut self, core: &mut CognitionCore, chunks: Vec<SemanticChunk>) {
        if self.frozen {
            log::error!("Attempted to learn while frozen!");
            return;
        }

        for chunk in chunks {
            info!("Learning chunk from {}", chunk.source);
            core.learn_text(&chunk.content);
        }

        // Incremental Consolidation Check
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
        if now - self.last_consolidation > self.consolidation_interval {
            self.consolidate(core);
            self.last_consolidation = now;
        }
    }

    pub fn consolidate(&self, core: &mut CognitionCore) {
        if self.frozen { return; }
        info!("Consolidating memory... (Deduplication, Decay, Pruning)");

        // 1. Deduplicate relation graph values
        for (_subject, relations) in core.relation_graph.iter_mut() {
            let unique: HashSet<_> = relations.drain(..).collect();
            *relations = unique.into_iter().collect();
        }

        // 2. Soft Cap & Decay Governance
        // In a real system, we would track access timestamps per relation.
        // For v5.1, we enforce a strict cap.
        let max_rels_per_concept = 100;
        let mut pruned_count = 0;
        for (subject, relations) in core.relation_graph.iter_mut() {
            if relations.len() > max_rels_per_concept {
                // Decay strategy: Keep the newest/strongest.
                // Since we don't track strength yet, truncate.
                // Improvement: Randomly drop excess to simulate decay of weak links?
                // Deterministic truncation is safer for now.
                relations.truncate(max_rels_per_concept);
                pruned_count += 1;
            }
        }
        if pruned_count > 0 {
            warn!("Pruned {} overgrown concepts during consolidation.", pruned_count);
        }
    }

    pub fn freeze(&mut self) {
        info!("Freezing Learning Engine. No further updates allowed.");
        self.frozen = true;
    }

    pub fn unfreeze(&mut self) {
         // Explicit override if needed, though typically one-way.
         info!("Unfreezing Learning Engine. Updates allowed.");
         self.frozen = false;
    }
}
