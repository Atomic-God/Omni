use std::path::PathBuf;
use memory::MemoryManager;
use log::info;
use ingestion::ingest_graph;
use core_vsa::HyperVector;

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
}

impl OmniMind {
    /// Initialize in Forge Mode (Training)
    pub fn new_forge(path: &str) -> Self {
        info!("Initializing OmniMind in FORGE mode at {}", path);
        let memory = MemoryManager::new(&PathBuf::from(path));
        Self {
            state: LifecycleState::Forge(memory),
            metrics: metrics::RuntimeMetrics::new(),
        }
    }

    /// Initialize in Runtime Mode (Inference + Delta)
    pub fn new_runtime(base_path: &str, delta_path: &str) -> Self {
        info!("Initializing OmniMind in RUNTIME mode.");
        let base_mem = MemoryManager::new(&PathBuf::from(base_path));
        let delta_mem = MemoryManager::new(&PathBuf::from(delta_path));

        Self {
            state: LifecycleState::Runtime(base_mem, delta_mem),
            metrics: metrics::RuntimeMetrics::new(),
        }
    }

    // --- API Methods ---

    pub fn ingest_file(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        info!("API: Ingesting file {}", path);
        let graph = ingest_graph(&PathBuf::from(path))?;

        // Store in memory based on mode
        match &mut self.state {
            LifecycleState::Forge(mem) => {
                for node in graph.nodes {
                    mem.store(node.id.as_str(), node.vector)?;
                }
            },
            LifecycleState::Runtime(_, delta) => {
                for node in graph.nodes {
                    delta.store(node.id.as_str(), node.vector)?;
                }
            }
        }
        Ok(())
    }

    pub fn query(&self, concept: &HyperVector) -> Vec<(String, f32)> {
        match &self.state {
            LifecycleState::Forge(mem) => mem.query_nearest(concept, 5),
            LifecycleState::Runtime(base, delta) => {
                // Merge query
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
            LifecycleState::Runtime(_, delta) => delta.save_snapshot(name), // Save delta only in runtime
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
}
