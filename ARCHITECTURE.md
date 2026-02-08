# Omni Forge Architecture

## Core Philosophy
Omni Forge is a neuro-symbolic AI fabrication system designed to produce sovereign, portable, and continuously learning Minds without relying on pretrained neural networks or cloud dependencies. It operates on the principles of **Vector Symbolic Architectures (VSA)** and **Recursive Sparse Holography (RSSH)**.

## System Components

### 1. Fabrication Layer (The Forge)
- **Role:** Creator and Teacher.
- **State:** Mutable, Unlimited Memory.
- **Function:** Ingests raw data, builds high-dimensional semantic spaces, and produces immutable **Mind Snapshots**.
- **Key Crates:** `fabricator`, `ingestion`, `learning`, `engine` (ForgeMind).

### 2. Runtime Layer (The Sovereign)
- **Role:** Consumer and Explorer.
- **State:** Immutable Base + Mutable Overlay.
- **Function:** Loads a snapshot, performs inference, and learns locally via a **Personal Overlay** (Delta).
- **Key Crates:** `engine` (RuntimeMind), `cognition`, `memory`.

### 3. Memory Architecture
- **HyperVector:** 10,000-dimensional bitwise vectors (Holographic representation).
- **MindPack (Snapshot):** A frozen, versioned artifact containing the Core Knowledge Graph and Semantic Indexes.
- **PersonalMemory (Overlay):** A local, append-only delta structure for runtime learning.
- **Key Crates:** `core-vsa`, `memory`.

## Data Flow

1. **Ingestion:** Raw data (Text, Code, etc.) -> `ingestion` -> `SemanticChunk`.
2. **Learning:** `SemanticChunk` -> `learning` -> `CognitionCore` (Forge).
3. **Fabrication:** `ForgeMind` -> `snapshot()` -> `.omf` Artifact.
4. **Runtime:** `.omf` -> `RuntimeMind` (Base) + `Overlay`.
5. **Inference:** Query -> `IntentParser` -> `CognitionCore` -> Response.
6. **Continuous Learning:** New Data -> `RuntimeMind` -> `Overlay` (Append).

## Hardware Truth Engine (HTE)
- Detects CPU features (AVX, NEON) and adjusts vector operations.
- Monitors memory pressure and thermal state to ensure stability on edge devices.
