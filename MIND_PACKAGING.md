# Mind Packaging

Omni Forge uses a deterministic packaging system for distribution.

## Format: `.mindpack` (or `.omf`)
A Zip container holding:
1.  `mind.bin`: Serialized VSA Core (bincode).
2.  `metadata.json`: Versioning, hashes, license.
3.  `manifest.json`: Capability flags, resource requirements.
4.  `integrity.hash`: SHA256 signature of the core.

## Profiles
-   **Server:** Full precision, massive graph.
-   **Desktop:** Balanced, pruned weak connections.
-   **Mobile:** Quantized/Compressed, high pruning.

## Sharding
For large models, the graph is split into shards (e.g., `core`, `vocab`, `episodic`) to allow lazy loading or partial updates.
