pub fn tokenize(text: &str) -> Vec<String> {
    text.split_whitespace()
        .map(|s| s.to_lowercase().replace(|c: char| !c.is_alphanumeric(), ""))
        .filter(|s| !s.is_empty())
        .collect()
}
