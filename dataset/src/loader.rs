use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use log::info;

pub struct StreamingLoader {
    files: Vec<PathBuf>,
}

impl StreamingLoader {
    pub fn new(root: &Path) -> Self {
        let mut files = Vec::new();
        for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                if let Some(ext) = entry.path().extension().and_then(|s| s.to_str()) {
                    match ext {
                        "txt" | "md" | "rs" | "py" | "json" | "csv" => files.push(entry.path().to_path_buf()),
                        _ => {}
                    }
                }
            }
        }
        info!("Found {} files in dataset.", files.len());
        Self { files }
    }

    pub fn iter(&self) -> impl Iterator<Item = String> + '_ {
        self.files.iter()
            .map(|p| std::fs::read_to_string(p))
            .filter_map(Result::ok)
            .flat_map(|s| {
                s.lines()
                    .filter(|l| !l.trim().is_empty())
                    .map(String::from)
                    .collect::<Vec<String>>()
                    .into_iter()
            })
    }
}
