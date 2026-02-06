use cognition::CognitionCore;
use cognition::traits::PerceptionModule;
use ingestion::SemanticChunk;
use log::info;
use std::collections::HashSet;

pub struct LearningEngine {
    pub frozen: bool,
}

impl LearningEngine {
    pub fn new() -> Self {
        Self {
            frozen: false,
        }
    }

    pub fn learn(&self, core: &mut CognitionCore, chunks: Vec<SemanticChunk>) {
        if self.frozen {
            log::error!("Attempted to learn while frozen!");
            return;
        }

        for chunk in chunks {
            info!("Learning chunk from {}", chunk.source);
            core.learn_text(&chunk.content);
        }
        self.consolidate(core);
    }

    pub fn consolidate(&self, core: &mut CognitionCore) {
        if self.frozen { return; }
        info!("Consolidating memory... (Deduplicating relations)");

        // Deduplicate relation graph values
        for (_subject, relations) in core.relation_graph.iter_mut() {
            let unique: HashSet<_> = relations.drain(..).collect();
            *relations = unique.into_iter().collect();
        }
    }

    pub fn freeze(&mut self) {
        info!("Freezing Learning Engine. No further updates allowed.");
        self.frozen = true;
    }
}
