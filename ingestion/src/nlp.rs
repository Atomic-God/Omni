use tracing::{debug, info};
use core_vsa::FactTriple;
use unicode_normalization::UnicodeNormalization;
use whatlang::{detect, Lang};

pub struct SymbolicNLP;

impl SymbolicNLP {
    /// Deep extraction of meaning using rule-based Industrial Entity Recognition and Relation Mapping.
    pub fn extract_deep_facts(text: &str) -> Vec<FactTriple> {
        // 1. Unicode Normalization (NFC)
        let normalized: String = text.nfc().collect();

        // 2. Language Detection
        let lang_info = detect(&normalized);
        let lang = lang_info.map(|info| info.lang()).unwrap_or(Lang::Eng);
        info!("Industrial NLP: Deep meaning extraction [Lang: {:?}] (len={})", lang, normalized.len());

        let mut facts = Vec::new();

        let _industrial_entities = [
            "factory", "sensor", "motor", "system", "electricity", "process",
            "output", "input", "machine", "device", "network", "server", "dog", "animal"
        ];

        let sentences = normalized.split(|c| c == '.' || c == '!' || c == '?');
        for sentence in sentences {
            let tokens: Vec<String> = sentence.split_whitespace()
                .map(|t| t.trim_matches(|c: char| !c.is_alphanumeric()).to_lowercase())
                .filter(|t| !t.is_empty())
                .collect();

            if tokens.len() < 2 { continue; }

            // 1. Taxonomy / Identification (Multi-language triggers)
            let tax_triggers = ["is", "are", "es", "son", "est", "sont", "ist", "sind", "ser", "esta"];
            let tax_pos = tokens.iter().position(|t| tax_triggers.contains(&t.as_str()));

            if let Some(pos) = tax_pos {
                if pos > 0 && pos < tokens.len() - 1 {
                    facts.push(FactTriple {
                        subject: tokens[pos-1].clone(),
                        predicate: "taxonomy".to_string(),
                        object: tokens[pos+1].clone(),
                    });
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

            // 3. Industrial Relations (Multi-language triggers)
            let industrial_verbs = [
                "uses", "requires", "causes", "triggers", "connects", "has", "means",
                "inhibits", "facilitates", "promotes", "blocks", "prevents",
                "eats", "eat", "breathes", "fly", "flies",
                "utiliza", "requiere", "causa", "conecta", "tiene", "significa",
                "utilise", "necessite", "provoque", "relie", "a", "signifie",
                "verwendet", "benotigt", "verursacht", "verbindet", "hat", "bedeutet"
            ];
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

            // 4. Proximity / Adjacency for Industrial Concepts
            for i in 0..tokens.len() - 1 {
                let w1 = &tokens[i];
                let w2 = &tokens[i+1];
                if w1.len() > 3 && w2.len() > 3 {
                    facts.push(FactTriple {
                        subject: w1.clone(),
                        predicate: "related_to".to_string(),
                        object: w2.clone(),
                    });
                }
            }
        }

        debug!("NLP: Extracted {} deep facts", facts.len());
        facts
    }
}
