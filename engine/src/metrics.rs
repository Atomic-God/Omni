use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeMetrics {
    pub uptime_seconds: u64,
    pub queries_processed: u64,
    pub learning_events: u64,
    pub average_confidence: f32, // Step 50
    pub consolidation_count: u32,
    pub memories_pruned: u32,
    pub hardware_snapshot: serde_json::Value,
    pub profiling_data: std::collections::HashMap<String, f64>,
}

impl RuntimeMetrics {
    pub fn new() -> Self {
        Self {
            uptime_seconds: 0,
            queries_processed: 0,
            learning_events: 0,
            average_confidence: 0.0,
            consolidation_count: 0,
            memories_pruned: 0,
            hardware_snapshot: serde_json::json!({}),
            profiling_data: std::collections::HashMap::new(),
        }
    }
}
