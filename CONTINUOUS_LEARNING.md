# Continuous Learning Architecture

Omni Forge supports "Learning without Restart".

## The Additive Model
- New knowledge is *bundled* into the existing Hypervector space.
- Old knowledge is not overwritten, but can be "decayed" via weight reduction.

## The Learning Engine
Runs inside `RuntimeMind`.
1.  **Ingest**: Takes `SemanticChunk` (User text).
2.  **Entropy Check**: Measures global entropy change. High drift triggers warning.
3.  **Learn**: Updates `CognitionCore` graph.
4.  **Consolidate**:
    - Runs periodically or on high entropy.
    - Prunes weak relations (>50 per concept).
    - Decays salience.

## Sovereignty
All learning is stored in `MindDelta` (`.local` file).
The Base Mind (`.mindpack`) remains untouched.
