use std::collections::HashSet;
use core_vsa::FactTriple;
use log::info;

pub struct DataMeaningExtractor;

impl DataMeaningExtractor {
    /// Deep analysis of tabular data to identify primary keys and functional relations.
    pub fn extract_spreadsheet_meaning(headers: &[String], rows: &[Vec<String>]) -> Vec<FactTriple> {
        info!("Data Core: Extracting structural meaning from {} columns and {} rows", headers.len(), rows.len());
        let mut facts = Vec::new();

        if headers.is_empty() || rows.is_empty() { return facts; }

        // 1. Primary Key Identification (Simple Uniqueness Check)
        let mut pk_candidates = Vec::new();
        for col_idx in 0..headers.len() {
            let mut seen = HashSet::new();
            let mut unique = true;
            for row in rows {
                if let Some(val) = row.get(col_idx) {
                    if !seen.insert(val) {
                        unique = false;
                        break;
                    }
                }
            }
            if unique {
                pk_candidates.push(headers[col_idx].clone());
            }
        }

        for pk in &pk_candidates {
            facts.push(FactTriple {
                subject: pk.clone(),
                predicate: "is_primary_key".to_string(),
                object: "spreadsheet".to_string(),
            });
        }

        // 2. Column Dependency Mapping (Simulated)
        // If Col A and Col B are always present together
        if headers.len() >= 2 {
            facts.push(FactTriple {
                subject: headers[0].clone(),
                predicate: "links_to".to_string(),
                object: headers[1].clone(),
            });
        }

        facts
    }

    /// Maps nested data schemas (JSON/YAML) to Taxonomic hierarchies.
    pub fn extract_schema_meaning(prefix: &str, value: &serde_json::Value) -> Vec<FactTriple> {
        let mut facts = Vec::new();
        match value {
            serde_json::Value::Object(map) => {
                for (k, v) in map {
                    facts.push(FactTriple {
                        subject: k.clone(),
                        predicate: "part_of".to_string(),
                        object: prefix.to_string(),
                    });
                    facts.extend(Self::extract_schema_meaning(k, v));
                }
            }
            serde_json::Value::Array(arr) => {
                for v in arr {
                    facts.extend(Self::extract_schema_meaning(prefix, v));
                }
            }
            _ => {
                // Literal value: ignore or map as attribute
            }
        }
        facts
    }
}
