# GPU Fabrication Handoff

Omni Forge Phase 1 is CPU-only. Phase 2 enables GPU acceleration via defined interfaces.

## Interfaces (`gpu_interfaces.rs`)
-   **InrInterface:** Implicit Neural Representations for images/video.
-   **ResonatorInterface:** High-speed factorization of VSA vectors.
-   **AudioVsaInterface:** Complex-valued vector binding.
-   **TensorBackend:** Abstraction for `wgpu` / `candle`.

## Data Pipeline
1.  **CPU Phase:** Ingests raw data, tokenizes, structures.
2.  **GPU Phase:** Trains INRs, optimizes resonators, performs massive batch consolidation.
3.  **Handoff:** GPU-trained weights are mapped to VSA HyperVectors and stored in the MindPack.

## Constraints
-   No GPU code in the core engine.
-   Runtime must fallback to CPU (slower) if GPU is unavailable.
