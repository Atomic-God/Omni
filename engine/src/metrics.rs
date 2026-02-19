pub struct RuntimeMetrics {
    pub uptime_seconds: u64,
    pub queries_processed: u64,
    pub learning_events: u64,
}

impl RuntimeMetrics {
    pub fn new() -> Self {
        Self {
            uptime_seconds: 0,
            queries_processed: 0,
            learning_events: 0,
        }
    }
}
