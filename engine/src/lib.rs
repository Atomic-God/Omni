use std::path::PathBuf;
use memory::MemoryManager;
use log::info;

pub mod adapter;
pub mod metrics;
pub mod explanation;
pub mod context;
pub mod budget;
pub mod governance;
pub mod bench; // New

pub enum LifecycleState {
    Forge(MemoryManager),
    Runtime(MemoryManager, MemoryManager),
}

pub struct OmniMind {
    pub state: LifecycleState,
    pub metrics: metrics::RuntimeMetrics,
}

impl OmniMind {
    pub fn new_forge(path: &str) -> Self {
        info!("Initializing OmniMind in FORGE mode at {}", path);
        let memory = MemoryManager::new(&PathBuf::from(path));
        Self {
            state: LifecycleState::Forge(memory),
            metrics: metrics::RuntimeMetrics::new(),
        }
    }

    pub fn new_runtime(base_path: &str, delta_path: &str) -> Self {
        info!("Initializing OmniMind in RUNTIME mode.");
        let base_mem = MemoryManager::new(&PathBuf::from(base_path));
        let delta_mem = MemoryManager::new(&PathBuf::from(delta_path));

        Self {
            state: LifecycleState::Runtime(base_mem, delta_mem),
            metrics: metrics::RuntimeMetrics::new(),
        }
    }

    pub fn lifecycle_status(&self) -> String {
        match &self.state {
            LifecycleState::Forge(_) => "FORGE (Training/Mutation Allowed)".to_string(),
            LifecycleState::Runtime(_, _) => "RUNTIME (Inference/Delta Only)".to_string(),
        }
    }
}
