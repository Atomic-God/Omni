use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NormalizedMetadata {
    pub source: String,
    pub file_type: String,
    pub hash: String,
    pub timestamp: u64,
    pub language: String,
    pub fields: HashMap<String, String>,
}

impl NormalizedMetadata {
    pub fn new(source: &str) -> Self {
        Self {
            source: source.to_string(),
            ..Default::default()
        }
    }

    pub fn insert(&mut self, key: &str, value: String) {
        self.fields.insert(key.to_string(), value);
    }

    pub fn to_map(self) -> HashMap<String, String> {
        let mut map = self.fields;
        map.insert("source".to_string(), self.source);
        map.insert("file_type".to_string(), self.file_type);
        map.insert("hash".to_string(), self.hash);
        map.insert("timestamp".to_string(), self.timestamp.to_string());
        map.insert("language".to_string(), self.language);
        map
    }
}
