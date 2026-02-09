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
        // DOCX is a zip. We need to find word/document.xml and extract text.
        let file = match File::open(path) { Ok(f) => f, Err(_) => return vec![] };
        let mut archive = match ZipArchive::new(file) { Ok(a) => a, Err(_) => return vec![] };

        let mut content = String::new();
        if let Ok(mut doc_file) = archive.by_name("word/document.xml") {
            let mut xml_content = String::new();
            if doc_file.read_to_string(&mut xml_content).is_ok() {
                // Parse XML to extract text
                let parser = EventReader::from_str(&xml_content);
                for e in parser {
                    if let Ok(XmlEvent::Characters(text)) = e {
                        content.push_str(&text);
                        content.push(' ');
                    }
                }
            }
        }

        if content.is_empty() {
            return vec![];
        }

        crate::chunk_content(&content, "docx_text", path)
    }
}

fn generic_read_to_string(path: &Path) -> std::io::Result<String> {
    let mut file = File::open(path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    Ok(content)
}
