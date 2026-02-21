#![deny(warnings)]
use std::path::PathBuf;
use memory::MemoryManager;
use tracing::{info, debug};
use core_vsa::HyperVector;
use core_vsa::traits::Ingestor;
use core_vsa::traits::MemoryStore;
use runtime::HardwareAdapter;
pub use cognition::{CognitionCore, ReasoningTrace};
use cognition::inference::{UncertaintyScorer, ReasoningValidator};
use ingestion::nlp::SymbolicNLP;
use serde::{Serialize, Deserialize};
use crate::governance::{SelfCorrectionLoop, SecuritySandbox, PrivacyGuard, PromptDefense, PermissionPolicy, AuditLog, Action};
use crate::task::TaskLoop;
use tracing::warn;
use std::fs::File;
use std::io::{BufReader, BufWriter};

pub mod adapter;
pub mod metrics;
pub mod explanation;
pub mod context;
pub mod budget;
pub mod governance;
pub mod bench;
pub mod learning;
pub mod task;

pub enum LifecycleState {
    Forge(MemoryManager, CognitionCore),
    Runtime(MemoryManager, MemoryManager, CognitionCore),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResponse {
    pub answer: String,
    pub trace: Option<ReasoningTrace>,
}

pub struct OmniMind {
    pub state: LifecycleState,
    pub metrics: metrics::RuntimeMetrics,
    pub vsa_dimension: usize,
    pub privacy: PrivacyGuard,
    pub task_loop: TaskLoop,
    pub permissions: PermissionPolicy,
    pub audit_log: AuditLog,
    pub adapter: HardwareAdapter,
}

impl OmniMind {
    pub fn new_forge(path: &str) -> Self {
        info!("Initializing OmniMind in FORGE mode at {}", path);
        let adapter = HardwareAdapter::new();
        adapter.report();
        let vsa_dimension = adapter.suggest_dimension();

        let mut memory = MemoryManager::with_dimension(&PathBuf::from(path), vsa_dimension);
        let cognition = CognitionCore::new();

        if adapter.is_low_memory_mode() {
            memory.set_low_memory_mode(true);
        }

        Self {
            state: LifecycleState::Forge(memory, cognition),
            metrics: metrics::RuntimeMetrics::new(),
            vsa_dimension,
            privacy: PrivacyGuard::new(),
            task_loop: TaskLoop::new(),
            permissions: PermissionPolicy::forge_default(),
            audit_log: AuditLog::new(&PathBuf::from(path)),
            adapter,
        }
    }

    pub fn new_runtime(base_path: &str, delta_path: &str) -> Self {
        info!("Initializing OmniMind in RUNTIME mode.");
        let adapter = HardwareAdapter::new();
        let vsa_dimension = adapter.suggest_dimension();

        let mut base_mem = MemoryManager::with_dimension(&PathBuf::from(base_path), vsa_dimension);
        let mut delta_mem = MemoryManager::with_dimension(&PathBuf::from(delta_path), vsa_dimension);
        let cognition = CognitionCore::new();

        if adapter.is_low_memory_mode() {
            base_mem.set_low_memory_mode(true);
            delta_mem.set_low_memory_mode(true);
        }

        Self {
            state: LifecycleState::Runtime(base_mem, delta_mem, cognition),
            metrics: metrics::RuntimeMetrics::new(),
            vsa_dimension,
            privacy: PrivacyGuard::new(),
            task_loop: TaskLoop::new(),
            permissions: PermissionPolicy::runtime_default(),
            audit_log: AuditLog::new(&PathBuf::from(delta_path)),
            adapter,
        }
    }

    pub fn ingest_file(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let p = std::path::Path::new(path);
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();

        if !self.permissions.is_allowed(Action::Ingest) {
            self.audit_log.log_event(crate::governance::AuditEntry {
                timestamp: now,
                action: Action::Ingest,
                resource: Some(path.to_string()),
                success: false,
                message: "Permission Denied: Ingestion blocked".into(),
                hardware_mode: format!("{:?}", self.adapter.current_mode),
            });
            return Err("Permission Denied: Ingestion blocked by industrial policy".into());
        }

        if !SecuritySandbox::validate_ingestion_path(p) {
             return Err("Security Violation: Path outside allowed boundary".into());
        }

        if let Err(e) = self.permissions.validate_file(p) {
            self.audit_log.log_event(crate::governance::AuditEntry {
                timestamp: now,
                action: Action::Ingest,
                resource: Some(path.to_string()),
                success: false,
                message: format!("Security Violation: {}", e),
                hardware_mode: format!("{:?}", self.adapter.current_mode),
            });
            return Err(e.into());
        }

        self.audit_log.log_event(crate::governance::AuditEntry {
            timestamp: now,
            action: Action::Ingest,
            resource: Some(path.to_string()),
            success: true,
            message: format!("Ingesting file: {}", path),
            hardware_mode: format!("{:?}", self.adapter.current_mode),
        });
        info!("API: Ingesting file and extracting proper meaning: {}", path);

        let ingestor = ingestion::UniversalIngestor::with_dimension(self.vsa_dimension);
        let graph = ingestor.ingest(p).map_err(|e| e.to_string())?;

        match &mut self.state {
            LifecycleState::Forge(mem, cog) => {
                cog.ingest_from_graph(&graph);
                for node in graph.nodes {
                    let _ = mem.store(node.id.as_str(), node.vector);
                }
            },
            LifecycleState::Runtime(_, delta, cog) => {
                cog.ingest_from_graph(&graph);
                for node in graph.nodes {
                    let _ = delta.store(node.id.as_str(), node.vector);
                }
            }
        }

        // Industrial Self-Correction after ingestion
        SelfCorrectionLoop::verify_and_correct(self);
        Ok(())
    }

    pub fn query(&self, concept: &HyperVector) -> Vec<(String, f32)> {
        match &self.state {
            LifecycleState::Forge(mem, _) => mem.query_nearest(concept, 5),
            LifecycleState::Runtime(base, delta, _) => {
                let mut results = delta.query_nearest(concept, 5);
                results.extend(base.query_nearest(concept, 5));
                results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
                results.truncate(5);
                results
            }
        }
    }

    pub fn encode_text(&self, text: &str) -> HyperVector {
        let mut words: Vec<String> = text.split_whitespace()
            .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()).to_lowercase())
            .filter(|w| !w.is_empty())
            .collect();

        if words.is_empty() {
            return HyperVector::deterministic_dim(0, self.vsa_dimension);
        }

        words.sort();

        let mut result = None;
        for word in words {
            let h = seahash::hash(word.as_bytes());
            let v = HyperVector::deterministic_dim(h, self.vsa_dimension);
            match result {
                None => result = Some(v),
                Some(r) => result = Some(r.bundle(&v)),
            }
        }
        result.unwrap()
    }

    pub fn learn(&mut self, text: &str) {
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();

        if !self.permissions.is_allowed(Action::Learn) {
            self.audit_log.log_event(crate::governance::AuditEntry {
                timestamp: now,
                action: Action::Learn,
                resource: None,
                success: false,
                message: "Permission Denied: Learning blocked".into(),
                hardware_mode: format!("{:?}", self.adapter.current_mode),
            });
            warn!("Permission Denied: Learning blocked by policy");
            return;
        }

        let text = match PromptDefense::sanitize(text) {
            Ok(t) => t,
            Err(e) => {
                warn!("Defense: Blocked learning due to injection: {}", e);
                return;
            }
        };

        let text = self.privacy.scrub(&text);

        self.audit_log.log_event(crate::governance::AuditEntry {
            timestamp: now,
            action: Action::Learn,
            resource: None,
            success: true,
            message: "Incremental Learning Event".into(),
            hardware_mode: format!("{:?}", self.adapter.current_mode),
        });
        debug!("OmniMind: Incremental learning with deep NLP extraction");
        let vector = self.encode_text(&text);

        // Proper Meaning Extraction
        let facts = SymbolicNLP::extract_deep_facts(&text);

        match &mut self.state {
            LifecycleState::Forge(mem, cog) => {
                let _ = mem.store(&text, vector);
                for fact in facts {
                    cog.add_fact_triple(fact, 1.0);
                }
                cog.reinforce_knowledge(&text, 1.0);
            },
            LifecycleState::Runtime(_, delta, cog) => {
                let _ = delta.store(&text, vector);
                for fact in facts {
                    cog.add_fact_triple(fact, 1.0);
                }
                cog.reinforce_knowledge(&text, 1.0);
            }
        }
    }

    pub fn ask(&mut self, text: &str) -> QueryResponse {
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();

        if !self.permissions.is_allowed(Action::Ask) {
            self.audit_log.log_event(crate::governance::AuditEntry {
                timestamp: now,
                action: Action::Ask,
                resource: None,
                success: false,
                message: "Denied Query attempt".into(),
                hardware_mode: format!("{:?}", self.adapter.current_mode),
            });
            return QueryResponse {
                answer: "Permission Denied: Query blocked by policy".to_string(),
                trace: None,
            };
        }

        let text = match PromptDefense::sanitize(text) {
            Ok(t) => t,
            Err(e) => {
                return QueryResponse {
                    answer: format!("Security Block: {}", e),
                    trace: None,
                };
            }
        };

        let text = self.privacy.scrub(&text);
        self.audit_log.log_event(crate::governance::AuditEntry {
            timestamp: now,
            action: Action::Ask,
            resource: Some(text.clone()),
            success: true,
            message: format!("Query Event: {}", text),
            hardware_mode: format!("{:?}", self.adapter.current_mode),
        });

        let words: Vec<String> = text.split_whitespace()
            .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()).to_lowercase())
            .filter(|w| !w.is_empty())
            .collect();

        let cog = match &self.state {
            LifecycleState::Forge(_, c) => c,
            LifecycleState::Runtime(_, _, c) => c,
        };

        // Industrial: Check for most recent truth first
        for s_candidate in &words {
            for p_candidate in &words {
                if let Some(fact) = cog.knowledge_graph.get_recent_truth(s_candidate, p_candidate) {
                    if let Some(ref t) = fact.triple {
                        let composite_conf = fact.get_composite_confidence();
                        let uncertainty = UncertaintyScorer::calculate_industrial_uncertainty(composite_conf, 1.0, 0.05);

                        let history_str = if fact.history.len() > 1 {
                            format!(" (Previous values: {})", fact.history.iter().map(|h| h.value.clone()).collect::<Vec<_>>().join(", "))
                        } else {
                            "".to_string()
                        };

                        return QueryResponse {
                            answer: format!("Recent Truth: {} {} is {}. [Uncertainty: {:.2}]{}", t.subject, t.predicate, t.object, uncertainty, history_str),
                            trace: None,
                        };
                    }
                }
            }
        }

        for s_candidate in &words {
            if let Some(relations) = cog.relation_graph.get(s_candidate) {
                for rel in relations {
                    let intermediate = &rel.target;
                    if let Some(next_rels) = cog.relation_graph.get(intermediate) {
                        for next_rel in next_rels {
                            if words.contains(&next_rel.target) {
                                if ReasoningValidator::validate_inference(cog, s_candidate, &next_rel.target) {
                                    let trace = cog.find_path(s_candidate, &next_rel.target, 3);
                                    let uncertainty = UncertaintyScorer::calculate_industrial_uncertainty(next_rel.confidence, rel.weight, 0.1);
                                    return QueryResponse {
                                        answer: format!("Logic: Yes, {} related to {} (via {}). [Uncertainty: {:.2}]", s_candidate, next_rel.target, intermediate, uncertainty),
                                        trace,
                                    };
                                } else {
                                    info!("Validation Loop: Blocked contradictory inference {} -> {}", s_candidate, next_rel.target);
                                }
                            }
                        }
                    }
                }
            }
        }

        for s_candidate in &words {
             if let Some(relations) = cog.relation_graph.get(s_candidate) {
                 for rel in relations {
                     if words.contains(&rel.target) || words.contains(&"what".to_string()) || words.contains(&"who".to_string()) {
                         let trace = cog.find_path(s_candidate, &rel.target, 2);
                         let uncertainty = UncertaintyScorer::calculate_industrial_uncertainty(rel.confidence, 1.0, 0.05);
                         return QueryResponse {
                             answer: format!("Logic: {} is related to {}. [Uncertainty: {:.2}]", s_candidate, rel.target, uncertainty),
                             trace,
                         };
                     }
                 }
             }
        }

        let vector = self.encode_text(&text);
        let results = self.query(&vector);
        if let Some((top, sim)) = results.first() {
            if *sim > 0.3 {
                let uncertainty = UncertaintyScorer::calculate_industrial_uncertainty(0.8, *sim, 0.2);
                QueryResponse {
                    answer: format!("Logic: Match found. Result: {} (sim: {:.2}) [Uncertainty: {:.2}]", top, sim, uncertainty),
                    trace: None,
                }
            } else {
                QueryResponse {
                    answer: "No match found.".to_string(),
                    trace: None,
                }
            }
        } else {
            QueryResponse {
                answer: "No match found.".to_string(),
                trace: None,
            }
        }
    }

    pub fn lifecycle_status(&self) -> String {
        match &self.state {
            LifecycleState::Forge(_, _) => "FORGE".to_string(),
            LifecycleState::Runtime(_, _, _) => "RUNTIME".to_string(),
        }
    }

    pub fn memory_stats(&self) -> String {
        match &self.state {
            LifecycleState::Forge(mem, _) => format!("Memory: {} entries", mem.metadata.len()),
            LifecycleState::Runtime(base, delta, _) => format!("Base: {}, Delta: {}", base.metadata.len(), delta.metadata.len()),
        }
    }

    pub fn update_metrics(&mut self) {
        self.metrics.hardware_snapshot = runtime::adaptation::PerformanceMonitor::capture_snapshot();

        // Industrial: Perform live hardware adaptation during metric updates
        self.adapter.live_adjust();
    }

    pub fn execute_task_step(&mut self) -> Option<String> {
        let cog = match &self.state {
            LifecycleState::Forge(_, c) => c,
            LifecycleState::Runtime(_, _, c) => c,
        };
        let uncertainty = UncertaintyScorer::estimate_entropy(cog);
        self.task_loop.step(uncertainty)
    }

    pub fn sleep_cycle(&mut self) {
        info!("OmniMind: Industrial Sleep Cycle starting.");
        match &mut self.state {
            LifecycleState::Forge(mem, cog) => {
                mem.sleep_cycle();
                cog.resolve_contradictions();
                cog.apply_knowledge_decay(0.98);

                // Industrial Self-Consistency Verification
                crate::governance::ConsistencyAuditor::audit_cross_source(self);
                crate::governance::SelfVerificationLoop::global_verification(self);
            },
            LifecycleState::Runtime(base, delta, cog) => {
                base.sleep_cycle();
                delta.sleep_cycle();
                cog.resolve_contradictions();
                cog.apply_knowledge_decay(0.98);

                crate::governance::ConsistencyAuditor::audit_cross_source(self);
                crate::governance::SelfVerificationLoop::global_verification(self);
            }
        }
        info!("OmniMind: Sleep Cycle complete.");
    }

    pub fn save(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();

        if !self.permissions.is_allowed(Action::Snapshot) {
            self.audit_log.log_event(crate::governance::AuditEntry {
                timestamp: now,
                action: Action::Snapshot,
                resource: Some(path.to_string()),
                success: false,
                message: "Denied Snapshot attempt".into(),
                hardware_mode: format!("{:?}", self.adapter.current_mode),
            });
            return Err("Permission Denied: Snapshot saving blocked by policy".into());
        }

        self.audit_log.log_event(crate::governance::AuditEntry {
            timestamp: now,
            action: Action::Snapshot,
            resource: Some(path.to_string()),
            success: true,
            message: format!("Saving state to: {}", path),
            hardware_mode: format!("{:?}", self.adapter.current_mode),
        });
        let cog_path = format!("{}.cog", path);
        match &self.state {
            LifecycleState::Forge(mem, cog) => {
                mem.save_snapshot(path)?;
                let file = File::create(cog_path)?;
                bincode::serialize_into(BufWriter::new(file), cog)?;
            },
            LifecycleState::Runtime(_, delta, cog) => {
                delta.save_snapshot(path)?;
                let file = File::create(cog_path)?;
                bincode::serialize_into(BufWriter::new(file), cog)?;
            }
        }
        Ok(())
    }

    pub fn load(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let cog_path = format!("{}.cog", path);
        match &mut self.state {
            LifecycleState::Forge(mem, cog) => {
                mem.load_snapshot(path)?;
                if std::path::Path::new(&cog_path).exists() {
                    let file = File::open(cog_path)?;
                    *cog = bincode::deserialize_from(BufReader::new(file))?;
                }
            },
            LifecycleState::Runtime(base, _, cog) => {
                base.load_snapshot(path)?;
                if std::path::Path::new(&cog_path).exists() {
                    let file = File::open(cog_path)?;
                    *cog = bincode::deserialize_from(BufReader::new(file))?;
                }
            }
        }
        Ok(())
    }
}
