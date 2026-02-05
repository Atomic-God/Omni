use cognition::CognitionCore;
use memory;

pub struct OmniMind {
    cognition: CognitionCore,
}

impl OmniMind {
    pub fn new() -> Self {
        Self {
            cognition: CognitionCore::new(),
        }
    }

    pub fn learn(&mut self, text: &str) {
        self.cognition.learn_text(text);
    }

    pub fn ask(&self, question: &str) -> String {
        self.cognition.query(question)
    }

    pub fn save(&self, path: &str) -> Result<(), std::io::Error> {
        memory::save(&self.cognition, path)
    }

    pub fn load(&mut self, path: &str) -> Result<(), std::io::Error> {
        self.cognition = memory::load(path)?;
        Ok(())
    }
}
