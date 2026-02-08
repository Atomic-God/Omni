use std::collections::{HashMap, VecDeque};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Debug)]
pub struct SalienceScoring {
    pub scores: HashMap<String, f32>,
    pub decay_rate: f32,
    pub last_update: u64,
}

impl SalienceScoring {
    pub fn new(decay_rate: f32) -> Self {
        Self {
            scores: HashMap::new(),
            decay_rate,
            last_update: SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs(),
        }
    }

    pub fn score(&mut self, concept: &str, boost: f32) {
        self.update_decay();
        let score = self.scores.entry(concept.to_string()).or_insert(0.0);
        *score += boost;
    }

    pub fn update_decay(&mut self) {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
        let elapsed = (now - self.last_update) as f32;
        if elapsed > 0.0 {
            for score in self.scores.values_mut() {
                *score *= (1.0 - self.decay_rate).powf(elapsed / 3600.0); // Hourly decay
            }
            self.last_update = now;
        }
    }

    pub fn get_top(&self, n: usize) -> Vec<String> {
        let mut items: Vec<_> = self.scores.iter().collect();
        items.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap());
        items.into_iter().take(n).map(|(k, _)| k.clone()).collect()
    }
}

// Memory Tiers
#[derive(Clone, Debug)]
pub struct MemoryTier {
    pub capacity: usize,
    pub items: VecDeque<String>,
}

impl MemoryTier {
    pub fn new(capacity: usize) -> Self {
        Self { capacity, items: VecDeque::new() }
    }

    pub fn add(&mut self, item: String) -> Option<String> {
        if self.items.contains(&item) { return None; }
        self.items.push_back(item);
        if self.items.len() > self.capacity {
            self.items.pop_front() // Evicted
        } else {
            None
        }
    }
}
