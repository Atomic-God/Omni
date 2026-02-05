use crate::traits::Decoder;
use core_vsa::index::LshIndex;
use core_vsa::HyperVector;
use std::collections::HashMap;

pub struct TextDecoder {
    pub vocab_index: LshIndex,
    pub vocab_map: HashMap<String, HyperVector>,
}

impl TextDecoder {
    pub fn new(vocab: HashMap<String, HyperVector>) -> Self {
        let mut index = LshIndex::new(64);
        for (_, vec) in &vocab {
            index.insert(vec.clone());
        }
        Self {
            vocab_index: index,
            vocab_map: vocab,
        }
    }

    pub fn decode_word(&self, hv: &HyperVector) -> String {
        self.decode(hv)
    }

    pub fn decode_svo(&self, hv: &HyperVector) -> String {
        // Unbind S, V, O
        let s_part = hv.bind(&core_vsa::ROLE_SUBJECT);
        let v_part = hv.bind(&core_vsa::ROLE_VERB);
        let o_part = hv.bind(&core_vsa::ROLE_OBJECT);

        let s_word = self.decode(&s_part);
        let v_word = self.decode(&v_part);
        let o_word = self.decode(&o_part);

        format!("{} {} {}", s_word, v_word, o_word)
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
