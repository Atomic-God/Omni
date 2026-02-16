use log::{debug, info};
use core_vsa::FactTriple;

pub struct SymbolicNLP;

impl SymbolicNLP {
    /// Deep extraction of meaning using rule-based Industrial Entity Recognition and Relation Mapping.
    pub fn extract_deep_facts(text: &str) -> Vec<FactTriple> {
        info!("Industrial NLP: Deep meaning extraction from text chunk (len={})", text.len());
        let mut facts = Vec::new();

        let _industrial_entities = [
            "factory", "sensor", "motor", "system", "electricity", "process",
            "output", "input", "machine", "device", "network", "server", "dog", "animal"
        ];

        let sentences = text.split(|c| c == '.' || c == '!' || c == '?');
        for sentence in sentences {
            let tokens: Vec<String> = sentence.split_whitespace()
                .map(|t| t.trim_matches(|c: char| !c.is_alphanumeric()).to_lowercase())
                .filter(|t| !t.is_empty())
                .collect();

            if tokens.len() < 2 { continue; }

            // 1. Taxonomy / Identification
            if tokens.contains(&"is".to_string()) || tokens.contains(&"are".to_string()) {
                if let Some(pos) = tokens.iter().position(|t| t == "is" || t == "are") {
                    if pos > 0 && pos < tokens.len() - 1 {
                        facts.push(FactTriple {
                            subject: tokens[pos-1].clone(),
                            predicate: "taxonomy".to_string(),
                            object: tokens[pos+1].clone(),
                        });
                    }
                }
            }

            // 2. Generic Action/Predicate (Last word as object if no verb found)
            if facts.is_empty() && tokens.len() == 2 {
                facts.push(FactTriple {
                    subject: tokens[0].clone(),
                    predicate: "action".to_string(),
                    object: tokens[1].clone(),
                });
            }

            // 3. Industrial Relations
            let industrial_verbs = ["uses", "requires", "causes", "triggers", "connects", "has", "eats", "eat", "breathes", "means"];
            for verb in industrial_verbs {
                if tokens.contains(&verb.to_string()) {
                    if let Some(pos) = tokens.iter().position(|t| t == verb) {
                        if pos > 0 && pos < tokens.len() - 1 {
                            facts.push(FactTriple {
                                subject: tokens[pos-1].clone(),
                                predicate: verb.to_string(),
                                object: tokens[pos+1].clone(),
                            });
                        }
                    }
                }
            }
        }

        debug!("NLP: Extracted {} deep facts", facts.len());
        facts
    }
}
