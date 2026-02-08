# Continuous Learning

Omni Forge implements continuous learning using an **Entropy-Based Consolidation** strategy.

## 1. Learning Boundary
- **Input:** Semantic Chunks (Text, Code, etc.)
- **Process:**
  - Ingests new data into `CognitionCore`.
  - Calculates global entropy of the relation graph.
  - If entropy spikes (high novelty/uncertainty), triggers consolidation.
  - Otherwise, continues streaming.

## 2. Memory Structure
- **Index Memory:** (Fast) Term lookups.
- **Semantic Memory:** (Deep) Meaning vectors (Context).
- **Relation Graph:** (Structured) Logic and causality.
- **Sentence Memory:** (Episodic) Raw event storage.

## 3. Consolidation Logic
- **Trigger:** Time interval (300s default) OR High Entropy Spike.
- **Deduplication:** Merges redundant relations, keeping strongest.
- **Decay:** Relations weaken over time unless reinforced.
- **Pruning:**
  - Removes weak or redundant relations to maintain cognitive health.
  - Limits concepts per node to a soft cap (default 50).
  - Prioritizes recent or heavily-weighted relations.

## 4. Runtime Adaptation
- **Personal Overlay (Delta):**
  - Stores all NEW learning in runtime.
  - Linked to the parent snapshot via hash.
  - Allows **Sovereign Growth** without modifying the base.
  - Can be saved (`/save`), loaded, or discarded.
- **Base Knowledge:** Always read-only.

## 5. Avoiding Catastrophic Forgetting
- **Base Stability:** The `ForgeMind` (Parent) is immutable during runtime.
- **Overlay Isolation:** New learning affects only the overlay.
- **Conflict Resolution:** Runtime queries prioritize local overlay, then fallback to base.
