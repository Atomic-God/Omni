use std::path::Path;
use std::fs::File;
use std::io::{BufRead, BufReader};
use core_vsa::FactTriple;
use log::debug;

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

                // 1. Structural Meaning: Functions
                if trimmed.starts_with("fn ") || trimmed.starts_with("def ") || trimmed.starts_with("public void ") {
                    let parts: Vec<&str> = trimmed.split(|c| c == ' ' || c == '(').collect();
                    if parts.len() > 1 {
                        facts.push(FactTriple {
                            subject: file_name.clone(),
                            predicate: "defines_function".to_string(),
                            object: parts[1].to_string(),
                        });
                    }
                }

                // 2. Structural Meaning: Dependencies
                if trimmed.starts_with("use ") || trimmed.starts_with("import ") {
                    let parts: Vec<&str> = trimmed.split(|c| c == ' ' || c == ';').collect();
                    if parts.len() > 1 {
                        facts.push(FactTriple {
                            subject: file_name.clone(),
                            predicate: "depends_on".to_string(),
                            object: parts[1].to_string(),
                        });
                    }
                }
            }
        }

        debug!("CodeAnalysis: Extracted {} structural facts from {}", facts.len(), file_name);
        facts
    }
}
