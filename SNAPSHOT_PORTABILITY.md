# Snapshot Portability & MindPacks

OmniForge uses a robust, portable format for sharing and backing up cognitive states.

## The MindPack Format
A `.mindpack` file is a Gzipped Bincode archive containing:
- **Snapshots**: Full or delta cognitive states.
- **Manifest**: Versioning and metadata about the mind.
- **Integrity**: SHA-256 hashes for every shard and metadata entry.

## Industrial Features
- **Delta Support**: Save only the changes between mind versions to reduce storage footprint.
- **Atomic Saves**: Uses transactional renames to ensure snapshots are never partially written or corrupted.
- **Encryption**: Optional XOR-based transformation (Phase-1) for basic content obfuscation.
- **Sequential Restore**: `incremental_restore` can apply a chain of delta snapshots to reach the latest state.

## Usage
MindPacks can be exported and imported across different hardware profiles. The system automatically adjusts VSA precision (e.g., truncating 10k-bit vectors to 2k-bit) if the target device is in Low-Memory mode.
