use regex::Regex;
use once_cell::sync::Lazy;

pub fn tokenize(text: &str) -> Vec<String> {
    // Universal Tokenizer
    // Splits by Unicode whitespace and punctuation, keeping meaningful symbols.
    // Handles CJK, Cyrillic, Latin, etc.

    // 1. Normalize
    let normalized = text.trim().to_lowercase(); // Simple case folding

    // 2. Split by Unicode Punctuation/Symbol/Separator
    // We want to keep words (alphanumeric+marks)
    // Regex: [^\w\s] -> Punctuation
    // Actually, simple split by non-alphanumeric is decent for VSA.
    // But we want to support non-latin.
    // Rust's `char::is_alphanumeric` supports unicode.

    let mut tokens = Vec::new();
    let mut current_token = String::new();

    for c in normalized.chars() {
        if c.is_alphanumeric() || c == '_' || c == '-' {
            current_token.push(c);
        } else {
            if !current_token.is_empty() {
                tokens.push(current_token.clone());
                current_token.clear();
            }
            // Treat punctuation as tokens if meaningful?
            // For now, discard.
        }
    }
    if !current_token.is_empty() {
        tokens.push(current_token);
    }

    tokens
}
