# GPU Extension Hooks
**Phase 2 Preparation**

Omni Forge v9.0 is CPU-only. The following hooks exist for GPU acceleration:

## 1. Perception Backend
**Trait**: `TensorBackend` (`perception/src/gpu_interfaces.rs`)
**Target**: `wgpu` or `tch-rs`.
**Usage**:
- Replace `TextEncoder` stub with Transformer-based embedding.
- Implement `InrInterface` for visual synthesis.

## 2. Hypervector Operations
**Trait**: `SimdBackend` (`hte/src/dispatch.rs`)
**Target**: CUDA Kernels.
**Usage**:
- Offload `bundle` (add) and `bind` (xor) for large vectors (D=10,000+).

## 3. Streaming Memory
**Trait**: `ShardedMemory` (`memory/src/streaming.rs`)
**Target**: GPU Direct Storage.
**Usage**:
- Page large context layers directly to VRAM.

## 4. OCR & Vision
**Adapter**: `ImageAdapter` (`ingestion/src/adapters_extended.rs`)
**Status**: Currently returns placeholder.
**Upgrade**: Connect to Tesseract/PaddleOCR via GPU.
