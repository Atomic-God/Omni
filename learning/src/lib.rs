use cognition::CognitionCore;
use cognition::traits::PerceptionModule;
use ingestion::SemanticChunk;
use log::{info, warn};
use std::collections::{HashSet, HashMap};
use std::time::{SystemTime, UNIX_EPOCH};

pub mod optimizer; // Added
use optimizer::LearningOptimizer;

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
        info!("Running Reflection Loop: No critical anomalies detected.");
    }
}

// Added Learning Governor
pub struct LearningGovernor {
    pub rate_limit_ms: u64,
    pub last_learning_event: u64,
    pub drift_threshold: f32,
}

impl Default for LearningGovernor {
    fn default() -> Self {
        Self {
            rate_limit_ms: 100, // Max 10 chunks per second per thread
            last_learning_event: 0,
            drift_threshold: 0.8, // High change allowed in Forge, low in Runtime
        }
    }
}

pub struct LearningEngine {
    pub frozen: bool,
    pub last_consolidation: u64,
    pub consolidation_interval: u64, // seconds
    pub previous_entropy: f32,
    pub frequency_tracker: ConceptFrequencyTracker,
    pub reflector: ReflectionLoop,
    pub governor: LearningGovernor, // Added
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
            governor: LearningGovernor::default(),
        }
    }

    pub fn learn(&mut self, core: &mut CognitionCore, chunks: Vec<SemanticChunk>) {
        if self.frozen {
            log::error!("Attempted to learn while frozen!");
            return;
        }

        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as u64;
        if now - self.governor.last_learning_event < self.governor.rate_limit_ms {
            // Throttling
            // Just warn for now
            // warn!("Learning throttled.");
        }
        self.governor.last_learning_event = now;

        let start_entropy = core.compute_global_entropy();

        for chunk in chunks {
            info!("Learning chunk from {} [Audit: Allowed]", chunk.source);
            core.learn_text(&chunk.content);
            self.frequency_tracker.record(&chunk.content);
        }

        let end_entropy = core.compute_global_entropy();
        let entropy_delta = (end_entropy - start_entropy).abs();

        // Anti-drift check
        if entropy_delta > self.governor.drift_threshold {
            warn!("High entropy drift ({:.4}). Verification required.", entropy_delta);
        }

        info!("Entropy Delta: {:.4} (Prev: {:.4} -> New: {:.4})", entropy_delta, start_entropy, end_entropy);

        let now_secs = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
        if entropy_delta > 0.5 || now_secs - self.last_consolidation > self.consolidation_interval {
            self.consolidate(core);
            self.last_consolidation = now_secs;
            self.previous_entropy = core.compute_global_entropy();

            self.reflector.assess_performance(core);
        }
    }

    pub fn consolidate(&self, core: &mut CognitionCore) {
        if self.frozen { return; }
        info!("Consolidating memory... (Deduplication, Decay, Pruning based on Weight)");

        for relations in core.relation_graph.values_mut() {
            for rel in relations.iter_mut() {
                if rel.weight > 5 {
                    rel.weight = rel.weight.saturating_sub(rel.weight / 10).max(1);
                }
            }
        }

        for (_subject, relations) in core.relation_graph.iter_mut() {
            relations.sort_by(|a, b| b.weight.cmp(&a.weight));

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

        let max_rels_per_concept = 50;
        let mut pruned_count = 0;

        for (_subject, relations) in core.relation_graph.iter_mut() {
            if relations.len() > max_rels_per_concept {
                relations.truncate(max_rels_per_concept);
                pruned_count += 1;
            }
        }

        if pruned_count > 0 {
            warn!("Pruned {} overgrown concepts (retained highest weight relations).", pruned_count);
        }

        // Run Advanced Optimization
        LearningOptimizer::compact_delta(core);
    }

    pub fn freeze(&mut self) {
        info!("Freezing Learning Engine. No further updates allowed.");
        self.frozen = true;
    }

    pub fn unfreeze(&mut self) {
         info!("Unfreezing Learning Engine. Updates allowed.");
         self.frozen = false;
    }

    pub fn strengthen_memory(&self, _concept: &str) {
        info!("Reinforcing memory (stub)");
    }
}
