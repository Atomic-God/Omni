use crate::FabricationPipeline;
use engine::OmniMind;
use memory::save_mind;
use std::sync::{Arc, Mutex};
use std::fs::File;
use std::io::Write;

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

    pub fn explain_concept(&self, concept: &str) -> Result<String, std::io::Error> {
        let mind = self.mind.lock().unwrap();
        Ok(mind.explain(concept))
    }

    pub fn export_mind(&self, output_path: &str) -> Result<(), std::io::Error> {
        let mind = self.mind.lock().unwrap();
        // Export relation graph to JSON
        let json = serde_json::to_string_pretty(&mind.cognition.relation_graph)?;
        let mut file = File::create(output_path)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }

    pub fn inspect_mind(&self, path: &str) -> Result<String, std::io::Error> {
        let pack = memory::load_mind(path)?;
        Ok(format!("{:#?}", pack.metadata))
    }
}
