use core_vsa::{HyperVector, DIMENSION};
use std::collections::VecDeque;
use log::{info, debug};

/// A Neuro-Symbolic Sequence Resonator.
/// Maintains a sliding window of recent history and a deep folded context.
///
/// Mathematical Model:
/// Short-term: Explicit buffer of last N items.
/// Long-term: H_t = P(H_{t-1}) + V_t (Rolling Permutation + Bundling)
/// This provides O(1) memory growth for O(T) sequence length.
pub struct SequenceResonator {
    /// Deep folded context (The "Gist" or "Theme").
    pub context_vector: HyperVector,
    /// Explicit sliding window for immediate precision (e.g., last 10 tokens).
    pub window: VecDeque<HyperVector>,
    /// Maximum window size.
    pub window_size: usize,
    /// Current time step.
    pub step: u64,
}

impl SequenceResonator {
    pub fn new(window_size: usize) -> Self {
        Self {
            context_vector: HyperVector::deterministic(0), // Init with zero-like or random
            window: VecDeque::with_capacity(window_size),
            window_size,
            step: 0,
        }
    }

    /// Adds a new token/vector to the sequence.
    /// 1. Updates the sliding window.
    /// 2. Updates the deep folded context via Rolling Permutation.
    pub fn add(&mut self, vector: &HyperVector) {
        // 1. Update Window
        if self.window.len() >= self.window_size {
            self.window.pop_front();
        }
        self.window.push_back(vector.clone());

        // 2. Update Deep Context
        // H_t = P(H_{t-1}) + V_t
        // Permute the existing context (move history "back" in time)
        let permuted_context = self.context_vector.permute(1);

        // Bundle with the new vector.
        // Note: Standard majority bundling with 1 vs 1 inputs yields 50/50 mix.
        // This effectively creates an exponential decay for older items.
        // H_t = 0.5 * P(H_{t-1}) + 0.5 * V_t
        // H_{t-1} was 0.5 * P(H_{t-2}) + 0.5 * V_{t-1}
        // So V_{t-1} is now 0.25, V_{t-2} is 0.125, etc.
        self.context_vector = permuted_context.bundle(vector);

        self.step += 1;

        if self.step % 1000 == 0 {
            debug!("SequenceResonator Step {}: Context Energy preserved.", self.step);
        }
    }

    /// Reconstructs the item at `steps_back` from the current moment.
    /// 0 = current item (last added).
    /// 1 = previous item.
    pub fn reconstruct(&self, steps_back: usize) -> HyperVector {
        // 1. Try Window first (Perfect recall)
        if steps_back < self.window.len() {
            // window is [oldest, ..., newest]
            // steps_back 0 -> index len-1
            // steps_back 1 -> index len-2
            let index = self.window.len() - 1 - steps_back;
            return self.window[index].clone();
        }

        // 2. Try Deep Context (Approximate recall)
        // H_t = V_t + P(V_{t-1}) + P^2(V_{t-2}) ...
        // To get V_{t-k}, we need to apply inverse permutation P^{-k} to H_t.
        // P^{-k}(H_t) = P^{-k}(... + P^k(V_{t-k}) + ...)
        //             = V_{t-k} + Noise

        // Inverse permutation is rotation right, or rotate left by (DIM - k).
        let total_bits = DIMENSION; // 10,000 normally.
        // permute() in core-vsa does bit-level rotation?
        // Let's check core-vsa implementation.
        // It rotates bits (conceptually).
        // If permute(1) is rotate_left(1), then inverse is rotate_right(1) or rotate_left(Total - 1).

        // Note: core-vsa::DIMENSION is bits.
        // permute(k) shifts left by k bits.

        // We need to handle the case where steps_back > total_bits (cyclic).
        // But usually permutations are modulo Dimension.

        let inverse_shift = (total_bits * 64) - (steps_back % (total_bits * 64)); // Wait, DIMENSION is bits?
        // core-vsa: DIMENSION = 10_000 bits.
        // permute() logic: "bits.rotate_left(shift)".
        // So yes, inverse is rotate_left(DIMENSION - k).

        let shift = if steps_back == 0 { 0 } else { DIMENSION - (steps_back % DIMENSION) };

        self.context_vector.permute(shift)
    }

    /// Folds a sequence of vectors into a single summary vector directly.
    pub fn fold_sequence(vectors: &[HyperVector]) -> HyperVector {
        let mut context = HyperVector::deterministic(0);
        for v in vectors {
            context = context.permute(1).bundle(v);
        }
        context
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sequence_window_recall() {
        let mut resonator = SequenceResonator::new(5);
        let v1 = HyperVector::random();
        let v2 = HyperVector::random();
        let v3 = HyperVector::random();

        resonator.add(&v1);
        resonator.add(&v2);
        resonator.add(&v3);

        // Reconstruct 0 -> v3
        let r0 = resonator.reconstruct(0);
        assert!(r0.similarity(&v3) > 0.99);

        // Reconstruct 1 -> v2
        let r1 = resonator.reconstruct(1);
        assert!(r1.similarity(&v2) > 0.99);

        // Reconstruct 2 -> v1
        let r2 = resonator.reconstruct(2);
        assert!(r2.similarity(&v1) > 0.99);
    }

    #[test]
    fn test_deep_context_decay() {
        // Test that recent items are retrievable from the folded context even if outside window
        // With majority bundle 50/50, capacity drops fast: 100% -> 50% -> 25% -> 12.5% -> 6.25% (Noise floor ~0.5 + epsilon)
        // Binary VSA capacity for superposition is low without "cleanup memory" (associative memory).
        // However, the *orientation* should still be closer to the target than random.

        let mut resonator = SequenceResonator::new(0); // No window!
        let v1 = HyperVector::random();
        let v2 = HyperVector::random();

        resonator.add(&v1); // Context = v1
        resonator.add(&v2); // Context = bundle(permute(v1), v2)

        // Recover v2 (step 0)
        let r0 = resonator.reconstruct(0);
        // Should be very close to v2 (50% bits from v2, 50% from p(v1))
        // Similarity should be ~0.5 (Hamming distance) -> 0.0 Cosine?
        // Wait, similarity() range is 1.0 to -1.0.
        // If 50% bits match, Hamming = Dim/2. Sim = 1 - 2*(0.5) = 0.0.
        // This implies "Orthogonal".
        // Actually, if we bundle A and B, the result C has sim(C, A) ~ 0.707 (in Euclidean) or bit-wise...
        // Let's check core-vsa bundle logic.
        // (a & !diff) | (random & diff)
        // If diff is 1 (50% of time), we take random (50% of that 50% match A).
        // So we match A in 50% (where bits equal) + 25% (where bits differ but random matches) = 75%.
        // Hamming distance = 25%.
        // Similarity = 1 - 2 * 0.25 = 0.5.
        // So we expect > 0.4.

        let sim0 = r0.similarity(&v2);
        assert!(sim0 > 0.4, "Similarity to last item should be > 0.4, got {}", sim0);

        // Recover v1 (step 1)
        let r1 = resonator.reconstruct(1);
        // r1 = permute(-1, context) = permute(-1, bundle(permute(v1), v2))
        //    = bundle(v1, permute(-1, v2))
        // So r1 should be similar to v1.
        let sim1 = r1.similarity(&v1);
        assert!(sim1 > 0.4, "Similarity to penultimate item should be > 0.4, got {}", sim1);
    }

    #[test]
    fn test_long_context_stability() {
        let mut resonator = SequenceResonator::new(10);
        // Add 1000 items
        for _ in 0..1000 {
            resonator.add(&HyperVector::random());
        }
        // Check memory usage (conceptually, by ensuring no panic or slowdown)
        assert_eq!(resonator.step, 1000);
    }
}
