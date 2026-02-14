use std::path::{Path, PathBuf};
use log::{info, warn, error};
use std::time::SystemTime;

pub struct SecurityManager {
    pub allow_network: bool,
    pub allow_external_exec: bool,
    pub sandbox_root: PathBuf,
}

impl SecurityManager {
    pub fn new(local_only: bool, sandbox_path: &Path) -> Self {
        Self {
            allow_network: !local_only,
            allow_external_exec: false, // Default strict
            sandbox_root: sandbox_path.to_path_buf(),
        }
    }

    pub fn validate_path(&self, path: &Path) -> bool {
        // Ensure path is inside sandbox root
        if let Ok(canon) = path.canonicalize() {
            if let Ok(root) = self.sandbox_root.canonicalize() {
                return canon.starts_with(root);
            }
        }
        // Fallback for non-existing paths (creation)
        // Check textually
        path.to_string_lossy().starts_with(&self.sandbox_root.to_string_lossy())
    }

    pub fn audit_ingestion(&self, file_size: u64, mime_type: &str) -> bool {
        // Limit ingestion
        if file_size > 100 * 1024 * 1024 { // 100MB limit
            warn!("Security: Rejected large file ingestion ({} bytes)", file_size);
            return false;
        }
        // Allowed types?
        true
    }

    pub fn enforce_local_only(&self) {
        if !self.allow_network {
            // In Rust, we can't easily block network syscalls without OS support (seccomp).
            // But we can enforce it in our logical modules (e.g. dont call reqwest).
            // This method serves as a global flag check.
            info!("Security: Local-Only mode active. Network features disabled.");
        }
    }
}
