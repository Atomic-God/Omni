use crate::traits::Decoder;
use core_vsa::index::LshIndex;
use core_vsa::HyperVector;
use std::collections::HashMap;

pub struct TextDecoder {
    pub vocab_index: LshIndex, // Should be populated with vocab
    pub vocab_map: HashMap<String, HyperVector>, // Reverse lookup needed? No, LshIndex stores HVs. We need HV -> String.
                                                 // LshIndex stores HyperVector. We can't map back to String unless LshIndex stores ID or we have a map.
                                                 // Let's assume we search LshIndex to get candidate HVs, then find exact match in map?
                                                 // Actually, simple NN search against vocab map is easier for prototype if LSH doesn't support payload.
                                                 // I'll add a `reverse_vocab: Vec<(HyperVector, String)>` or similar.
                                                 // Or just iterate `vocab_map` for now. The prompt says "Use existing LshIndex to speed up".
                                                 // I need to modify LshIndex to store payload or parallel array.
                                                 // For now, I will use linear scan over vocab_map for correctness as LshIndex is just HVs.
                                                 // Wait, prompt says "Compare hypervector against vocab.json word vectors".
}

impl TextDecoder {
    pub fn new(vocab: HashMap<String, HyperVector>) -> Self {
        // Build LSH index?
        let mut index = LshIndex::new(64);
        for (_, vec) in &vocab {
            index.insert(vec.clone());
        }
        Self {
            vocab_index: index,
            vocab_map: vocab,
        }
    }
}

impl Decoder for TextDecoder {
    fn decode(&self, hv: &HyperVector) -> String {
        // Find nearest neighbor in vocab
        // Naive: scan all.
        let mut best_word = "unknown".to_string();
        let mut best_sim = -1.0;

        for (word, vec) in &self.vocab_map {
            let sim = hv.similarity(vec);
            if sim > best_sim {
                best_sim = sim;
                best_word = word.clone();
            }
        }
        best_word
    }
}
