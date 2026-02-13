use core_vsa::HyperVector;
use std::collections::HashMap;

/// Resolves user intent from a natural language vector (or string).
pub struct IntentResolver;

#[derive(Debug, PartialEq)]
pub enum Intent {
    Query(String),
    Command(String),
    Statement(String),
    Unknown,
}

impl IntentResolver {
    /// Maps a perception vector to an Intent.
    /// In Phase 1, this uses simple heuristic or similarity to prototypes.
    pub fn resolve(vector: &HyperVector, text_hint: Option<&str>) -> Intent {
        // If text hint is available (e.g. from tokenizer), use rule-based for 100% accuracy on basic commands
        if let Some(text) = text_hint {
            if text.ends_with('?') || text.starts_with("what") || text.starts_with("who") {
                return Intent::Query(text.to_string());
            }
            if text.starts_with("run") || text.starts_with("create") || text.starts_with("delete") {
                return Intent::Command(text.to_string());
            }
            return Intent::Statement(text.to_string());
        }

        // VSA Logic: Compare with prototype vectors (ROLE_VERB, etc.)
        // Stub: Assume Unknown if no text.
        Intent::Unknown
    }
}
