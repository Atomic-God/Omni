use core_vsa::HyperVector;

// ==================================================================================
// GPU / KAGGLE PHASE INTERFACES
// These traits define the contract for future GPU-accelerated modules.
// DO NOT implement logic here. Only define the boundary.
// ==================================================================================

/// Interface for Implicit Neural Representations (INR)
/// Used for resolution-independent image/video encoding.
pub trait InrInterface: Send + Sync {
    /// Encodes an image buffer into a functional representation (HyperVector weights).
    fn encode_image(&self, pixels: &[u8], width: u32, height: u32) -> HyperVector;

    /// Decodes a HyperVector back into an image at a specific resolution.
    fn decode_image(&self, vec: &HyperVector, width: u32, height: u32) -> Vec<u8>;

    /// Trains the INR on a stream of visual data (Online Learning).
    fn train_batch(&mut self, images: &[Vec<u8>]);
}

/// Interface for Resonator Networks
/// Used for high-speed factorization of composite vectors (Search in Superposition).
pub trait ResonatorInterface: Send + Sync {
    /// Factorizes a composite vector P into factors {A, B, C} such that P = A * B * C.
    /// Returns the most likely factors.
    fn factorize(&self, product: &HyperVector, codebooks: &[Vec<HyperVector>]) -> Vec<HyperVector>;

    /// Runs the resonator dynamics for a fixed number of iterations.
    fn iterate(&self, state: &mut [HyperVector]) -> bool; // converged?
}

/// Interface for Audio VSA (Complex-Valued / FHRR)
/// Used for frequency-domain binding.
pub trait AudioVsaInterface: Send + Sync {
    /// Converts PCM audio samples to a Holographic Reduced Representation.
    fn encode_audio(&self, samples: &[f32], sample_rate: u32) -> HyperVector;

    /// Binds audio features to a temporal trajectory.
    fn bind_temporal(&self, sequence: &[HyperVector]) -> HyperVector;
}

/// Interface for GPU Tensor Backend
/// Abstracts `wgpu`, `candle`, or `tch`.
pub trait TensorBackend: Send + Sync {
    fn name(&self) -> &str;
    fn available_vram(&self) -> usize;
    fn matmul(&self, a: &[f32], b: &[f32], dims: (usize, usize, usize)) -> Vec<f32>;
}
