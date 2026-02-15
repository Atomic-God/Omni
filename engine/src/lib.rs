use std::path::PathBuf;
use memory::MemoryManager;
use log::info;
use ingestion::ingest_graph;
use core_vsa::HyperVector;
use core_vsa::traits::MemoryStore;
use runtime::HardwareAdapter;

pub mod adapter;
pub mod metrics;
pub mod explanation;
pub mod context;
pub mod budget;
pub mod governance;
pub mod bench;

pub enum LifecycleState {
    Forge(MemoryManager),
    Runtime(MemoryManager, MemoryManager),
}

pub struct OmniMind {
    pub state: LifecycleState,
    pub metrics: metrics::RuntimeMetrics,
    pub vsa_dimension: usize, // Added
}

impl OmniMind {
    pub fn new_forge(path: &str) -> Self {
        info!("Initializing OmniMind in FORGE mode at {}", path);
        let mut memory = MemoryManager::new(&PathBuf::from(path));

        let adapter = HardwareAdapter::new();
        adapter.report();
        let vsa_dimension = adapter.suggest_dimension();

        if adapter.is_low_memory_mode() {
            memory.set_low_memory_mode(true);
        }

        Self {
            state: LifecycleState::Forge(memory),
            metrics: metrics::RuntimeMetrics::new(),
            vsa_dimension,
        }
    }

    pub fn new_runtime(base_path: &str, delta_path: &str) -> Self {
        info!("Initializing OmniMind in RUNTIME mode.");
        let mut base_mem = MemoryManager::new(&PathBuf::from(base_path));
        let mut delta_mem = MemoryManager::new(&PathBuf::from(delta_path));

        let adapter = HardwareAdapter::new();
        let vsa_dimension = adapter.suggest_dimension();

        if adapter.is_low_memory_mode() {
            base_mem.set_low_memory_mode(true);
            delta_mem.set_low_memory_mode(true);
        }

        Self {
            state: LifecycleState::Runtime(base_mem, delta_mem),
            metrics: metrics::RuntimeMetrics::new(),
            vsa_dimension,
        }
    }

    pub fn ingest_file(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        info!("API: Ingesting file {}", path);
        let graph = ingest_graph(&PathBuf::from(path)).map_err(|e| e.to_string())?;

        match &mut self.state {
            LifecycleState::Forge(mem) => {
                for node in graph.nodes {
                    let _ = mem.store(node.id.as_str(), node.vector);
                }
            },
            LifecycleState::Runtime(_, delta) => {
                for node in graph.nodes {
                    let _ = delta.store(node.id.as_str(), node.vector);
                }
            }
        }
        Ok(())
    }

    pub fn query(&self, concept: &HyperVector) -> Vec<(String, f32)> {
        match &self.state {
            LifecycleState::Forge(mem) => mem.query_nearest(concept, 5),
            LifecycleState::Runtime(base, delta) => {
                let mut results = delta.query_nearest(concept, 5);
                results.extend(base.query_nearest(concept, 5));
                results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
                results.truncate(5);
                results
            }
        }
    }

    pub fn snapshot(&self, name: &str) -> Result<(), Box<dyn std::error::Error>> {
        match &self.state {
            LifecycleState::Forge(mem) => mem.save_snapshot(name),
            LifecycleState::Runtime(_, delta) => delta.save_snapshot(name),
        }
    }

    pub fn memory_stats(&self) -> String {
        match &self.state {
            LifecycleState::Forge(mem) => format!("Forge Memory: {} items", mem.metadata.len()),
            LifecycleState::Runtime(base, delta) => format!("Base: {} | Delta: {}", base.metadata.len(), delta.metadata.len()),
        }
    }

    pub fn lifecycle_status(&self) -> String {
        match &self.state {
            LifecycleState::Forge(_) => "FORGE (Training/Mutation Allowed)".to_string(),
            LifecycleState::Runtime(_, _) => "RUNTIME (Inference/Delta Only)".to_string(),
        }
    }

    pub fn learn(&mut self, text: &str) {
        let vector = HyperVector::deterministic_dim(text.len() as u64, self.vsa_dimension);
        match &mut self.state {
            LifecycleState::Forge(mem) => {
                let _ = mem.store(text, vector);
            },
            LifecycleState::Runtime(_, delta) => {
                let _ = delta.store(text, vector);
            }
        }
    }

    pub fn ask(&mut self, text: &str) -> String {
        let vector = HyperVector::deterministic_dim(text.len() as u64, self.vsa_dimension);
        let results = self.query(&vector);
        if let Some((top, sim)) = results.first() {
            format!("Closest match: {} (sim: {:.2})", top, sim)
        } else {
            "No match found.".to_string()
        }
    }

    pub fn save(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.snapshot(path)
    }

    pub fn load(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        match &mut self.state {
            LifecycleState::Forge(mem) => mem.load_snapshot(path),
            LifecycleState::Runtime(base, _) => base.load_snapshot(path),
        }
    }

    pub fn trace_reasoning(&self, concept: &str) -> String {
        // Placeholder for trace logic
        format!("Trace for {}: [Observation] -> [Orient] -> [Decide] -> [Act]", concept)
    }

    pub fn export_portable_bundle(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        info!("Exporting portable runtime bundle to {}", path);
        match &self.state {
            LifecycleState::Forge(mem) => mem.export_mindpack(&std::path::Path::new(path))?,
            LifecycleState::Runtime(base, _) => base.export_mindpack(&std::path::Path::new(path))?,
        }
        Ok(())
    }
}
