# Omni Forge v5.1 Industrial Mind-Factory

Omni Forge is a compiler that builds Sovereign AI Minds. It is not a chatbot, a model wrapper, or a demo. It is a production-grade fabrication system for Neuro-Symbolic Artificial Intelligence using Vector Symbolic Architectures (VSA).

## Core Identity

*   **Sovereign:** Minds run offline, on-device, without cloud dependencies.
*   **Deterministic:** Fabrication is reproducible; Runtime is predictable.
*   **Neuro-Symbolic:** Combines statistical learning (BEAGLE) with symbolic reasoning (VSA/HDC).
*   **Industrial:** Built for large-scale ingestion, continuous learning, and immutable deployment.

## Architecture

Omni Forge enforces a strict **Fabrication-Runtime Duality**:

1.  **Fabricator (Compiler):**
    *   Ingests raw data (Text, Code, Logs, PDF, CSV, etc.).
    *   Learns structure and semantics.
    *   Consolidates memory (deduplication, pruning).
    *   Produces an immutable `.omf` artifact.
2.  **Runtime (Engine):**
    *   Loads `.omf` artifacts.
    *   Enforces READ-ONLY state (panics on mutation attempts).
    *   Executes reasoning queries (Multi-hop inference, Analogy).
    *   Runs on 400k AnTuTu class hardware.

## Quick Start

### 1. Fabricate a Mind
Ingest a dataset to build a new mind.
```bash
omni-forge fabricate ./my_dataset
```

### 2. Inspect the Artifact
Audit the metadata, OS/Arch compatibility, and hashes.
```bash
omni-forge inspect mind.omf
```

### 3. Run the Runtime
Enter the interactive read-only shell.
```bash
omni-forge run mind.omf
```

### 4. Single Query
Execute a single reasoning task.
```bash
omni-forge query mind.omf "What does system build?"
```

## Supported Inputs
*   **Text:** .txt, .md
*   **Code:** .rs, .py, .c, .cpp, .h, .json, .toml, .yaml
*   **Data:** .csv
*   **Streams:** stdin support via library API.

## Safety & Governance
*   **Lifecycle States:** `Fabricated` -> `Frozen` -> `Runtime`.
*   **Memory Safety:** Soft caps on relation graphs; 10MB chunking limits.
*   **Integrity:** SHA256 content deduplication; Core state hashing.
*   **OS Agnostic:** Path normalization and hardware detection (HTE) for Linux, macOS, Windows.

## License
Proprietary / Internal.
