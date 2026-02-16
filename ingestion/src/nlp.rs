use log::debug;
use core_vsa::FactTriple;

pub struct SymbolicNLP;

impl SymbolicNLP {
    pub fn extract_svo(text: &str) -> Option<FactTriple> {
        let tokens: Vec<&str> = text.split_whitespace()
            .map(|t| t.trim_matches(|c: char| !c.is_alphanumeric()))
            .filter(|t| !t.is_empty())
            .collect();

        if tokens.len() < 3 { return None; }

        let verbs = ["is", "has", "contains", "causes", "uses", "requires", "provides", "works", "connects"];

        for (i, token) in tokens.iter().enumerate() {
            let lower = token.to_lowercase();
            if verbs.contains(&lower.as_str()) {
                if i > 0 && i < tokens.len() - 1 {
                    let subject = tokens[..i].join(" ");
                    let predicate = lower;
                    let object = tokens[i+1..].join(" ");

                    debug!("NLP: Extracted SVO ({}, {}, {})", subject, predicate, object);
                    return Some(FactTriple {
                        subject,
                        predicate,
                        object,
                    });
                }
            }
        }
        None
    }

    pub fn extract_facts(text: &str) -> Vec<FactTriple> {
        let mut facts = Vec::new();
        let sentences = text.split(|c| c == '.' || c == '!' || c == '?');
        for sentence in sentences {
            if let Some(fact) = Self::extract_svo(sentence) {
                facts.push(fact);
            }
        }
        facts
    }
}
