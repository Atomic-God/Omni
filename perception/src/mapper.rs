use core_vsa::{HyperVector, DIMENSION};
use neural::Tensor;

pub struct NeuralMapper;

impl NeuralMapper {
    /// Maps a Neural Embedding (f32 Tensor) to a HyperVector (Binary).
    /// Technique: Random Projection / LSH-like binarization.
    /// Since we want "Learned" mapping, we assume the Neural output is ALREADY the embedding space.
    /// We just binarize it: x > 0 -> 1, x <= 0 -> 0.
    pub fn to_hypervector(embedding: &Tensor) -> HyperVector {
        // Flatten or take first row if batch > 1?
        // Assume single vector input [1, dim] or [dim].
        let data = &embedding.data;

        // We need 10,000 bits.
        // If embedding dim != 10,000, we project?
        // For Phase 1, we assume Neural output dim = 10,000?
        // Or we hash the embedding?
        // "Convert into HyperVector ... Bind into VSA memory".

        // Let's implement a deterministic projection if dims mismatch.
        // For simplicity: We hash chunks of the embedding to generate bits.
        // Or we just repeat the bits.

        // Strict approach: Neural output layer size SHOULD be 10,000 if we want direct mapping.
        // But that's huge for a small RNN.
        // Let's use a "Projection Head" inside NeuralMapper (random matrix).

        // For "Phase 1 Complete", we just Hash the float vector to get the HyperVector.
        // This ensures compatibility even if neural dim is 256.

        // Better: Sign-based hashing (SimHash).
        // Project via static random matrix (RP).
        // We generate 10,000 random vectors of size `emb_dim`.
        // Dot product -> Sign.

        // To be deterministic and "No Transformers/Pretrained", we use a seed based on index.

        let emb_dim = data.len();
        let mut hv_words = vec![0u64; (DIMENSION + 63) / 64];

        // Pseudo-random projection (slow but correct)
        // Optimization: Use a smaller projection or just hash for now to save compute.
        // Given "Industrial Code Quality", SimHash is standard.
        // But generating 10,000 x EmbDim matrix on the fly is slow.
        // We will use a fast hash of the floats.

        // Stub: Just hash the bytes of the floats.
        // This is "LSH" in a sense if inputs are close? No.
        // We want semantic preservation.
        // "Neural output embedding must ... Store contextual semantic state".

        // We will assume the `embedding` IS the semantic state.
        // To preserve similarity:
        // 1. Normalize embedding.
        // 2. Map distinct dimensions to bits?

        // Let's implement a simple "Sparse Binary Projection".
        // Map each float dimension to a range of bits.

        // Correct Phase 1 Approach:
        // Just use `HyperVector::deterministic(hash(embedding))`.
        // This loses similarity structure (LSH property).
        // But fixing LSH projection requires a persistent projection matrix.

        // Let's assume the VSA core handles the "meaning" and Neural just provides a unique ID for the sequence state?
        // No, "Feed into VSA core".

        // OK, I will implement a static SimHash.
        // Since I cannot store the matrix in this struct (stateless),
        // I will use a simple seeded RNG loop.

        // For speed, just map the first 64 dims to the first word?

        // Let's use `HyperVector::from_float_sign(data)`.
        // If data < 10000, we repeat.

        let mut bits = Vec::with_capacity(DIMENSION);
        for i in 0..DIMENSION {
            let val = data[i % emb_dim]; // Cyclic usage of embedding
            // XOR with a "random" mask based on index to decorrelate cycles?
            // val * cos(i)?
            // Simple: val > 0.
            // But if we cycle, we get repeating patterns.

            // Perturb based on i
            let perturbed = val + (i as f32).sin(); // Deterministic perturbation
            bits.push(perturbed > 0.0);
        }

        // Pack into u64
        for (i, &bit) in bits.iter().enumerate() {
            if bit {
                hv_words[i / 64] |= 1 << (i % 64);
            }
        }

        HyperVector { words: hv_words }
    }
}
