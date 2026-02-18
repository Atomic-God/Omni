# Architecture: OmniForge v10.0 (Industrial Phase 1)

## Overview
A Neuro-Symbolic AI Runtime entirely in Rust. No Transformers, No Python Dependencies. Designed for sovereign industrial intelligence and lifelong learning.

## Crate Architecture
- **core-vsa**: HyperVector Algebra (10k-bit). Optimized bit-counting and noise resilience.
- **cognition**: Reasoning layer. Includes Knowledge Graph, Abductive Reasoner, and Belief Revision.
- **memory**: Hierarchical storage (Working, Episodic, Semantic). Uses LSH Indexing and atomic snapshots.
- **engine**: Orchestrates memory, cognition, and governance (OODA loop).
- **ingestion**: Universal data pipeline. Supports PDF, DOCX, Code, Audio, Video, and XML.
- **runtime**: Hardware Adaptation (HTE) and Performance Monitoring.
- **api/cli**: Command and service interfaces for the runtime.
- **fabricator**: High-level mind construction and mutation tools.

## Key Industrial Features
- **Deterministic:** 100% Reproducible seeds and hypervector generation.
- **Scalable:** Hardware adaptation layer scales VSA precision and memory shards based on live RAM/CPU profiling.
- **Safe:** Integrated `SecuritySandbox` for path validation and `PrivacyGuard` for data scrubbing.
- **Portable:** `MindPack` format with SHA-256 integrity, Gzip compression, and incremental delta support.

## Lifecycle Management
1. **Forge Mode**: Primary state for ingestion, high-speed learning, and base snapshot creation.
2. **Runtime Mode**: Distributed state for inference, task execution, and local incremental learning (delta).

## Continuous Learning
Implements the Ebbinghaus forgetting curve and background Sleep Cycles for memory consolidation, ensuring long-term cognitive stability.
