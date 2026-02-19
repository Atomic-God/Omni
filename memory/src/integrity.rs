use crate::{load_snapshot, MindPack};
use log::{info, warn};

pub fn verify_integrity(path: &str) -> Result<bool, String> {
    info!("Verifying integrity of {}", path);

    // 1. Load the pack (checks structural validity and metadata/integrity.hash match)
    let pack = match load_snapshot(path) {
        Ok(p) => p,
        Err(e) => return Err(format!("Load failed: {}", e)),
    };

    // 2. Deep Verification: Recompute hash from loaded core
    let computed_hash = pack.memory.core.compute_integrity_hash();

    if computed_hash != pack.metadata.core_hash {
        warn!("Deep Verification FAILED!");
        warn!("Metadata claims: {}", pack.metadata.core_hash);
        warn!("Actual content:  {}", computed_hash);
        return Err("Content mismatch (Corruption detected)".to_string());
    }

    info!("Deep Verification PASSED. Content hash matches metadata.");
    Ok(true)
}
