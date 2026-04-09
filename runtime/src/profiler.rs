use std::time::{Instant, Duration};
use std::collections::HashMap;
use std::sync::Mutex;
use once_cell::sync::Lazy;

static PROFILER: Lazy<Mutex<Profiler>> = Lazy::new(|| Mutex::new(Profiler::new()));

pub struct Profiler {
    stats: HashMap<String, Duration>,
    counts: HashMap<String, usize>,
}

impl Profiler {
    pub fn new() -> Self {
        Self {
            stats: HashMap::new(),
            counts: HashMap::new(),
        }
    }

    pub fn record(name: &str, duration: Duration) {
        if let Ok(mut p) = PROFILER.lock() {
            *p.stats.entry(name.to_string()).or_insert(Duration::ZERO) += duration;
            *p.counts.entry(name.to_string()).or_insert(0) += 1;
        }
    }

    pub fn get_averages() -> HashMap<String, f64> {
        let mut averages = HashMap::new();
        if let Ok(p) = PROFILER.lock() {
            for (name, duration) in &p.stats {
                let count = p.counts[name];
                if count > 0 {
                    averages.insert(name.clone(), duration.as_secs_f64() / count as f64 * 1000.0);
                }
            }
        }
        averages
    }
}

pub struct ProfileScope {
    name: String,
    start: Instant,
}

impl ProfileScope {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            start: Instant::now(),
        }
    }
}

impl Drop for ProfileScope {
    fn drop(&mut self) {
        Profiler::record(&self.name, self.start.elapsed());
    }
}
