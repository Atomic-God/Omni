use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeMetrics {
    pub start_time: u64,
    pub query_count: u64,
    pub learning_events: u64,
    pub memory_pressure: f32, // 0.0 - 1.0
}

impl RuntimeMetrics {
    pub fn new() -> Self {
        Self {
            start_time: SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs(),
            query_count: 0,
            learning_events: 0,
            memory_pressure: 0.0,
        }
    }

    pub fn uptime(&self) -> u64 {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
        now - self.start_time
    }
}
