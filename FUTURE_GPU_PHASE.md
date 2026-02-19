# The Future of Phase 2: GPU Ascension
**Status: Deferred**

This document outlines the features and capabilities explicitly deferred to Phase 2 (GPU), as per the initial project scope.

## 1. High-Performance Perception (INR)
*   **HC-INR:** Hyper-Coordinate Implicit Neural Representations for high-fidelity visual synthesis (O(D)).
*   **Audio VSA:** Frequency-domain Vector Symbolic Architectures (FHRR) for binaural localization and speech synthesis.
*   **Current State:** CPU stubs exist in `perception/src/vision_stub.rs` and `modality.rs`.
*   **Phase 2:** Implement CUDA/WGPU backends for these modules.

## 2. Massive Parallelism
*   **HTE Extension:** Extend `HardwareProfile` to detect NVIDIA Tensor Cores (currently detects CPU AVX/AMX).
*   **Mind Fabricator:** Offload heavy `CognitionCore` operations (bundling, binding) to GPU kernels (thousands of threads).
*   **Real-Time Training:** Enable continuous learning at 60Hz+ on consumer hardware.

## 3. Advanced Cognition
*   **Hrrformer:** Implement the O(T) linear complexity self-attention mechanism on GPU.
*   **Video Reasoning:** Temporal binding across frames using GPU memory bandwidth.
*   **Generative Feedback:** Top-down signal propagation (Inverse Graphics) for imagination.

## 4. Hardware Requirement Handoff
*   **Target:** NVIDIA RTX 3060 / Apple M1 Pro (or better).
*   **Dependencies:** `wgpu`, `cuda-runtime`, `tch-rs` (PyTorch bindings).
*   **Architecture:** Hybrid CPU-GPU pipeline. CPU manages logic (VSA), GPU handles dense tensors (INR).

## 5. Security Implications
*   GPU access introduces new attack vectors (side-channel).
*   The `PermissionBoundary` (engine) must extend to GPU memory allocation.
*   Zero-Knowledge proofs for computation integrity might be needed.

---
*The foundation is laid. The factory awaits power.*
