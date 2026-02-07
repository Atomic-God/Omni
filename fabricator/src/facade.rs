use crate::FabricationPipeline;
use engine::{ForgeMind, RuntimeMind};
use memory::save_mind;
use std::sync::{Arc, Mutex};
use std::fs::File;
use std::io::Write;

pub struct OmniForge {
    pub forge_mind: Arc<Mutex<ForgeMind>>,
    pub runtime_mind: Arc<Mutex<Option<RuntimeMind>>>,
}

impl OmniForge {
    pub fn new() -> Self {
        Self {
            forge_mind: Arc::new(Mutex::new(ForgeMind::new())),
            runtime_mind: Arc::new(Mutex::new(None)),
        }
    }

    // --- Forge Actions ---

    pub fn fabricate_mind(&self, source: &str, output_path: &str) -> Result<(), std::io::Error> {
        let pipeline = FabricationPipeline::new();
        let pack = pipeline.fabricate(source);
        save_mind(&pack, output_path)
    }

    pub fn load_forge_master(&self, path: &str) -> Result<(), std::io::Error> {
        let mut mind = self.forge_mind.lock().unwrap();
        mind.load_master(path)
    }

    pub fn save_forge_master(&self, path: &str) -> Result<(), std::io::Error> {
        let mind = self.forge_mind.lock().unwrap();
        mind.save_master(path)
    }

    pub fn snapshot(&self, version: &str, source: &str, path: &str) -> Result<(), std::io::Error> {
        let mind = self.forge_mind.lock().unwrap();
        mind.snapshot(version, source, path)
    }

    pub fn export_mind(&self, output_path: &str) -> Result<(), std::io::Error> {
        let mind = self.forge_mind.lock().unwrap();
        // Export relation graph to JSON
        let json = serde_json::to_string_pretty(&mind.cognition.relation_graph)?;
        let mut file = File::create(output_path)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }

    // --- Runtime Actions ---

    pub fn load_runtime(&self, base_path: &str, overlay_path: Option<&str>) -> Result<(), std::io::Error> {
        let runtime = RuntimeMind::load(base_path, overlay_path)?;
        let mut slot = self.runtime_mind.lock().unwrap();
        *slot = Some(runtime);
        Ok(())
    }

    pub fn save_runtime_overlay(&self, path: &str) -> Result<(), std::io::Error> {
        let slot = self.runtime_mind.lock().unwrap();
        if let Some(runtime) = slot.as_ref() {
            runtime.save_overlay(path)
        } else {
            Err(std::io::Error::new(std::io::ErrorKind::Other, "No Runtime Loaded"))
        }
    }

    pub fn learn_runtime(&self, text: &str) -> Result<(), std::io::Error> {
        let mut slot = self.runtime_mind.lock().unwrap();
        if let Some(runtime) = slot.as_mut() {
            runtime.learn_personal(text);
            Ok(())
        } else {
            Err(std::io::Error::new(std::io::ErrorKind::Other, "No Runtime Loaded"))
        }
    }

    pub fn run_query(&self, query: &str) -> String {
        let slot = self.runtime_mind.lock().unwrap();
        if let Some(runtime) = slot.as_ref() {
            runtime.ask(query)
        } else {
            let forge = self.forge_mind.lock().unwrap();
            format!("(Forge Inspector) {}", forge.ask(query))
        }
    }

    pub fn explain_concept(&self, concept: &str) -> Result<String, std::io::Error> {
        let slot = self.runtime_mind.lock().unwrap();
        if let Some(runtime) = slot.as_ref() {
            // Use ask("Explain X") via Runtime
             Ok(runtime.ask(&format!("Explain {}", concept)))
        } else {
            let forge = self.forge_mind.lock().unwrap();
            Ok(forge.explain(concept))
        }
    }

    pub fn inspect_mind(&self, path: &str) -> Result<String, std::io::Error> {
        let pack = memory::load_snapshot(path)?;
        Ok(format!("{:#?}", pack.metadata))
    }
}
