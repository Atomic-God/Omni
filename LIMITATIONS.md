# System Limitations (Phase 1)

This document lists the explicit limitations of the current CPU-only Omni Forge implementation.

## 1. Perception
- **Vision:** Not implemented. Stubs exist in `perception::VisionModality`.
- **Audio:** Not implemented. Stubs exist in `perception::AudioModality`.
- **PDFs:** Basic text extraction is not integrated; `PdfAdapter` is a placeholder.

## 2. Scale
- **Memory:** Entire Knowledge Graph must fit in RAM. No disk-based paging (AirLLM style) yet.
- **Concurrency:** `ForgeMind` uses a global `Mutex`. Heavy parallel ingestion may contend.

## 3. Reasoning
- **Depth:** Multi-hop inference is limited to 3 steps to ensure low latency on CPUs.
- **Math:** No symbolic math solver (e.g., integrals).
- **Code Generation:** Can parse code structure but cannot write complex executable code.

## 4. Language
- **Translation:** No translation capability.
- **Script Support:** Tokenizer is Unicode-aware, but semantic understanding relies on structural similarity, not pretrained embeddings. Ancient languages require manual grounding or massive structural corpus.

## 5. Deferrals (Phase 2)
The following features are explicitly deferred to the GPU/Kaggle phase:
- Implicit Neural Representations (INR) for images.
- Resonator Networks for high-speed factorization.
- Massive-scale graph consolidation.
- Self-improving reflection loops (beyond basic stub).
