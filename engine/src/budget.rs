// Token Budget Logic (Approximate)
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TokenBudget {
    pub max_tokens: usize,
    pub current_usage: usize,
}

impl TokenBudget {
    pub fn new(max_tokens: usize) -> Self {
        Self { max_tokens, current_usage: 0 }
    }

    pub fn estimate_usage(text: &str) -> usize {
        // Rough estimate: 1 token ~ 4 bytes
        text.len() / 4
    }

    pub fn can_afford(&self, cost: usize) -> bool {
        self.current_usage + cost <= self.max_tokens
    }

    pub fn consume(&mut self, cost: usize) {
        self.current_usage += cost;
    }

    pub fn reset(&mut self) {
        self.current_usage = 0;
    }
}
