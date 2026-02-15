use std::path::PathBuf;
use memory::MemoryManager;
use log::info;
use ingestion::ingest_graph;
use core_vsa::HyperVector;
use core_vsa::traits::MemoryStore;
use runtime::HardwareAdapter;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use cognition::{CognitionCore, RelationType};
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
        info!("API: Ingesting file {}", path);
        let graph = ingest_graph(&PathBuf::from(path)).map_err(|e| e.to_string())?;

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

    fn encode_text(&self, text: &str) -> HyperVector {
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
        let vector = self.encode_text(text);
        let cog = match &mut self.state {
            LifecycleState::Forge(mem, cog) => {
                let _ = mem.store(text, vector);
                cog
            },
            LifecycleState::Runtime(_, delta, cog) => {
                let _ = delta.store(text, vector);
                cog
            }
        };

        let parts: Vec<String> = text.split_whitespace()
            .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()).to_lowercase())
            .filter(|w| !w.is_empty())
            .collect();
        if parts.len() >= 2 {
             let s = &parts[0];
             let o = &parts[parts.len()-1];
             let rel_type = if parts.contains(&"is".to_string()) { RelationType::Taxonomic } else { RelationType::Structural };
             cog.add_relation(s, o, rel_type, 1.0, 1.0);
             cog.reinforce_knowledge(text, 1.0);
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
                                return format!("Logic: Yes, {} related to {} (via {})", s_candidate, next_rel.target, intermediate);
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
                         return format!("Logic: {} is related to {}", s_candidate, rel.target);
                     }
                 }
             }
        }

        let vector = self.encode_text(text);
        let results = self.query(&vector);
        if let Some((top, sim)) = results.first() {
            if *sim > 0.3 {
                format!("Logic: Match found for query. Result: {} (confidence: {:.2})", top, sim)
            } else {
                "No match found.".to_string()
            }
        } else {
            "No match found.".to_string()
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
