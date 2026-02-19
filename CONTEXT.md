# Context System Industrialization
**State Management & Arbitration**

Omni Forge uses a multi-tiered Context Manager to arbitrate attention and memory.

## Architecture

### 1. Memory Tiers
- **Short-Term Memory (STM)**: Capacity ~7 items (Miller's Law). Immediate activation.
- **Episodic Buffer**: Log of recent interactions (~100 items).
- **Structural Focus**: High-salience pointers to long-term knowledge graph.
- **Goal Memory**: Stack of active objectives.

### 2. Arbitration Logic
The `ContextManager` merges these tiers into a single `active_context` vector:
1.  **Goals**: Always top priority.
2.  **Focus**: Concepts with high `SalienceScoring`.
3.  **Recency**: Last 3-5 episodic items.

### 3. Salience & Decay
- Every concept has a `salience` score.
- Scores decay exponentially over time (default: 5% per hour).
- Activation (usage) boosts score +1.0.

### 4. Persistence
Context state is **serialized** into the `PersonalMemory` overlay (`.local` file).
This ensures that restarting the Mind restores the exact train of thought and active goals.
