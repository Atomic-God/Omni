use std::collections::{HashMap, HashSet};
use core_vsa::HyperVector;
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StructuralMemoryEntry {
    pub id: String, // Hash
    pub content: String,
    pub source: String,
    pub timestamp: u64,
    pub vector: Option<HyperVector>,
    pub metadata: HashMap<String, String>,
    pub relations: Vec<String>, // IDs of related entries
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct RelationalIndex {
    pub entries: HashMap<String, StructuralMemoryEntry>,
    pub tag_index: HashMap<String, Vec<String>>,
    pub source_index: HashMap<String, Vec<String>>,
}

impl RelationalIndex {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, entry: StructuralMemoryEntry) {
        // Update indices
        self.source_index.entry(entry.source.clone()).or_default().push(entry.id.clone());

        if let Some(type_tag) = entry.metadata.get("structure_type") {
            self.tag_index.entry(type_tag.clone()).or_default().push(entry.id.clone());
        }

        self.entries.insert(entry.id.clone(), entry);
    }

    pub fn find_by_source(&self, source: &str) -> Vec<&StructuralMemoryEntry> {
        self.source_index.get(source).map(|ids| ids.iter().filter_map(|id| self.entries.get(id)).collect()).unwrap_or_default()
    }

    pub fn link_related(&mut self, id_a: &str, id_b: &str) {
        if let Some(entry_a) = self.entries.get_mut(id_a) {
            if !entry_a.relations.contains(&id_b.to_string()) {
                entry_a.relations.push(id_b.to_string());
            }
        }
        if let Some(entry_b) = self.entries.get_mut(id_b) {
            if !entry_b.relations.contains(&id_a.to_string()) {
                entry_b.relations.push(id_a.to_string());
            }
        }
    }
}
