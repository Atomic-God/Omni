# Memory Architecture

## Hierarchy
1.  **Working Memory:** Transient, high-speed.
2.  **Episodic Memory:** Event history.
3.  **Invariant Memory:** Stable facts.

## Storage
- **LSH Index:** O(1) similarity search.
- **Sharded Storage:** `shard_N.bin` on disk.
- **Layered Memory:**
    - `Base`: Immutable shared knowledge.
    - `Delta`: Personal adaptations.

## Persistence
- Binary snapshots (Bincode + Gzip).
- SHA256 Checksums.
