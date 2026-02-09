# The Universal Mind Contract (UMC)
**Specification v1.0**

## 1. The Mind Pack Artifact
This document defines the structure and compatibility of the `.omf` (Omni Forge) file format.

### 1.1 Components
A valid Mind Pack MUST contain:
1.  **metadata.json**: Identity, version, hash, source hardware info.
2.  **limits.json**: Learning policies (max concepts, decay rate).
3.  **schema.json**: Encoder configuration (model name).
4.  **mind.bin**: The serialized `CognitionCore` (binary, platform-agnostic).
5.  **integrity.hash**: SHA-256 hash of the core for tamper verification.
6.  **blueprint.json** (Optional): Instructions for instantiation.

### 1.2 Versioning
*   **Memory Version:** Currently **v8.4**. All Runtimes MUST support v8.3 or v8.4.
*   **Forward Compatibility:** Runtimes MAY ignore unknown fields in JSON metadata.
*   **Backward Compatibility:** Runtimes MUST support loading older `.omf` files via migration layers (if implemented).

### 1.3 Platform Agnosticism
*   **Endianness:** All binary data (mind.bin) is serialized in **Little Endian** (bincode default).
*   **Architecture:** No hardware-specific optimizations (SIMD/AVX) are stored in the artifact. All optimizations are runtime-detected.
*   **Floating Point:** Hypervectors use standard IEEE 754 floats.

## 2. The Verification Protocol
1.  **Hash Verification:** The Runtime MUST compute the SHA-256 hash of `mind.bin` and compare it with `integrity.hash` before execution.
2.  **Schema Check:** The Runtime MUST verify that `encoder_config` matches its available perception modules.
3.  **Policy Enforcement:** The Runtime MUST respect `learning_policies` (e.g., max_concepts) during execution.

## 3. The Personal Overlay Extension
*   Personal Overlays are separate files (`.local`) that extend the Base Mind.
*   They MUST reference the `parent_hash` of the Base Mind they were created from.
*   If the `parent_hash` does not match, the Overlay is **invalid** and must be re-based or discarded.

---
*This contract ensures that a Mind created on a Cloud Workstation can run unmodified on a Mobile Device.*
