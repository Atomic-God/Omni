use std::path::{Path, PathBuf};
use log::warn;

pub struct PermissionBoundary {
    pub allowed_paths: Vec<PathBuf>,
    pub allow_network: bool,
    pub max_memory_mb: usize,
}

impl Default for PermissionBoundary {
    fn default() -> Self {
        Self {
            allowed_paths: Vec::new(),
            allow_network: false,
            max_memory_mb: 1024, // 1GB default
        }
    }
}

impl PermissionBoundary {
    pub fn new(allowed_root: PathBuf) -> Self {
        Self {
            allowed_paths: vec![allowed_root],
            allow_network: false,
            max_memory_mb: 1024,
        }
    }

    pub fn check_path(&self, path: &Path) -> Result<(), String> {
        let canonical = match path.canonicalize() {
            Ok(p) => p,
            Err(_) => return Err("Invalid path or does not exist".to_string()),
        };

        for allowed in &self.allowed_paths {
            if canonical.starts_with(allowed) {
                return Ok(());
            }
        }

        warn!("Security Violation: Access denied to {:?}", path);
        Err("Access Denied: Path outside sandbox".to_string())
    }

    pub fn check_network(&self) -> Result<(), String> {
        if self.allow_network {
            Ok(())
        } else {
            Err("Access Denied: Network disabled".to_string())
        }
    }
}
