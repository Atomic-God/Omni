use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Intent {
    Query(String),
    Explain(String),
    Plan { start: String, end: String },
    Learn(String),
    Unknown,
}

pub struct IntentParser;

impl IntentParser {
    pub fn parse(input: &str) -> Intent {
        let lower = input.trim().to_lowercase();

        if lower.starts_with("explain ") {
            let concept = input[8..].trim().to_string();
            return Intent::Explain(concept);
        }

        if lower.starts_with("plan ") || lower.starts_with("how to ") {
            // simplified parsing: "Plan X to Y"
            if let Some(to_idx) = lower.find(" to ") {
                let start_idx = if lower.starts_with("plan ") { 5 } else { 7 };
                let start = input[start_idx..to_idx].trim().to_string();
                let end = input[to_idx+4..].trim().to_string();
                return Intent::Plan { start, end };
            }
        }

        if lower.starts_with("remember ") || lower.starts_with("learn ") {
             let content = if lower.starts_with("remember ") { input[9..].trim() } else { input[6..].trim() };
             return Intent::Learn(content.to_string());
        }

        if lower.ends_with("?") {
            return Intent::Query(input.to_string());
        }

        // Default fallback
        Intent::Unknown
    }
}
