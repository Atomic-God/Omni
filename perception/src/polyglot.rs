use std::collections::HashMap;

pub struct UnknownLanguageAnalyzer;

impl UnknownLanguageAnalyzer {
    pub fn analyze(text: &str) -> LanguageAnalysis {
        let mut char_freq = HashMap::new();
        let mut word_freq = HashMap::new();
        let mut total_chars = 0;
        let mut total_words = 0;

        for word in text.split_whitespace() {
            *word_freq.entry(word.to_string()).or_insert(0) += 1;
            total_words += 1;
            for char in word.chars() {
                *char_freq.entry(char).or_insert(0) += 1;
                total_chars += 1;
            }
        }

        LanguageAnalysis {
            char_entropy: compute_entropy(&char_freq, total_chars),
            word_entropy: compute_entropy(&word_freq, total_words),
            script_type: detect_script(text),
            estimated_morphology: estimate_morphology(&word_freq),
        }
    }
}

#[derive(Debug)]
pub struct LanguageAnalysis {
    pub char_entropy: f32,
    pub word_entropy: f32,
    pub script_type: String,
    pub estimated_morphology: String,
}

fn compute_entropy<K>(counts: &HashMap<K, usize>, total: usize) -> f32 {
    let mut entropy = 0.0;
    for &count in counts.values() {
        let p = count as f32 / total as f32;
        entropy -= p * p.log2();
    }
    entropy
}

fn detect_script(text: &str) -> String {
    // Simple heuristic based on first few chars
    if let Some(c) = text.chars().next() {
        if c.is_ascii() { "Latin/Ascii".to_string() }
        else if c >= '\u{0400}' && c <= '\u{04FF}' { "Cyrillic".to_string() }
        else if c >= '\u{4E00}' && c <= '\u{9FFF}' { "Han".to_string() }
        else { "Unknown".to_string() }
    } else {
        "Empty".to_string()
    }
}

fn estimate_morphology(word_freq: &HashMap<String, usize>) -> String {
    // Basic clustering: check if many words share suffixes
    let mut suffixes = HashMap::new();
    for word in word_freq.keys() {
        if word.len() > 3 {
            let suffix = &word[word.len()-2..];
            *suffixes.entry(suffix.to_string()).or_insert(0) += 1;
        }
    }

    // If top suffix appears in > 10% of words, might be agglutinative or inflectional
    let total_unique = word_freq.len();
    if total_unique == 0 { return "None".to_string(); }

    let mut max_count = 0;
    let mut best_suffix = String::new();
    for (s, c) in suffixes {
        if c > max_count {
            max_count = c;
            best_suffix = s;
        }
    }

    if max_count > total_unique / 5 {
        format!("Suffix-heavy (e.g. -{})", best_suffix)
    } else {
        "Isolating/Unknown".to_string()
    }
}
