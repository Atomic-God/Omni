use std::collections::HashMap;
use unicode_normalization::UnicodeNormalization;
use log::warn;

pub struct SimpleTokenizer {
    pub vocab: HashMap<String, usize>,
    pub reverse_vocab: HashMap<usize, String>,
    pub unk_token: usize,
    pub pad_token: usize,
    pub bos_token: usize,
    pub eos_token: usize,
}

impl SimpleTokenizer {
    pub fn build(data_iter: impl Iterator<Item = String>, max_vocab: usize) -> Self {
        let mut counts = HashMap::new();
        // Reservoir sampling or streaming stats could be used here for efficiency

        for line in data_iter {
            let normalized = line.nfc().collect::<String>();
            let tokens = Self::tokenize_raw(&normalized);
            for token in tokens {
                *counts.entry(token).or_insert(0) += 1;
            }
        }

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

        Self { vocab, reverse_vocab, unk_token: 1, pad_token: 0, bos_token: 2, eos_token: 3 }
    }

    pub fn tokenize_raw(text: &str) -> Vec<String> {
        let mut tokens = Vec::new();
        let mut current_token = String::new();

        for c in text.chars() {
            if c.is_whitespace() {
                if !current_token.is_empty() {
                    tokens.push(current_token.clone());
                    current_token.clear();
                }
            } else if c.is_ascii_punctuation() || c.is_control() { // Simple punctuation boundary
                if !current_token.is_empty() {
                    tokens.push(current_token.clone());
                    current_token.clear();
                }
                tokens.push(c.to_string());
            } else {
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
        let mut ids = Vec::new();

        ids.push(self.bos_token);
        for t in raw_tokens {
            ids.push(*self.vocab.get(&t).unwrap_or(&self.unk_token));
        }
        ids.push(self.eos_token);

        ids
    }

    pub fn decode(&self, tokens: &[usize]) -> String {
        tokens.iter()
            .filter(|&&id| id != self.pad_token && id != self.bos_token && id != self.eos_token)
            .map(|&id| self.reverse_vocab.get(&id).map(|s| s.as_str()).unwrap_or(""))
            .collect::<Vec<&str>>()
            .join(" ")
    }

    pub fn vocab_size(&self) -> usize {
        self.vocab.len()
    }
}
