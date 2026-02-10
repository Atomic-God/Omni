use crate::{IngestionAdapter, SemanticChunk};
use std::path::Path;
use log::info;

pub struct DataIngestionRegistry {
    adapters: Vec<Box<dyn IngestionAdapter>>,
}

impl Default for DataIngestionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl DataIngestionRegistry {
    pub fn new() -> Self {
        Self {
            adapters: Vec::new(),
        }
    }

    pub fn register<A: IngestionAdapter + 'static>(&mut self, adapter: A) {
        self.adapters.push(Box::new(adapter));
    }

    pub fn ingest(&self, path: &Path) -> Vec<SemanticChunk> {
        for adapter in &self.adapters {
            if adapter.can_handle(path) {
                info!("Dispatching ingestion for {:?} to adapter", path);
                return adapter.ingest(path);
            }
        }
        // No specific adapter found, try generic fallback logic or return empty
        // The calling code (ingest_path) usually handles fallback for archives/binaries,
        // but ideally we'd have a FallbackAdapter here too.
        vec![]
    }
}
