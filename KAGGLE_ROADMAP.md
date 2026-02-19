# Kaggle / GPU Phase Roadmap

This document serves as the handoff for Phase 2 implementation, which requires GPU acceleration.

## 1. Perception Upgrades (`perception` crate)
- **Implicit Neural Representations (INR):**
  - Implement `SIREN` or `WIRE` networks for resolution-independent image encoding.
  - Map INR weights to HyperVectors via `Vector Function Architecture (VFA)`.
- **Audio VSA:**
  - Implement Complex-Valued VSA (FHRR) for frequency-domain audio binding.
  - Bind audio symbols to temporal sequences.

## 2. Advanced Reasoning (`cognition` crate)
- **Resonator Networks:**
  - Replace BFS inference with Resonator Networks for $O(1)$ factorization of composite structures.
  - Requires parallel matrix multiplication (Tensor Cores).
- **Planetary Scale:**
  - Implement memory-mapped graph traversal for datasets larger than RAM.

## 3. Self-Improvement (`learning` crate)
- **Reflection Loop:**
  - Implement `Action-Critic` logic (symbolic, not RLHF) to score explanation quality.
  - Automatically prune weak connections based on feedback loops.

## 4. Hardware Optimization (`hte` crate)
- **CUDA / Metal / Vulkan:**
  - Integrate `wgpu` or `candle` for tensor operations.
  - Detect GPU VRAM and adjust batch sizes accordingly.

## 5. Mobile Deployment
- **Binary Sharding:**
  - Split `.omf` artifacts into core vs. modality shards for downloading on demand.
  - Implement `mmap` loading for Android/iOS.
