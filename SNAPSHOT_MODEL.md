# Snapshot Model

## Core Principle
A **Mind Snapshot** is an immutable, versioned artifact representing a coherent state of intelligence. It is the "gold master" from which runtime instances derive.

## Structure (MindPack)

### 1. Metadata (`metadata.json`)
- **Version:** Internal Schema Version (e.g., "8.2").
- **Core Hash:** SHA256 of the cognitive core (integrity).
- **Source:** Origin of the data (e.g., "en_wiki_dump").
- **Timestamp:** Fabrication time.
- **State:** `Frozen` or `Fabricated`.
- **Compiler Version:** The Omni Forge version used to build it.
- **Semantic Version:** User-defined version (e.g., "1.0.0").

### 2. Cognitive Core (`mind.bin`)
- **Serialized VSA Graph:**
  - **Index Memory:** Term-to-HyperVector mapping.
  - **Semantic Memory:** Contextual meaning vectors.
  - **Relation Graph:** Weighted edges (Causal, Temporal, etc.).
- **Format:** `bincode` (Compact, fast binary serialization).

### 3. Vocabulary (`words.bin`)
- **Separate Index:** Can be loaded independently for heavy memory optimization (future).
- **Format:** `bincode`.

### 4. Configuration (`schema.json`, `limits.json`)
- **Encoder Config:** Defines the VSA model used (e.g., "beagle-v5").
- **Learning Policies:** Defines limits (e.g., `max_concepts`) for runtime overlays.

## Integrity
- Every snapshot includes an `integrity.hash` file.
- Runtime verifies this hash on load. **If mismatch, load fails.**
- Snapshots are designed to be **write-once, read-many**.

## Cloning
- Cloning is simply copying the `.omf` file.
- Since it is immutable, multiple runtime instances can safely read from the same file.
