use crate::{IngestionAdapter, SemanticChunk};
use std::path::Path;
use std::fs::File;
use std::io::Read;
use zip::read::ZipArchive;
use xml::reader::{EventReader, XmlEvent};
pub struct HtmlAdapter;
impl IngestionAdapter for HtmlAdapter {
    fn can_handle(&self, path: &Path) -> bool {
        matches!(path.extension().and_then(|s| s.to_str()), Some("html" | "htm"))
    }
    fn ingest(&self, path: &Path) -> Vec<SemanticChunk> {
        let content = match generic_read_to_string(path) { Ok(c) => c, Err(_) => return vec![] };
        let text = html2text::from_read(content.as_bytes(), 80);
        crate::chunk_content(&text, "html_text", path)
    }
}

pub struct DocxAdapter;
impl IngestionAdapter for DocxAdapter {
    fn can_handle(&self, path: &Path) -> bool {
        matches!(path.extension().and_then(|s| s.to_str()), Some("docx"))
    }
    fn ingest(&self, path: &Path) -> Vec<SemanticChunk> {
        let file = match File::open(path) { Ok(f) => f, Err(_) => return vec![] };
        let mut archive = match ZipArchive::new(file) { Ok(a) => a, Err(_) => return vec![] };

        let mut content = String::new();
        if let Ok(mut doc_file) = archive.by_name("word/document.xml") {
            let mut xml_content = String::new();
            if doc_file.read_to_string(&mut xml_content).is_ok() {
                let parser = EventReader::from_str(&xml_content);
                for e in parser {
                    if let Ok(XmlEvent::Characters(text)) = e {
                        content.push_str(&text);
                        content.push(' ');
                    }
                }
            }
        }

        if content.is_empty() { return vec![]; }
        crate::chunk_content(&content, "docx_text", path)
    }
}

pub struct MarkdownAdapter;
impl IngestionAdapter for MarkdownAdapter {
    fn can_handle(&self, path: &Path) -> bool {
        matches!(path.extension().and_then(|s| s.to_str()), Some("md" | "markdown"))
    }
    fn ingest(&self, path: &Path) -> Vec<SemanticChunk> {
        let content = match generic_read_to_string(path) { Ok(c) => c, Err(_) => return vec![] };
        // Basic Markdown parsing: split by headers
        let mut chunks = Vec::new();
        let mut current_section = "Root".to_string();
        let mut buffer = String::new();

        for line in content.lines() {
            if line.starts_with("# ") {
                if !buffer.trim().is_empty() {
                    chunks.extend(crate::chunk_content_with_structure(&buffer, "markdown_section", path, &current_section));
                    buffer.clear();
                }
                current_section = line[2..].trim().to_string();
            } else if line.starts_with("## ") {
                if !buffer.trim().is_empty() {
                    chunks.extend(crate::chunk_content_with_structure(&buffer, "markdown_subsection", path, &current_section));
                    buffer.clear();
                }
                current_section = line[3..].trim().to_string();
            } else {
                buffer.push_str(line);
                buffer.push('\n');
            }
        }
        if !buffer.trim().is_empty() {
            chunks.extend(crate::chunk_content_with_structure(&buffer, "markdown_text", path, &current_section));
        }
        chunks
    }
}

pub struct JsonAdapter;
impl IngestionAdapter for JsonAdapter {
    fn can_handle(&self, path: &Path) -> bool {
        matches!(path.extension().and_then(|s| s.to_str()), Some("json"))
    }
    fn ingest(&self, path: &Path) -> Vec<SemanticChunk> {
        let content = match generic_read_to_string(path) { Ok(c) => c, Err(_) => return vec![] };
        // Treat as structured data
        // For simplicity in this implementation, we treat top-level keys as sections if it's an object
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
            match json {
                serde_json::Value::Object(map) => {
                    let mut chunks = Vec::new();
                    for (k, v) in map {
                        let val_str: String = v.to_string();
                        chunks.extend(crate::chunk_content_with_structure(&val_str, "json_field", path, &format!("field:{}", k)));
                    }
                    return chunks;
                },
                _ => return crate::chunk_content(&content, "json_blob", path),
            }
        }
        crate::chunk_content(&content, "json_text", path)
    }
}

pub struct XmlAdapter;
impl IngestionAdapter for XmlAdapter {
    fn can_handle(&self, path: &Path) -> bool {
        matches!(path.extension().and_then(|s| s.to_str()), Some("xml"))
    }
    fn ingest(&self, path: &Path) -> Vec<SemanticChunk> {
        let content = match generic_read_to_string(path) { Ok(c) => c, Err(_) => return vec![] };
        let mut text = String::new();
        let parser = EventReader::from_str(&content);
        for e in parser {
            if let Ok(XmlEvent::Characters(c)) = e {
                text.push_str(&c);
                text.push(' ');
            }
        }
        crate::chunk_content(&text.trim(), "xml_text", path)
    }
}

pub struct PdfAdapter;
impl IngestionAdapter for PdfAdapter {
    fn can_handle(&self, path: &Path) -> bool {
        matches!(path.extension().and_then(|s| s.to_str()), Some("pdf"))
    }
    fn ingest(&self, path: &Path) -> Vec<SemanticChunk> {
        use lopdf::Document;
        let mut chunks = Vec::new();

        if let Ok(doc) = Document::load(path) {
            let pages = doc.get_pages();
            for (page_num, _) in pages {
                if let Ok(text) = doc.extract_text(&[page_num]) {
                    if !text.trim().is_empty() {
                        chunks.extend(crate::chunk_content_with_structure(
                            &text,
                            "pdf_page",
                            path,
                            &format!("Page {}", page_num)
                        ));
                    }
                }
            }

            // Extract metadata if available
            if let Ok(info) = doc.get_dictionary(doc.trailer.get(b"Info").and_then(|obj| obj.as_reference()).unwrap_or((0, 0))) {
                let mut meta_str = String::new();
                for (key, value) in info {
                    meta_str.push_str(&format!("{}: {:?}\n", String::from_utf8_lossy(key), value));
                }
                if !meta_str.is_empty() {
                    chunks.extend(crate::chunk_content_with_structure(&meta_str, "pdf_metadata", path, "metadata"));
                }
            }
        }

        chunks
    }
}

fn generic_read_to_string(path: &Path) -> std::io::Result<String> {
    let mut file = File::open(path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    Ok(content)
}
