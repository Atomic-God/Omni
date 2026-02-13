# Memory Architecture

## 1. Layers
- **Working Memory:** High-speed, transient. Stores current observation and immediate context. High decay rate.
- **Episodic Memory:** Event-based. Stores sequence of actions/results. Medium decay.
- **Invariant Memory:** Fact-based. Stores high-confidence relations (e.g. from ingestion). Zero or low decay.

## 2. Storage Strategy
- **LSH Index:** In-memory Locality Sensitive Hashing for O(1) similarity search.
- **Sharded Storage:** Disk-based `shard_N.bin` files for payload storage.
- **Snapshots:** Gzipped Bincode files containing version header, checksum, and full state.

## 3. Integrity
- **Checksums:** SHA256 verification on snapshot load.
- **Recovery:** Fallback to last known good snapshot; shard integrity checks on boot.
