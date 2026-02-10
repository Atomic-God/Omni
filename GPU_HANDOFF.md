# GPU Handoff Plan (Future Phase)

Omni Forge v9.0 is CPU-optimized (AVX/Neon).
To ascend to Phase 2 (Massive Parallelism), the following hooks are prepared:

1.  **Tensor Backend**:
    - `perception/src/gpu_interfaces.rs` defines `TensorBackend` trait.
    - Implement this using `wgpu` or `cuda`.

2.  **Hypervector Operations**:
    - `core-vsa` operations (xor, permute) are O(D).
    - GPU kernel can run these in parallel for millions of concepts.

3.  **Visual Synthesis**:
    - `perception` has `InrInterface`.
    - Implement HC-INR (Hyper-Coordinate Implicit Neural Representations) on GPU.

4.  **Streaming**:
    - `memory/src/streaming.rs` defines `ShardedMemory`.
    - Implement GPU Direct Storage loading.
