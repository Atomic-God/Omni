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

        // Scheduler check
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
        if now - self.last_consolidation > self.consolidation_interval {
            self.consolidate(core);
            self.last_consolidation = now;
        }
    }

    pub fn consolidate(&self, core: &mut CognitionCore) {
        if self.frozen { return; }
        info!("Consolidating memory... (Deduplication & pruning)");

        // 1. Deduplicate relation graph values
        let mut total_rels = 0;
        for (_subject, relations) in core.relation_graph.iter_mut() {
            let unique: HashSet<_> = relations.drain(..).collect();
            *relations = unique.into_iter().collect();
            total_rels += relations.len();
        }

        // 2. Soft Cap Governance
        // If relations explode, prune rare ones (Placeholder logic: remove if list > 100)
        // In a real system, we'd track access counts.
        let max_rels_per_concept = 100;
        for (subject, relations) in core.relation_graph.iter_mut() {
            if relations.len() > max_rels_per_concept {
                warn!("Pruning overgrown concept: {}", subject);
                relations.truncate(max_rels_per_concept);
            }
        }
    }

    pub fn freeze(&mut self) {
        info!("Freezing Learning Engine. No further updates allowed.");
        self.frozen = true;
    }
}
