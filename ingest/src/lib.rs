use engine::OmniMind;
use log::{error, info};
use notify::{Config, RecommendedWatcher, RecursiveMode, Result, Watcher};
use std::fs;
use std::path::Path;
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use std::thread;
use walkdir::WalkDir;

pub fn ingest_path(path: &str, mind: &mut OmniMind) {
    info!("Ingesting directory: {}", path);
    for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        if entry.path().is_file() {
            if let Some(ext) = entry.path().extension() {
                if ext == "txt" {
                    if let Ok(content) = fs::read_to_string(entry.path()) {
                        info!("Learning from file: {:?}", entry.path());
                        // Simple cleanup: replace newlines with spaces
                        let clean_text = content.replace('\n', " ");
                        mind.learn(&clean_text);
                    }
                }
            }
        }
    }
}

pub struct DirectoryWatcher {
    #[allow(dead_code)]
    watcher: RecommendedWatcher,
}

impl DirectoryWatcher {
    pub fn new(path: String, mind: Arc<Mutex<OmniMind>>) -> Result<Self> {
        let (tx, rx) = channel();

        let mut watcher = notify::RecommendedWatcher::new(tx, Config::default())?;

        watcher.watch(Path::new(&path), RecursiveMode::Recursive)?;

        thread::spawn(move || {
            for res in rx {
                match res {
                    Ok(event) => {
                        // Check if file creation or modification
                        match event.kind {
                            notify::EventKind::Create(_) | notify::EventKind::Modify(_) => {
                                for path in event.paths {
                                    if path.is_file()
                                        && path.extension().map_or(false, |e| e == "txt")
                                    {
                                        if let Ok(content) = fs::read_to_string(&path) {
                                            if let Ok(mut mind_lock) = mind.lock() {
                                                info!(
                                                    "Watcher detected change in {:?}, learning...",
                                                    path
                                                );
                                                let clean_text = content.replace('\n', " ");
                                                mind_lock.learn(&clean_text);
                                            }
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    Err(e) => error!("Watch error: {:?}", e),
                }
            }
        });

        Ok(Self { watcher })
    }
}
