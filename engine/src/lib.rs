use std::path::PathBuf;
use memory::MemoryManager;
use log::{info, debug};
use ingestion::ingest_graph;
use core_vsa::HyperVector;
use core_vsa::traits::MemoryStore;
use runtime::HardwareAdapter;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use cognition::CognitionCore;
use cognition::inference::{UncertaintyScorer, ReasoningValidator};
use ingestion::nlp::SymbolicNLP;
use crate::governance::SelfCorrectionLoop;
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

pub enum LifecycleState {
    Forge(MemoryManager, CognitionCore),
    Runtime(MemoryManager, MemoryManager, CognitionCore),
}

pub struct OmniMind {
    pub state: LifecycleState,
    pub metrics: metrics::RuntimeMetrics,
    pub vsa_dimension: usize,
}

impl OmniMind {
    pub fn new_forge(path: &str) -> Self {
        info!("Initializing OmniMind in FORGE mode at {}", path);
        let mut memory = MemoryManager::new(&PathBuf::from(path));
        let cognition = CognitionCore::new();

        let adapter = HardwareAdapter::new();
        adapter.report();
        let vsa_dimension = adapter.suggest_dimension();

        if adapter.is_low_memory_mode() {
            memory.set_low_memory_mode(true);
        }

        Self {
            state: LifecycleState::Forge(memory, cognition),
            metrics: metrics::RuntimeMetrics::new(),
            vsa_dimension,
        }
    }

    pub fn new_runtime(base_path: &str, delta_path: &str) -> Self {
        info!("Initializing OmniMind in RUNTIME mode.");
        let mut base_mem = MemoryManager::new(&PathBuf::from(base_path));
        let mut delta_mem = MemoryManager::new(&PathBuf::from(delta_path));
        let cognition = CognitionCore::new();

        let adapter = HardwareAdapter::new();
        let vsa_dimension = adapter.suggest_dimension();

        if adapter.is_low_memory_mode() {
            base_mem.set_low_memory_mode(true);
            delta_mem.set_low_memory_mode(true);
        }

        Self {
            state: LifecycleState::Runtime(base_mem, delta_mem, cognition),
            metrics: metrics::RuntimeMetrics::new(),
            vsa_dimension,
        }
    }

    pub fn ingest_file(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        info!("API: Ingesting file and extracting proper meaning: {}", path);
        let graph = ingest_graph(&std::path::Path::new(path)).map_err(|e| e.to_string())?;

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
            let mut hasher = DefaultHasher::new();
            word.hash(&mut hasher);
            let v = HyperVector::deterministic_dim(hasher.finish(), self.vsa_dimension);
            match result {
                None => result = Some(v),
                Some(r) => result = Some(r.bundle(&v)),
            }
        }
        result.unwrap()
    }

    pub fn learn(&mut self, text: &str) {
        debug!("OmniMind: Incremental learning with deep NLP extraction");
        let vector = self.encode_text(text);

        // Proper Meaning Extraction
        let facts = SymbolicNLP::extract_deep_facts(text);

        match &mut self.state {
            LifecycleState::Forge(mem, cog) => {
                let _ = mem.store(text, vector);
                for fact in facts {
                    cog.add_fact_triple(fact, 1.0);
                }
                cog.reinforce_knowledge(text, 1.0);
            },
            LifecycleState::Runtime(_, delta, cog) => {
                let _ = delta.store(text, vector);
                for fact in facts {
                    cog.add_fact_triple(fact, 1.0);
                }
                cog.reinforce_knowledge(text, 1.0);
            }
        }
    }

    pub fn ask(&mut self, text: &str) -> String {
        let words: Vec<String> = text.split_whitespace()
            .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()).to_lowercase())
            .filter(|w| !w.is_empty())
            .collect();

        let cog = match &self.state {
            LifecycleState::Forge(_, c) => c,
            LifecycleState::Runtime(_, _, c) => c,
        };

        for s_candidate in &words {
            if let Some(relations) = cog.relation_graph.get(s_candidate) {
                for rel in relations {
                    let intermediate = &rel.target;
                    if let Some(next_rels) = cog.relation_graph.get(intermediate) {
                        for next_rel in next_rels {
                            if words.contains(&next_rel.target) {
                                if ReasoningValidator::validate_inference(cog, s_candidate, &next_rel.target) {
                                    let uncertainty = UncertaintyScorer::calculate_industrial_uncertainty(next_rel.confidence, rel.weight, 0.1);
                                    return format!("Logic: Yes, {} related to {} (via {}). [Uncertainty: {:.2}]", s_candidate, next_rel.target, intermediate, uncertainty);
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
                         let uncertainty = UncertaintyScorer::calculate_industrial_uncertainty(rel.confidence, 1.0, 0.05);
                         return format!("Logic: {} is related to {}. [Uncertainty: {:.2}]", s_candidate, rel.target, uncertainty);
                     }
                 }
             }
        }

        let vector = self.encode_text(text);
        let results = self.query(&vector);
        if let Some((top, sim)) = results.first() {
            if *sim > 0.3 {
                let uncertainty = UncertaintyScorer::calculate_industrial_uncertainty(0.8, *sim, 0.2);
                format!("Logic: Match found. Result: {} (sim: {:.2}) [Uncertainty: {:.2}]", top, sim, uncertainty)
            } else {
                "No match found.".to_string()
            }
        } else {
            "No match found.".to_string()
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

    pub fn save(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
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
