use log::{info, warn, debug};
use crate::OmniMind;
use cognition::knowledge::{ReasoningEngine};

pub struct SelfCorrectionLoop;

impl SelfCorrectionLoop {
    /// Runs a self-correction pass on the mind's current knowledge base.
    pub fn verify_and_correct(mind: &mut OmniMind) {
        info!("Governance: Starting Self-Correction Loop...");

        let contradictions_resolved;

        match &mut mind.state {
            crate::LifecycleState::Forge(_mem, cog) => {
                cog.knowledge_graph.resolve_all_contradictions();
                contradictions_resolved = cog.knowledge_graph.contradictions.len();
            }
            crate::LifecycleState::Runtime(_base, _delta, cog) => {
                cog.knowledge_graph.resolve_all_contradictions();
                contradictions_resolved = cog.knowledge_graph.contradictions.len();
            }
        }

        if contradictions_resolved > 0 {
            warn!("Governance: Resolved {} knowledge contradictions.", contradictions_resolved);
        }

        info!("Governance: Self-Correction Loop complete.");
    }
}

pub struct SelfVerificationLoop;

impl SelfVerificationLoop {
    /// Verifies all facts in the knowledge graph for internal consistency using transitive inference.
    pub fn global_verification(mind: &mut OmniMind) {
        info!("Governance: Starting Global Self-Verification...");

        let mut inconsistencies = 0;

        // This is an intensive process, ideally run as a background task
        match &mut mind.state {
            crate::LifecycleState::Forge(_mem, cog) => {
                let facts: Vec<_> = cog.knowledge_graph.facts.values().cloned().collect();
                for fact in facts {
                    if let Some(ref triple) = fact.triple {
                        let (is_valid, _confidence) = ReasoningEngine::verify_fact(&cog.knowledge_graph, triple);
                        if !is_valid {
                            debug!("Inconsistency found: {:?}", triple);
                            inconsistencies += 1;
                        }
                    }
                }
            }
            _ => {}
        }

        if inconsistencies > 0 {
            warn!("Governance: Global verification found {} potential inconsistencies.", inconsistencies);
        }

        info!("Governance: Global verification complete.");
    }
}

pub struct AuditLog {
    pub entries: Vec<String>,
}

impl AuditLog {
    pub fn new() -> Self {
        Self { entries: Vec::new() }
    }

    pub fn log(&mut self, action: &str) {
        let entry = format!("[Action] {}", action);
        self.entries.push(entry);
    }
}
