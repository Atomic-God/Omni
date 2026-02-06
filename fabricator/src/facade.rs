use crate::FabricationPipeline;
use engine::OmniMind;
use memory::save_mind;
use std::sync::{Arc, Mutex};

pub struct OmniForge {
    pub mind: Arc<Mutex<OmniMind>>,
}

impl OmniForge {
    pub fn new() -> Self {
        Self {
            mind: Arc::new(Mutex::new(OmniMind::new())),
        }
    }

    pub fn fabricate_mind(&self, source: &str, output_path: &str) -> Result<(), std::io::Error> {
        let pipeline = FabricationPipeline::new();
        let pack = pipeline.fabricate(source);
        save_mind(&pack, output_path)
    }

    pub fn load_mind(&self, path: &str) -> Result<(), std::io::Error> {
        let mut mind = self.mind.lock().unwrap();
        mind.load(path)
    }

    pub fn run_query(&self, query: &str) -> String {
        let mind = self.mind.lock().unwrap();
        mind.ask(query)
    }

    pub fn inspect_mind(&self, path: &str) -> Result<String, std::io::Error> {
        // Load without initializing engine, just read headers/metadata
        // memory::load_mind reads the whole zip. Ideally we'd just read metadata.
        // For now, loading full mind is acceptable for inspection.
        let pack = memory::load_mind(path)?;
        Ok(format!("{:#?}", pack.metadata))
    }
}
