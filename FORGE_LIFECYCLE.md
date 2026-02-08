# Forge Lifecycle

The Omni Forge lifecycle defines how Minds are created, grown, and distributed.

## 1. Initialization (Forge Creation)
- **Command:** `omniforge init`
- **Output:** Creates a `forge_master.omf` (The "Parent").
- **State:** Empty, malleable, waiting for ingestion.

## 2. Ingestion (Growth)
- **Command:** `omniforge ingest <data>`
- **Process:**
  - Reads text/code/data from path.
  - Generates semantic chunks.
  - Performs **incremental learning** (VSA bundling/binding).
  - Updates the `forge_master.omf` (The "Parent").
- **Note:** The Forge grows continuously. No "retraining" needed.

## 3. Snapshot (Birth)
- **Command:** `omniforge snapshot <version>`
- **Output:** A frozen `.omf` Mind Artifact (The "Child").
- **State:** Immutable, portable, distributable.
- **Example:** `mind_v1.0.omf`

## 4. Runtime (Life)
- **Command:** `omniforge run <mind.omf>`
- **State:**
  - Loads the immutable base (`mind.omf`).
  - Creates/Loads a mutable overlay (`mind.omf.local`).
- **Function:**
  - Performs inference.
  - Learns new information LOCALLY into the overlay.
  - **Does NOT modify the original `mind.omf` or the `forge_master.omf`.**

## 5. Cloning & Sharing
- **Process:** Just copy the `.omf` file.
- **Result:** A new instance starts with the same base knowledge but zero personal memories.
- **Sovereignty:** Each clone learns independently.

## 6. Compression & Export
- **Command:** `omniforge compress` (Future)
- **Function:** Reduces vector precision or prunes weak connections for distribution.
- **Goal:** Optimize for specific hardware targets (Mobile vs Server).
