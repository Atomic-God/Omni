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

        // 2. Conflict Resolution & Decay Governance
        // If a subject has contradictory relations or too many, we prioritize recent or frequent ones.
        // For v6.0, we simulate "reinforcement beats decay":
        // - Truncate list to soft cap (100).
        // - In future, we would sort by weight. Here, simple truncation assumes FIFO/append order is temporal.

        let max_rels_per_concept = 100;
        let mut pruned_count = 0;
        for (_subject, relations) in core.relation_graph.iter_mut() {
            if relations.len() > max_rels_per_concept {
                // Keep the *latest* added relations (tail of the vector)
                // relations is [oldest ... newest]
                let start_idx = relations.len() - max_rels_per_concept;
                *relations = relations.split_off(start_idx);
                pruned_count += 1;
            }
        }

        if pruned_count > 0 {
            warn!("Pruned {} overgrown concepts (retaining recent memories).", pruned_count);
        }
    }

    pub fn freeze(&mut self) {
        info!("Freezing Learning Engine. No further updates allowed.");
        self.frozen = true;
    }

    pub fn unfreeze(&mut self) {
         info!("Unfreezing Learning Engine. Updates allowed.");
         self.frozen = false;
    }
}
