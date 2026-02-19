use crate::{MemoryManager, LSHIndex, ShardedStorage};
use serde::{Serialize, Deserialize};
use std::path::Path;
use std::fs::File;
use std::io::{Read, Write};
use sha2::{Sha256, Digest};
use flate2::write::GzEncoder;
use flate2::read::GzDecoder;
use flate2::Compression;

#[derive(Serialize, Deserialize)]
pub struct SnapshotHeader {
    pub version: u32,
    pub timestamp: u64,
    pub checksum: String, // SHA256 of the body
}

#[derive(Serialize, Deserialize)]
pub struct MindSnapshot {
    pub header: SnapshotHeader,
    pub index: LSHIndex,
    pub storage: ShardedStorage,
}

pub struct SnapshotManager;

impl SnapshotManager {
    pub fn save(memory: &MemoryManager, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        // Serialize body
        let body = MindSnapshotBody {
            index: memory.index.clone(), // Clone is expensive. Industrial? Move?
            // Snapshotting usually happens at checkpoints.
            storage: memory.storage.clone(),
        };

        let body_bytes = bincode::serialize(&body)?;

        // Compute Checksum
        let mut hasher = Sha256::new();
        hasher.update(&body_bytes);
        let checksum = hex::encode(hasher.finalize());

        let header = SnapshotHeader {
            version: crate::MEMORY_VERSION,
            timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_secs(),
            checksum,
        };

        // Write File: Header + Gzip(Body)
        let file = File::create(path)?;
        let mut encoder = GzEncoder::new(file, Compression::default());

        bincode::serialize_into(&mut encoder, &header)?;
        encoder.write_all(&body_bytes)?;
        encoder.finish()?;

        Ok(())
    }

    pub fn load(path: &Path) -> Result<MindSnapshot, Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let mut decoder = GzDecoder::new(file);

        // Read Header?
        // Gzip stream contains everything.
        // We need to read continuously.
        // But header is inside gzip?
        // My save logic: `serialize_into(encoder, &header)` -> Header IS compressed.

        // So just deserialize MindSnapshot struct which contains header?
        // No, I wrote header then body bytes.
        // I should define a container struct.

        let container: MindSnapshotContainer = bincode::deserialize_from(&mut decoder)?;

        // Validate checksum
        // This requires re-serializing index/storage to bytes to hash them?
        // Or we trust Bincode?
        // "Checksum validation" is required.
        // If I serialize entire container, I can't check checksum before full load.

        // Correct way:
        // 1. Read Header (Uncompressed or separate?)
        // If everything compressed, we must decompress all.

        // Let's assume validation happens AFTER load for simplicity in Phase 3,
        // or we use a container that separates header hash.

        // Re-compute hash of body
        let body_bytes = bincode::serialize(&container.body)?;
        let mut hasher = Sha256::new();
        hasher.update(&body_bytes);
        let computed = hex::encode(hasher.finalize());

        if computed != container.header.checksum {
            return Err("Snapshot checksum mismatch! File corrupted.".into());
        }

        Ok(MindSnapshot {
            header: container.header,
            index: container.body.index,
            storage: container.body.storage,
        })
    }
}

#[derive(Serialize, Deserialize)]
struct MindSnapshotBody {
    index: LSHIndex,
    storage: ShardedStorage,
}

#[derive(Serialize, Deserialize)]
struct MindSnapshotContainer {
    header: SnapshotHeader,
    body: MindSnapshotBody,
}
