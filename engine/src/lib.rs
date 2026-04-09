#![deny(warnings)]
use std::path::PathBuf;
use memory::MemoryManager;
use tracing::{info, debug};
use core_vsa::HyperVector;
use core_vsa::traits::Ingestor;
use core_vsa::traits::MemoryStore;
use runtime::HardwareAdapter;
pub use cognition::{CognitionCore, ReasoningTrace};
use cognition::knowledge::IndustrialGraph;
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
        let _prof = runtime::ProfileScope::new("OmniMind::ingest_file");
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

        let cog = match &self.state {
            LifecycleState::Forge(_, c) => c,
            LifecycleState::Runtime(_, _, c) => c,
        };

        // Industrial: Cross-language symbol mapping for VSA encoding
        let kg = cog.knowledge_graph.clone();
        let mapper = std::sync::Arc::new(move |term: &str| {
            cognition::concepts::MultilingualConceptLinker::get_canonical_term(&kg, term)
        });

        let ingestor = ingestion::UniversalIngestor::with_dimension(self.vsa_dimension)
            .with_mapper(mapper);

        let mut graph = ingestor.ingest(p).map_err(|e| e.to_string())?;

        // Industrial: Cross-language symbol mapping for ingested nodes
        for node in &mut graph.nodes {
            if node.id.starts_with("chunk:") {
                // If it's a text chunk, we might want to re-encode, but for Phase-1
                // we'll just ensure the KG facts extracted from it are mapped.
            }
        }

        match &mut self.state {
            LifecycleState::Forge(mem, cog) => {
                for edge in &graph.edges {
                    let triple = core_vsa::FactTriple {
                        subject: edge.source.clone(),
                        predicate: edge.relation.clone(),
                        object: edge.target.clone(),
                    };
                    let (valid, _conf) = cog.knowledge_graph.validate_fact(&triple);
                    if !valid {
                        warn!("Industrial Validation: Skipping contradictory fact: {:?}", triple);
                        continue;
                    }
                    cog.add_fact_triple(triple, edge.confidence);
                }
                for node in graph.nodes {
                    let _ = mem.store(node.id.as_str(), node.vector);
                }
            },
            LifecycleState::Runtime(_, delta, cog) => {
                for edge in &graph.edges {
                    let triple = core_vsa::FactTriple {
                        subject: edge.source.clone(),
                        predicate: edge.relation.clone(),
                        object: edge.target.clone(),
                    };
                    let (valid, _conf) = cog.knowledge_graph.validate_fact(&triple);
                    if !valid {
                        warn!("Industrial Validation: Skipping contradictory fact: {:?}", triple);
                        continue;
                    }
                    cog.add_fact_triple(triple, edge.confidence);
                }
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
        let mut results = match &self.state {
            LifecycleState::Forge(mem, _) => mem.query_nearest(concept, 10),
            LifecycleState::Runtime(base, delta, _) => {
                let mut res = delta.query_nearest(concept, 10);
                res.extend(base.query_nearest(concept, 10));
                res.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
                res
            }
        };

        // Industrial Relevance Weighting
        let context = self.task_loop.get_context_vector();
        crate::learning::ContinuousLearningEngine::weight_relevance(self, &mut results, &context);

        results.truncate(5);
        results
    }

    pub fn encode_text(&self, text: &str) -> HyperVector {
        let cog = match &self.state {
            LifecycleState::Forge(_, c) => c,
            LifecycleState::Runtime(_, _, c) => c,
        };

        let mut words: Vec<String> = text.split_whitespace()
            .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()).to_lowercase())
            .filter(|w| !w.is_empty())
            .map(|w| cognition::concepts::MultilingualConceptLinker::get_canonical_term(&cog.knowledge_graph, &w))
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
                    let (valid, _conf) = cog.knowledge_graph.validate_fact(&fact);
                    if valid {
                        cog.add_fact_triple(fact, 1.0);
                    } else {
                        warn!("Industrial Validation: learn() blocked fact: {:?}", fact);
                    }
                }
                cog.reinforce_knowledge(&text, 1.0);
            },
            LifecycleState::Runtime(_, delta, cog) => {
                let _ = delta.store(&text, vector);
                for fact in facts {
                    let (valid, _conf) = cog.knowledge_graph.validate_fact(&fact);
                    if valid {
                        cog.add_fact_triple(fact, 1.0);
                    } else {
                        warn!("Industrial Validation: learn() blocked fact: {:?}", fact);
                    }
                }
                cog.reinforce_knowledge(&text, 1.0);
            }
        }
    }

    pub fn ask(&mut self, text: &str) -> QueryResponse {
        let _prof = runtime::ProfileScope::new("OmniMind::ask");
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

        // 1. Multi-Hop reasoning across Knowledge Graph: Find unknown endpoints
        for s in &words {
            if s == "what" || s == "where" || s == "is" || s == "who" || s == "how" || s == "means" || s == "causes" || s == "inhibits" { continue; }

            // Try to find any significant multi-hop connection to something NOT in query
            for fact in cog.knowledge_graph.facts.values() {
                if let Some(ref t) = fact.triple {
                    let e = &t.object;
                    if words.contains(e) { continue; }

                    if let Some((path, conf)) = cognition::knowledge::ReasoningEngine::multi_hop_reason(&cog.knowledge_graph, s, e, 4) {
                        if path.len() > 2 && conf > 0.1 {
                            let trace = Some(ReasoningTrace { steps: path, final_confidence: conf });
                            let report = crate::explanation::ExplanationEngine::generate_industrial_report(cog, &text, &format!("Industrial Inference: Derived that {} is connected to {} via a {}-hop chain.", s, e, trace.as_ref().unwrap().steps.len()-1), trace.as_ref());
                            return QueryResponse {
                                answer: report,
                                trace,
                            };
                        }
                    }
                }
            }
        }

        // 2. Industrial Deep Reasoning: Check for Causal Chains
        for candidate in &words {
            if candidate == "what" || candidate == "is" || candidate == "who" || candidate == "how" || candidate == "means" || candidate == "causes" || candidate == "inhibits" { continue; }
            let causes = cog.identify_causal_chains(candidate);
            if !causes.is_empty() {
                let (top_cause, conf) = &causes[0];
                let _uncertainty = UncertaintyScorer::calculate_industrial_uncertainty(*conf, 0.9, 0.1);
                let trace = cog.find_path(top_cause.split(": ").last().unwrap_or(top_cause), candidate, 3);
                let report = crate::explanation::ExplanationEngine::generate_industrial_report(cog, &text, &format!("Causal Reasoning: I've identified that '{}' is a significant factor for '{}'.", top_cause, candidate), trace.as_ref());
                return QueryResponse {
                    answer: report,
                    trace,
                };
            }
        }

        // 2. Logic Reasoning: Check for multi-step relations
        let mut best_logic_res: Option<(QueryResponse, f32)> = None;

        for s_candidate in &words {
            if let Some(relations) = cog.relation_graph.get(s_candidate) {
                for rel in relations {
                    let intermediate = &rel.target;
                    if let Some(next_rels) = cog.relation_graph.get(intermediate) {
                        for next_rel in next_rels {
                            if words.contains(&next_rel.target) {
                                if ReasoningValidator::validate_inference(cog, s_candidate, &next_rel.target) {
                                    let trace = cog.find_path(s_candidate, &next_rel.target, 3);
                                    let report = crate::explanation::ExplanationEngine::generate_industrial_report(cog, &text, &format!("Logic: Yes, {} related to {} (via {}).", s_candidate, next_rel.target, intermediate), trace.as_ref());
                                    let res = QueryResponse {
                                        answer: report,
                                        trace,
                                    };
                                    if best_logic_res.as_ref().map_or(true, |(_, conf)| next_rel.confidence > *conf) {
                                        best_logic_res = Some((res, next_rel.confidence));
                                    }
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
                     if words.contains(&rel.target) || (words.contains(&"what".to_string()) && rel.target != "means" && rel.target != "causes" && rel.target != "inhibits") || words.contains(&"who".to_string()) {
                         // Industrial: Fetch latest composite confidence from Knowledge Graph to ensure Belief Revision is respected
                         let kg_conf = cog.knowledge_graph.facts.values()
                             .filter(|f| f.triple.as_ref().map_or(false, |t| t.subject == *s_candidate && t.object == rel.target))
                             .map(|f| f.get_composite_confidence())
                             .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                             .unwrap_or(rel.confidence);

                         let trace = cog.find_path(s_candidate, &rel.target, 2);
                         let report = crate::explanation::ExplanationEngine::generate_industrial_report(cog, &text, &format!("Logic: {} is related to {}.", s_candidate, rel.target), trace.as_ref());
                         let res = QueryResponse {
                             answer: report,
                             trace,
                         };
                         if best_logic_res.as_ref().map_or(true, |(_, conf)| kg_conf > *conf) {
                             best_logic_res = Some((res, kg_conf));
                         }
                     }
                 }
             }
        }
        if let Some((res, _)) = best_logic_res { return res; }

        // 3. Property Lookup: Check for most recent truth (Backwards compatibility)
        for s_candidate in &words {
            for word in &words {
                if word == "what" || word == "is" || word == "who" || word == "how" || word == "means" || word == "causes" { continue; }

                let fact_opt = cog.knowledge_graph.facts.values()
                    .filter(|f| f.triple.as_ref().map_or(false, |t| t.subject == *s_candidate && (t.predicate.contains(word) || t.object.contains(word))))
                    .filter(|f| f.get_composite_confidence() > 0.4)
                    .max_by(|a, b| a.get_composite_confidence().partial_cmp(&b.get_composite_confidence()).unwrap_or(std::cmp::Ordering::Equal));

                if let Some(fact) = fact_opt {
                    if let Some(ref t) = fact.triple {
                        let composite_conf = fact.get_composite_confidence();
                        let _uncertainty = UncertaintyScorer::calculate_industrial_uncertainty(composite_conf, 1.0, 0.05);

                        let history_str = if fact.history.len() > 1 {
                            format!(" (Previous values: {})", fact.history.iter().map(|h| h.value.clone()).collect::<Vec<_>>().join(", "))
                        } else {
                            "".to_string()
                        };

                        let trace = Some(ReasoningTrace {
                                steps: vec![format!("Source: Fact ID {}", fact.id), format!("Composite Confidence: {:.2}", composite_conf)],
                                final_confidence: composite_conf,
                            });
                         let report = crate::explanation::ExplanationEngine::generate_industrial_report(cog, &text, &format!("Recent Truth: {} {} is {}.{}", t.subject, t.predicate, t.object, history_str), trace.as_ref());
                        return QueryResponse {
                            answer: report,
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
        self.metrics.profiling_data = runtime::Profiler::get_averages();

        // Industrial: Perform live hardware adaptation during metric updates
        self.adapter.live_adjust();

        if self.adapter.is_low_memory_mode() {
            warn!("Industrial Recovery: Emergency Low Memory detected. Triggering urgent consolidation.");
            self.sleep_cycle();
        }

        // Point 5: Local Adaptation Loop - Adjust cognitive parameters
        self.run_local_adaptation();
    }

    fn run_local_adaptation(&mut self) {
        let cog = match &self.state {
            LifecycleState::Forge(_, c) => c,
            LifecycleState::Runtime(_, _, c) => c,
        };

        let entropy = cognition::inference::UncertaintyScorer::estimate_entropy(cog);

        // Adaptive Scaling: If entropy is high, increase consolidation frequency and decay
        if entropy > 0.6 {
            info!("Local Adaptation: High cognitive entropy ({:.2}). Accelerating consolidation logic.", entropy);
            // In a real system we might trigger a sleep cycle sooner.
            // Here we just log for industrial diagnostics.
        }
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
        let _prof = runtime::ProfileScope::new("OmniMind::sleep_cycle");
        info!("OmniMind: Industrial Sleep Cycle starting.");

        let (prototypes, cog_ref) = match &mut self.state {
            LifecycleState::Forge(mem, cog) => (mem.sleep_cycle(), cog),
            LifecycleState::Runtime(base, delta, cog) => {
                let mut p = base.sleep_cycle();
                p.extend(delta.sleep_cycle());
                (p, cog)
            }
        };

        // 1. Concept Evolution: Link prototypes in KG
        for (proto_key, _, members) in prototypes {
            for member in members {
                cog_ref.add_relation(&member, &proto_key, cognition::RelationType::Taxonomic, 1.0, 0.95);
            }
        }

        // 2. Concept Formation: Hierarchical Clustering (Topic -> Domain)
        let topics = cognition::concepts::ConceptFormationEngine::detect_topics(&cog_ref.knowledge_graph);
        for topic in &topics {
            info!("Concept Formation: Emerging Topic detected: {}", topic.label);
            cog_ref.add_relation(&topic.id, &topic.label, cognition::RelationType::Taxonomic, 1.0, 0.9);
            for fact_id in &topic.members {
                cog_ref.add_relation(fact_id, &topic.id, cognition::RelationType::Structural, 1.0, 0.8);
            }
        }

        let domains = cognition::concepts::ConceptFormationEngine::abstract_domains(&topics);
        for domain in domains {
            info!("Concept Formation: Emerging Domain detected: {}", domain.label);
            cog_ref.add_relation(&domain.id, &domain.label, cognition::RelationType::Taxonomic, 1.0, 0.95);
            for topic in domain.topics {
                cog_ref.add_relation(&topic.id, &domain.id, cognition::RelationType::Structural, 1.0, 0.9);
            }
        }

        cog_ref.resolve_contradictions();
        cog_ref.apply_knowledge_decay(0.98);

        // Continuous Learning: Autonomous Reinforcement
        crate::learning::ContinuousLearningEngine::autonomous_reinforcement(self);

        // Industrial Self-Consistency Verification
        crate::governance::ConsistencyAuditor::audit_cross_source(self);
        crate::governance::SelfVerificationLoop::global_verification(self);

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

    pub fn state_memory(&self) -> &MemoryManager {
        match &self.state {
            LifecycleState::Forge(mem, _) => mem,
            LifecycleState::Runtime(_, delta, _) => delta,
        }
    }

    pub fn state_memory_mut(&mut self) -> &mut MemoryManager {
        match &mut self.state {
            LifecycleState::Forge(mem, _) => mem,
            LifecycleState::Runtime(_, delta, _) => delta,
        }
    }

    pub fn describe_image_at_path(&self, path: &str) -> Result<Vec<(String, f32)>, Box<dyn std::error::Error>> {
        let img = image::open(path)?;
        let sv = multimodal::vision::VisualGrounding::analyze_image(&img, self.vsa_dimension);

        let _cog = match &self.state {
            LifecycleState::Forge(_, c) => c,
            LifecycleState::Runtime(_, _, c) => c,
        };

        // For Phase 1, we use a simple linear scan of text labels in memory
        // In industrial deployment, we would use LSH for this cross-modal lookup
        let mut results = Vec::new();
        let mem = self.state_memory();
        for (label, entry) in &mem.metadata {
            if entry.layer == "semantic" || entry.layer == "episodic" {
                let sim = sv.vector.similarity(&entry.vector);
                if sim > 0.25 {
                    results.push((label.clone(), sim));
                }
            }
        }

        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(5);
        Ok(results)
    }

    pub fn find_similar_sounds_at_path(&self, path: &str) -> Result<Vec<(String, f32)>, Box<dyn std::error::Error>> {
        let sv = multimodal::audio::AudioGrounding::analyze_audio(std::path::Path::new(path), self.vsa_dimension);

        let mut results = Vec::new();
        let mem = self.state_memory();
        for (label, entry) in &mem.metadata {
            if entry.layer == "semantic" || entry.layer == "episodic" {
                let sim = sv.vector.similarity(&entry.vector);
                if sim > 0.2 {
                    results.push((label.clone(), sim));
                }
            }
        }

        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(5);
        Ok(results)
    }
}
