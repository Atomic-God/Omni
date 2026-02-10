# Runtime Architecture
**Sovereign Execution & Adaptation**

## The Runtime Mind
The `RuntimeMind` is the consumer-side entity.
- **Base**: Immutable `MindPack` (Arc).
- **Overlay**: Mutable `PersonalMemory`.
- **Learner**: `LearningEngine` (Continuous).
- **Adapter**: `RuntimeAdapter` (Hardware-aware).

## Adaptation Strategy
On startup, `RuntimeAdapter` detects:
1.  **Memory**: Sets max context size.
2.  **ISA**: Selects SIMD width (AVX2/AVX512/Neon).
3.  **Concurrency**: Threads pool size.

## Execution Policy
- **LowPower**: Single-thread, lazy loading.
- **Balanced**: Default.
- **HighPerformance**: Max concurrency, pre-loading.

## Context Persistence
The Context Manager state (Goals, Episodic Buffer, Focus) is serialized into the `.local` overlay on shutdown/save.
This ensures user sessions resume seamlessly.
