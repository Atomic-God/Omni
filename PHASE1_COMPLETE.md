# OmniForge Phase 1: Industrial Core Complete

## 1. Executive Summary
OmniForge Phase 1 "Industrial Core" is a production-grade, offline-sovereign Neuro-Symbolic AI runtime.
It bridges Vector Symbolic Architectures (VSA) with a custom, CPU-based Neural Stack (RNN/Dense) for continuous learning and reasoning.

## 2. Core Capabilities
- **Universal Ingestion:** Supports PDF, XLSX, CSV, Code, Images (Metadata), Archives with content deduplication and relational graph construction.
- **Cognitive Loop:** OODA (Observe-Orient-Decide-Act) loop with Goal Tracking, Intent Resolution, and Energy Budgeting.
- **Memory Intelligence:** Hierarchical Memory (Working/Episodic/Invariant) with importance-based consolidation and binary snapshot persistence.
- **Neural Stack:** Trainable `SequenceModel` (RNN) with SGD/Adam optimizers, capable of sequence generation and embedding mapping.
- **Hardware Agnostic:** Runtime Hardware Truth Engine (HTE) adapts to CPU/Memory constraints.

## 3. Architecture
- **Language:** Rust (Stable)
- **Persistence:** Bincode + Gzip (Versioned Snapshots)
- **Neural Backend:** Custom `neural` crate (Rayon-accelerated CPU)
- **Reasoning:** `core-vsa` (10k-bit HyperVectors) + `cognition` (Abductive/Planning)

## 4. Usage
See `USING_OMNIFORGE.md` or run `omniforge --help`.
