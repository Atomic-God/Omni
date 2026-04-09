use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LayoutElement {
    Header(usize, String),
    Paragraph(String),
    List(Vec<String>),
    Table(Vec<Vec<String>>),
    Section(String),
}

pub struct SemanticLayoutExtractor;

impl SemanticLayoutExtractor {
    pub fn extract_from_text(text: &str) -> Vec<LayoutElement> {
        let mut elements = Vec::new();
        let lines: Vec<&str> = text.lines().collect();
        let mut current_list = Vec::new();

        for line in lines {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                if !current_list.is_empty() {
                    elements.push(LayoutElement::List(current_list.clone()));
                    current_list.clear();
                }
                continue;
            }

            if trimmed.starts_with("#") {
                let level = trimmed.chars().take_while(|&c| c == '#').count();
                let content = trimmed.trim_start_matches('#').trim().to_string();
                elements.push(LayoutElement::Header(level, content));
            } else if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
                current_list.push(trimmed[2..].to_string());
            } else if trimmed.contains("|") && trimmed.split('|').count() > 2 {
                // Basic table stub
                let cells: Vec<String> = trimmed.split('|').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
                elements.push(LayoutElement::Table(vec![cells]));
            } else {
                if !current_list.is_empty() {
                    elements.push(LayoutElement::List(current_list.clone()));
                    current_list.clear();
                }
                elements.push(LayoutElement::Paragraph(trimmed.to_string()));
            }
        }

        if !current_list.is_empty() {
            elements.push(LayoutElement::List(current_list));
        }

        elements
    }
}
