# Security & Sovereignty

Omni Forge is designed with strict sovereignty and security principles.

## 1. Fabrication vs. Runtime Separation
- **Forge (Creator):** Has unlimited memory and write access to the Master Mind. Never exposed to the edge.
- **Runtime (Consumer):** Loads an **immutable** snapshot. Cannot modify the base artifact. Learning is restricted to a local, append-only overlay.

## 2. Deterministic Integrity
- **Artifact Hashing:** Every `.omf` snapshot includes a `core_hash` (SHA256) of its knowledge graph.
- **Verification:** The runtime calculates the hash on load. If it mismatches the metadata, loading fails (`Integrity Violation`).
- **Reproducibility:** `core-vsa` uses deterministic seeding for hypervectors. The same input sequence always produces the same Mind.

## 3. Capability Sandboxing (Phase 1)
- **File Access:** Runtime is restricted to its own configuration and overlay directories.
- **Network:** No network code exists in the `engine` or `cognition` crates. The `api` crate is optional and explicit.
- **Recursion Limits:** Inference depth is capped (default 3 hops) to prevent Denial of Service (DoS) via cyclic graphs.
- **Memory Budget:** `ContextManager` enforces hard limits on working memory items (Miller's Law: 7 +/- 2) and episodic buffer size.

## 4. Privacy
- **No Telemetry:** Omni Forge contains zero telemetry code.
- **Local Only:** All learning and inference happen on the local CPU.
- **Encryption:** Snapshots can be distributed as standard files; users are responsible for filesystem encryption (e.g., LUKS, BitLocker).

## 5. Audit Logging
- **Action Log:** High-level actions (Learn, Snapshot, Ingest) are logged via `log` crate (stdout/stderr).
- **Transparency:** `omniforge inspect` reveals the full metadata, including the compiler version and source hash.
