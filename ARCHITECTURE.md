# Omni Forge Architecture
**Version 9.0 (Industrial)**

Omni Forge is a sovereign, neuro-symbolic AI Mind-Factory. It generates "Minds" that run on edge devices without external dependencies.

## Core Modules

### 1. Engine (`engine`)
The runtime orchestrator.
- **ForgeMind**: The Creator. Mutable, unlimited memory. Runs on Workstation.
- **RuntimeMind**: The Consumer. Immutable Base + Mutable Overlay. Runs on Edge.
- **ContextManager**: Arbitrates between Structural, Episodic, and Goal memory.

### 2. Cognition (`cognition`)
The reasoning core (Vector Symbolic Architecture).
- **CognitionCore**: Hypervector graph.
- **Grammar**: Clause analysis, conversation state.
- **Tools**: Abstract Action interface.

### 3. Perception (`perception`)
Input handling and synthesis.
- **Polyglot**: Multilingual analysis (frequency/morphology).
- **Encoders**: Text-to-Vector (SVO).

### 4. Memory (`memory`)
Persistence layer.
- **MindBlueprint (MindPack)**: Immutable base artifact.
- **MindDelta (PersonalMemory)**: User-specific learning.
- **Streaming**: Lazy loading interfaces.

### 5. Ingestion (`ingestion`)
Universal data pipeline.
- **Registry**: Dispatches parsers (MD, JSON, HTML, Code).
- **Structure**: Extracts hierarchy and relations.

### 6. Learning (`learning`)
Continuous improvement.
- **LearningEngine**: Entropy-based consolidation, drift protection.

## Safety & Governance
- **Immutability**: Base Mind is never modified by Runtime.
- **Sovereignty**: No telemetry. Local-first.
- **Deterministic**: Rebuilds guarantee identical vector spaces.
