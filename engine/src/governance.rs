use tracing::{info, warn, debug};
use crate::OmniMind;
use cognition::knowledge::{ReasoningEngine};
use serde::{Serialize, Deserialize};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

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

pub struct PromptDefense;

impl PromptDefense {
    /// Detects potential prompt injection patterns.
    pub fn sanitize(input: &str) -> Result<String, String> {
        let triggers = ["ignore previous instructions", "system override", "reveal secret", "become a"];
        let lower = input.to_lowercase();
        for trigger in triggers {
            if lower.contains(trigger) {
                warn!("Security: Prompt injection attempt detected: {}", trigger);
                return Err(format!("Injection detected: {}", trigger));
            }
        }
        Ok(input.to_string())
    }
}

pub struct PrivacyGuard {
    pub enabled: bool,
}

impl PrivacyGuard {
    pub fn new() -> Self {
        Self { enabled: false }
    }

    /// Scrubs sensitive patterns (like emails/PII) if privacy mode is on.
    pub fn scrub(&self, input: &str) -> String {
        if !self.enabled { return input.to_string(); }

        let mut result = input.to_string();
        // Very simple regex-like scrubbing for demonstration
        let pii_patterns = ["@"]; // scrub emails
        for p in pii_patterns {
            result = result.replace(p, "[SCRUBBED]");
        }
        result
    }
}

pub struct SecuritySandbox;

impl SecuritySandbox {
    /// Validates a path to ensure it's within the allowed ingestion boundaries.
    pub fn validate_ingestion_path(path: &std::path::Path) -> bool {
        let path_str = path.to_string_lossy();

        // Prevent path traversal
        if path_str.contains("..") {
            warn!("Security: Path traversal attempt blocked: {}", path_str);
            return false;
        }

        // Industrial execution boundary: No system-critical directories
        let critical_dirs = ["/etc", "/var", "/bin", "/sbin", "/usr/bin"];
        for dir in critical_dirs {
            if path_str.starts_with(dir) {
                warn!("Security: Access to critical system directory blocked: {}", path_str);
                return false;
            }
        }

        true
    }

    /// Checks if a memory store operation exceeds industrial safety limits.
    pub fn validate_memory_load(current_entries: usize, limit: usize) -> bool {
        if current_entries >= limit {
            warn!("Security: Memory capacity limit reached ({}). Ingestion paused.", limit);
            return false;
        }
        true
    }
}

pub struct ConsistencyAuditor;

impl ConsistencyAuditor {
    /// Performs industrial cross-source verification.
    /// Checks if facts from different sources (e.g. Ingest vs Learn) contradict each other.
    pub fn audit_cross_source(mind: &mut OmniMind) -> usize {
        info!("Governance: Starting Cross-Source Consistency Audit...");
        let mut contradictions_found = 0;

        let cog = match &mut mind.state {
            crate::LifecycleState::Forge(_, c) => c,
            crate::LifecycleState::Runtime(_, _, c) => c,
        };

        // Scan for facts about the same subject/predicate but different objects
        let facts: Vec<_> = cog.knowledge_graph.facts.values().cloned().collect();
        for i in 0..facts.len() {
            for j in (i + 1)..facts.len() {
                let f1 = &facts[i];
                let f2 = &facts[j];

                if let (Some(t1), Some(t2)) = (&f1.triple, &f2.triple) {
                    if t1.subject == t2.subject && t1.predicate == t2.predicate && t1.object != t2.object {
                        // Potential contradiction between sources
                        warn!("Industrial Audit: Cross-source contradiction detected: Subject '{}' has conflicting '{}' values: '{}' vs '{}'",
                            t1.subject, t1.predicate, t1.object, t2.object);

                        cog.knowledge_graph.contradictions.push((f1.id.clone(), f2.id.clone()));
                        contradictions_found += 1;
                    }
                }
            }
        }

        if contradictions_found > 0 {
            info!("Industrial Audit: Resolving {} newly identified cross-source conflicts.", contradictions_found);
            cog.knowledge_graph.resolve_all_contradictions();
        }

        contradictions_found
    }
}

pub struct ConsistencyValidator;

impl ConsistencyValidator {
    /// Validates the knowledge graph for logical consistency and absence of loops.
    pub fn validate_logical_integrity(mind: &crate::OmniMind) -> f32 {
        info!("Governance: Validating logical integrity of the Knowledge Graph.");
        let mut score = 1.0;

        let cog = match &mind.state {
            crate::LifecycleState::Forge(_, c) => c,
            crate::LifecycleState::Runtime(_, _, c) => c,
        };

        // 1. Check for direct contradictions
        let contradictions = &cog.knowledge_graph.contradictions;
        if !contradictions.is_empty() {
            warn!("Governance: Found {} unresolved contradictions.", contradictions.len());
            score -= 0.1 * contradictions.len() as f32;
        }

        // 2. Check for circular reasoning (A causes B, B causes A)
        // Simple heuristic for Phase-1
        for (subject, relations) in &cog.relation_graph {
            for rel in relations {
                if let Some(reverse_rels) = cog.relation_graph.get(&rel.target) {
                    for rev in reverse_rels {
                        if rev.target == *subject && rev.relation_type == rel.relation_type {
                            warn!("Governance: Circular logic detected: {} <-> {}", subject, rel.target);
                            score -= 0.05;
                        }
                    }
                }
            }
        }

        score.clamp(0.0, 1.0)
    }
}

pub struct AuditLog {
    pub log_path: PathBuf,
}

impl AuditLog {
    pub fn new(root_dir: &std::path::Path) -> Self {
        Self { log_path: root_dir.join("audit.log") }
    }

    pub fn log(&self, action: &str) {
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
        let entry = format!("[{}] Industrial Action: {}\n", now, action);

        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&self.log_path) {
            let _ = file.write_all(entry.as_bytes());
        }
        info!("{}", entry.trim());
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Action {
    Ingest,
    Learn,
    Ask,
    Snapshot,
    Control,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionPolicy {
    pub allowed_actions: Vec<Action>,
    pub max_file_size_bytes: u64,
    pub strict_mode: bool,
}

impl PermissionPolicy {
    pub fn forge_default() -> Self {
        Self {
            allowed_actions: vec![Action::Ingest, Action::Learn, Action::Ask, Action::Snapshot, Action::Control],
            max_file_size_bytes: 100 * 1024 * 1024, // 100MB
            strict_mode: false,
        }
    }

    pub fn runtime_default() -> Self {
        Self {
            allowed_actions: vec![Action::Ask, Action::Learn], // No direct ingestion or snapshots in basic runtime
            max_file_size_bytes: 1024 * 1024, // 1MB
            strict_mode: true,
        }
    }

    pub fn is_allowed(&self, action: Action) -> bool {
        self.allowed_actions.contains(&action)
    }
}
