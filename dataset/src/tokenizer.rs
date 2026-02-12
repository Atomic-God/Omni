use std::collections::{HashMap, HashSet};

pub struct SimpleTokenizer {
    pub vocab: HashMap<char, usize>,
    pub reverse_vocab: HashMap<usize, char>,
    pub unk_token: usize,
}

impl SimpleTokenizer {
    pub fn build(data_iter: impl Iterator<Item = String>, max_vocab: usize) -> Self {
        let mut counts = HashMap::new();
        for line in data_iter {
            for char in line.chars() {
                *counts.entry(char).or_insert(0) += 1;
            }
        }

        // Sort by frequency
        let mut entries: Vec<_> = counts.into_iter().collect();
        entries.sort_by_key(|&(_, count)| std::cmp::Reverse(count));

        let mut vocab = HashMap::new();
        let mut reverse_vocab = HashMap::new();

        // Reserved tokens? 0=PAD, 1=UNK, 2=BOS, 3=EOS
        vocab.insert('\0', 0); reverse_vocab.insert(0, '\0'); // PAD
        vocab.insert('', 1); reverse_vocab.insert(1, ''); // UNK

        let start_idx = 2;
        for (i, (char, _)) in entries.into_iter().take(max_vocab - start_idx).enumerate() {
            vocab.insert(char, start_idx + i);
            reverse_vocab.insert(start_idx + i, char);
        }

        Self { vocab, reverse_vocab, unk_token: 1 }
    }

    pub fn encode(&self, text: &str) -> Vec<usize> {
        text.chars()
            .map(|c| *self.vocab.get(&c).unwrap_or(&self.unk_token))
            .collect()
    }

    pub fn decode(&self, tokens: &[usize]) -> String {
        tokens.iter()
            .map(|&id| *self.reverse_vocab.get(&id).unwrap_or(&''))
            .collect()
    }

    pub fn vocab_size(&self) -> usize {
        self.vocab.len()
    }
}
