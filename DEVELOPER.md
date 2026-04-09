# OmniForge Developer Documentation (Phase-1 Industrial Core)

## Overview
OmniForge is an industrial-grade neuro-symbolic cognitive runtime based on Vector Symbolic Architectures (VSA). It is designed to be hardware-agnostic, sovereign, and capable of lifelong learning without transformers or pretrained neural models.

## Crate Architecture
- **core-vsa**: The mathematical foundation. Implements Binary Spatter Codes (BSC) HyperVectors.
- **cognition**: The reasoning engine. Handles Knowledge Graphs, abductive inference, and planning.
- **memory**: Hierarchical storage. Manages Working, Episodic, and Semantic memory layers.
- **engine**: The orchestration layer. Ties together memory, cognition, and hardware adaptation.
- **ingestion**: Universal data pipeline. Extracts symbolic meaning from PDF, DOCX, Code, Audio, and Video.
- **api/cli**: External interfaces for the runtime.

## Key Industrial Features
1. **Universal Ingestion**: Deep meaning extraction using symbolic NLP and code analysis.
2. **Knowledge Graph Layer**: Weighted causal links, contradiction detection, and trust scoring.
3. **Continuous Learning**: Ebbinghaus forgetting curve, reinforcement weighting, and sleep-cycle consolidation.
4. **Hardware Adaptation (HTE)**: Live thermal/load profiling and dynamic VSA scaling.
5. **Security & Governance**: Prompt injection defense, privacy scrubbing, and path sandboxing.
6. **MindPack snapshots**: Portable, compressed, SHA-256 verified cognitive states with delta support.

## Developing
- **Lints**: `#![deny(warnings)]` is enforced in all core crates.
- **Logging**: Uses the `tracing` crate for structured logging. Set `RUST_LOG` to control levels.
- **Testing**: Use `cargo test --workspace` for integration tests and `cargo bench -p engine` for stress tests.
- **Release**: Optimized with LTO and codegen-units for production stability.

## Task Loop
OmniForge supports autonomous multi-step task execution through the `TaskLoop` module in the `engine` crate. Goals are tracked persistently and updated through the cognitive reasoning cycle.

---
*OmniForge Phase-1: Industrial Readiness Achieved.*
