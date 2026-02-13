use std::collections::HashMap;
use unicode_normalization::UnicodeNormalization;
use log::warn;

pub struct SimpleTokenizer {
    pub vocab: HashMap<String, usize>,
    pub reverse_vocab: HashMap<usize, String>,
    pub unk_token: usize,
    pub pad_token: usize,
}

impl SimpleTokenizer {
    pub fn build(data_iter: impl Iterator<Item = String>, max_vocab: usize) -> Self {
        let mut counts = HashMap::new();
        // Use a reservoir sampler or limit iteration for building vocab to avoid massive memory
        // For Phase 1, we assume the iterator is streamable.

        for line in data_iter {
            let normalized = line.nfc().collect::<String>();
            // Split by whitespace and punctuation boundaries
            // Simple regex-less split for "Industrial" robustness without heavy regex crate deps if possible?
            // Actually regex is standard.
            // Let's use simple char checks for multilingual safety.

            let tokens = Self::tokenize_raw(&normalized);
            for token in tokens {
                *counts.entry(token).or_insert(0) += 1;
            }
        }

        // Sort by frequency
        let mut entries: Vec<_> = counts.into_iter().collect();
        entries.sort_by_key(|&(_, count)| std::cmp::Reverse(count));

        let mut vocab = HashMap::new();
        let mut reverse_vocab = HashMap::new();

        // Reserved tokens
        vocab.insert("<PAD>".to_string(), 0); reverse_vocab.insert(0, "<PAD>".to_string());
        vocab.insert("<UNK>".to_string(), 1); reverse_vocab.insert(1, "<UNK>".to_string());
        vocab.insert("<BOS>".to_string(), 2); reverse_vocab.insert(2, "<BOS>".to_string());
        vocab.insert("<EOS>".to_string(), 3); reverse_vocab.insert(3, "<EOS>".to_string());

        let start_idx = 4;
        for (i, (token, _)) in entries.into_iter().take(max_vocab - start_idx).enumerate() {
            let idx = start_idx + i;
            vocab.insert(token.clone(), idx);
            reverse_vocab.insert(idx, token);
        }

        Self { vocab, reverse_vocab, unk_token: 1, pad_token: 0 }
    }

    fn tokenize_raw(text: &str) -> Vec<String> {
        // Multilingual splitting:
        // 1. Normalize (NFC) - handled by caller or here
        // 2. Split on whitespace
        // 3. Keep punctuation separate?
        // Simple logic: Isolate punctuation chars.

        let mut tokens = Vec::new();
        let mut current_token = String::new();

        for c in text.chars() {
            if c.is_whitespace() {
                if !current_token.is_empty() {
                    tokens.push(current_token.clone());
                    current_token.clear();
                }
            } else if c.is_ascii_punctuation() || c.is_control() {
                if !current_token.is_empty() {
                    tokens.push(current_token.clone());
                    current_token.clear();
                }
                tokens.push(c.to_string());
            } else {
                // Accumulate alnum / other unicode
                current_token.push(c);
            }
        }
        if !current_token.is_empty() {
            tokens.push(current_token);
        }
        tokens
    }

    pub fn encode(&self, text: &str) -> Vec<usize> {
        let normalized = text.nfc().collect::<String>();
        let raw_tokens = Self::tokenize_raw(&normalized);
        raw_tokens.iter()
            .map(|t| *self.vocab.get(t).unwrap_or(&self.unk_token))
            .collect()
    }

    pub fn decode(&self, tokens: &[usize]) -> String {
        tokens.iter()
            .map(|&id| self.reverse_vocab.get(&id).map(|s| s.as_str()).unwrap_or(""))
            .collect::<Vec<&str>>()
            .join(" ")
    }

    pub fn vocab_size(&self) -> usize {
        self.vocab.len()
    }
}
