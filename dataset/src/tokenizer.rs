use std::collections::HashMap;
use unicode_normalization::UnicodeNormalization;
use tracing::info;

pub struct SimpleTokenizer {
    vocab: HashMap<String, usize>,
    reverse_vocab: HashMap<usize, String>,
}

impl SimpleTokenizer {
    pub fn build<I>(text_iter: I, max_vocab: usize) -> Self
    where I: Iterator<Item = String>
    {
        let mut counts = HashMap::new();
        for line in text_iter {
            let normalized = line.nfc().collect::<String>();
            for word in normalized.split_whitespace() {
                *counts.entry(word.to_string()).or_insert(0) += 1;
            }
        }

        let mut counts_vec: Vec<_> = counts.into_iter().collect();
        counts_vec.sort_by(|a, b| b.1.cmp(&a.1));

        let mut vocab = HashMap::new();
        let mut reverse_vocab = HashMap::new();

        vocab.insert("<PAD>".to_string(), 0);
        vocab.insert("<UNK>".to_string(), 1);
        reverse_vocab.insert(0, "<PAD>".to_string());
        reverse_vocab.insert(1, "<UNK>".to_string());

        for (i, (word, _)) in counts_vec.into_iter().take(max_vocab - 2).enumerate() {
            vocab.insert(word.clone(), i + 2);
            reverse_vocab.insert(i + 2, word);
        }

        info!("Tokenizer built with {} tokens.", vocab.len());
        Self { vocab, reverse_vocab }
    }

    pub fn encode(&self, text: &str) -> Vec<usize> {
        let normalized = text.nfc().collect::<String>();
        normalized.split_whitespace()
            .map(|word| *self.vocab.get(word).unwrap_or(&1))
            .collect()
    }

    pub fn decode(&self, ids: &[usize]) -> String {
        ids.iter()
            .map(|id| self.reverse_vocab.get(id).cloned().unwrap_or_else(|| "<UNK>".to_string()))
            .collect::<Vec<_>>()
            .join(" ")
    }

    pub fn vocab_size(&self) -> usize {
        self.vocab.len()
    }
}
