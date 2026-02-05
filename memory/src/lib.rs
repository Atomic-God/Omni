use cognition::CognitionCore;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufReader;

const MEMORY_VERSION: &str = "1.0";

#[derive(Serialize, Deserialize)]
struct VersionedMemory {
    version: String,
    core: CognitionCore,
}

pub fn save(core: &CognitionCore, path: &str) -> Result<(), std::io::Error> {
    let container = VersionedMemory {
        version: MEMORY_VERSION.to_string(),
        // Clone is expensive, but for prototype it's safe.
        // Ideally serialize reference, but serde struct requires ownership or lifetime.
        // CognitionCore is Clone-able via its fields? No, HashMap and Vec are, but standard derive Serialize implies ownership often unless ref.
        // Let's rely on standard serialization of the struct fields.
        // To avoid clone, we can implement custom serializer or just accept the cost.
        // Or change VersionedMemory to hold a reference: `core: &'a CognitionCore`.
        core: core.clone(),
    };
    let file = File::create(path)?;
    serde_json::to_writer(file, &container)?;
    Ok(())
}

pub fn load(path: &str) -> Result<CognitionCore, std::io::Error> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let container: VersionedMemory = serde_json::from_reader(reader)?;

    if container.version != MEMORY_VERSION {
        // Handle migration if needed in future
        println!(
            "Warning: Memory version mismatch. Loaded: {}, Current: {}",
            container.version, MEMORY_VERSION
        );
    }

    Ok(container.core)
}
