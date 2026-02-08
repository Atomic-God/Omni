# Security Model

## 1. Sovereignty
-   **Local Execution:** No data leaves the device.
-   **No Telemetry:** Disabled by default and by design.

## 2. Integrity
-   **Signed Snapshots:** Artifacts are hashed (SHA256). Runtime verifies hash on load.
-   **Immutable Base:** The base knowledge graph is read-only memory mapped.

## 3. Sandboxing
-   **Filesystem:** Runtime can only read from config/data dirs and write to specific overlay paths.
-   **Network:** Blocked by default. `PermissionBoundary` struct enforces this.

## 4. Isolation
-   **Clone Independence:** Clones share a base but have distinct, encrypted overlay files.
-   **Memory Safety:** Rust's ownership model prevents buffer overflows.
