use engine::OmniMind;
use std::sync::{Arc, Mutex};

pub struct OmniForge {
    pub mind: Arc<Mutex<OmniMind>>,
}

impl OmniForge {
    pub fn new(path: &str) -> Self {
        Self {
            mind: Arc::new(Mutex::new(OmniMind::new_forge(path))),
        }
    }

    pub fn fabricate_mind(&self, _source: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mind = self.mind.lock().unwrap();
        mind.save(output_path)
    }

    pub fn load_runtime(&self, base_path: &str, delta_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let runtime = OmniMind::new_runtime(base_path, delta_path);
        let mut slot = self.mind.lock().unwrap();
        *slot = runtime;
        Ok(())
    }

    pub fn run_query(&self, query: &str) -> String {
        let mut mind = self.mind.lock().unwrap();
        mind.ask(query).answer
    }

    pub fn learn(&self, text: &str) {
        let mut mind = self.mind.lock().unwrap();
        mind.learn(text);
    }
}
