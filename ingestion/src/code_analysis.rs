use std::path::Path;
use std::fs::File;
use std::io::{BufRead, BufReader};
use core_vsa::FactTriple;
use tracing::debug;

pub struct CodeAnalyzer;

impl CodeAnalyzer {
    pub fn extract_code_meaning(path: &Path) -> Vec<FactTriple> {
        let mut facts = Vec::new();
        let file_name = path.file_name().unwrap_or_default().to_string_lossy().to_string();

        let file = match File::open(path) {
            Ok(f) => f,
            Err(_) => return facts,
        };

        let reader = BufReader::new(file);
        for line in reader.lines() {
            if let Ok(l) = line {
                let trimmed = l.trim();

                // 1. Structural Meaning: Functions & Methods
                let func_triggers = ["fn ", "def ", "public void ", "public static ", "async fn ", "function "];
                for trigger in func_triggers {
                    if trimmed.starts_with(trigger) {
                        let parts: Vec<&str> = trimmed[trigger.len()..].split(|c: char| !c.is_alphanumeric() && c != '_').collect();
                        if let Some(name) = parts.first().filter(|s| !s.is_empty()) {
                            facts.push(FactTriple {
                                subject: file_name.clone(),
                                predicate: "defines_function".to_string(),
                                object: name.to_string(),
                            });
                        }
                        break;
                    }
                }

                // 2. Structural Meaning: Types & Classes
                let type_triggers = ["struct ", "class ", "enum ", "interface ", "pub struct ", "pub enum "];
                for trigger in type_triggers {
                    if trimmed.starts_with(trigger) {
                        let parts: Vec<&str> = trimmed[trigger.len()..].split(|c: char| !c.is_alphanumeric() && c != '_').collect();
                        if let Some(name) = parts.first().filter(|s| !s.is_empty()) {
                            facts.push(FactTriple {
                                subject: file_name.clone(),
                                predicate: "defines_type".to_string(),
                                object: name.to_string(),
                            });
                        }
                        break;
                    }
                }

                // 3. Structural Meaning: Dependencies
                if trimmed.starts_with("use ") || trimmed.starts_with("import ") || trimmed.starts_with("extern crate ") {
                    let parts: Vec<&str> = trimmed.split(|c| c == ' ' || c == ';' || c == ':').collect();
                    for part in parts.iter().skip(1) {
                         if !part.is_empty() && *part != "crate" && *part != "pub" {
                            facts.push(FactTriple {
                                subject: file_name.clone(),
                                predicate: "depends_on".to_string(),
                                object: part.to_string(),
                            });
                            break;
                         }
                    }
                }
            }
        }

        debug!("CodeAnalysis: Extracted {} structural facts from {}", facts.len(), file_name);
        facts
    }
}
