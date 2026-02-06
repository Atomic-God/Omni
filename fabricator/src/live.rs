use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use notify::{Watcher, RecursiveMode, Result as NotifyResult, Event};
use engine::OmniMind;
use learning::LearningEngine;
use ingestion::ingest_path;
use log::{info, error, warn};

pub struct LiveFabricator {
    mind: Arc<Mutex<OmniMind>>,
    learning_engine: Arc<Mutex<LearningEngine>>,
}

impl LiveFabricator {
    pub fn new() -> Self {
        Self {
            mind: Arc::new(Mutex::new(OmniMind::new())),
            learning_engine: Arc::new(Mutex::new(LearningEngine::new())),
        }
    }

    pub fn watch(&self, path_str: &str) -> NotifyResult<()> {
        let path = PathBuf::from(path_str);
        if !path.exists() {
            warn!("Watch path does not exist: {:?}", path);
            return Ok(());
        }

        info!("Starting Live Fabricator on: {:?}", path);

        // Initial Ingestion
        self.ingest_and_learn(&path);

        // Setup Watcher
        let (tx, rx) = std::sync::mpsc::channel();
        let mut watcher = notify::recommended_watcher(tx)?;

        watcher.watch(&path, RecursiveMode::Recursive)?;

        info!("Watching for changes... (Press Ctrl+C to stop)");

        for res in rx {
            match res {
                Ok(event) => self.handle_event(event),
                Err(e) => error!("Watch error: {:?}", e),
            }
        }

        Ok(())
    }

    fn handle_event(&self, event: Event) {
        match event.kind {
            notify::EventKind::Create(_) | notify::EventKind::Modify(_) => {
                for path in event.paths {
                    if path.is_file() {
                        info!("Change detected: {:?}", path);
                        self.ingest_and_learn(&path);
                    }
                }
            },
            _ => {},
        }
    }

    fn ingest_and_learn(&self, path: &PathBuf) {
        let chunks = ingest_path(path.clone());
        if chunks.is_empty() { return; }

        let mut mind = self.mind.lock().unwrap();
        let mut engine = self.learning_engine.lock().unwrap();

        // Ensure we can learn (might need to unfreeze if reused, but new instance is default)
        // In live mode, we assume mutable.

        // Using internal learn method or exposing engine?
        // OmniMind has learn() which delegates to cognition.
        // LearningEngine has learn() which calls cognition.learn_text and manages consolidation.
        // We should use LearningEngine to drive OmniMind's cognition.

        engine.learn(&mut mind.cognition, chunks);
        info!("Live updated knowledge.");
    }
}
