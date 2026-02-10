# Forge Lifecycle
**From Ingestion to Sovereign Runtime**

1.  **Initialize (`init`)**
    - Creates `./omniforge_data/`
    - Initializes Master Forge.

2.  **Ingest (`ingest`)**
    - Scans directory.
    - Parsers (MD, Code, PDF) extract hierarchy.
    - `CognitionCore` learns relations.

3.  **Fabricate/Snapshot (`snapshot`)**
    - Freezes the Forge state.
    - Optimizes vectors (future: quantization).
    - Writes `MindBlueprint` (.mindpack).

4.  **Distribute (`compress`)**
    - Bundles artifact for transport.

5.  **Run (`run`)**
    - Loads Blueprint (Read-Only).
    - Creates `MindDelta` (Overlay).
    - `LearningEngine` runs continuously.

6.  **Evolve**
    - User teaches Runtime.
    - Delta grows.
    - Forge continues evolving separately.
