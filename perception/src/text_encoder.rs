use crate::tokenizer;
use crate::traits::Encoder;
use core_vsa::{HyperVector, ROLE_OBJECT, ROLE_SUBJECT, ROLE_VERB};
use std::collections::HashMap;

pub struct TextEncoder {
    pub vocab: HashMap<String, HyperVector>,
}

impl TextEncoder {
    pub fn new(vocab: HashMap<String, HyperVector>) -> Self {
        Self { vocab }
    }
}

impl Encoder for TextEncoder {
    fn encode(&self, input: &str) -> HyperVector {
        let words = tokenizer::tokenize(input);

        if words.len() >= 3 {
            // SVO
            let s = self
                .vocab
                .get(&words[0])
                .unwrap_or(&HyperVector::random())
                .clone();
            let v = self
                .vocab
                .get(&words[1])
                .unwrap_or(&HyperVector::random())
                .clone();
            let o = self
                .vocab
                .get(&words[2])
                .unwrap_or(&HyperVector::random())
                .clone();

            s.bind(&ROLE_SUBJECT)
                .bundle(&v.bind(&ROLE_VERB))
                .bundle(&o.bind(&ROLE_OBJECT))
        } else {
            // Bag of words
            let mut bundle = HyperVector::random(); // Should be zero vector but we use random for now or first word
                                                    // Better: bundle all words.
            if let Some(first) = words.first() {
                bundle = self
                    .vocab
                    .get(first)
                    .unwrap_or(&HyperVector::random())
                    .clone();
                for word in words.iter().skip(1) {
                    if let Some(vec) = self.vocab.get(word) {
                        bundle = bundle.bundle(vec);
                    }
                }
            }
            bundle
        }
    }
}
