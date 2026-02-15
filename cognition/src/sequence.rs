use core_vsa::{HyperVector, DIMENSION};
use std::collections::VecDeque;
use log::debug;

pub struct SequenceResonator {
    pub context_vector: HyperVector,
    pub window: VecDeque<HyperVector>,
    pub window_size: usize,
    pub step: u64,
}

impl SequenceResonator {
    pub fn new(window_size: usize) -> Self {
        Self {
            context_vector: HyperVector::deterministic(0),
            window: VecDeque::with_capacity(window_size),
            window_size,
            step: 0,
        }
    }

    pub fn add(&mut self, vector: &HyperVector) {
        if self.window.len() >= self.window_size && self.window_size > 0 {
            self.window.pop_front();
        }
        if self.window_size > 0 {
            self.window.push_back(vector.clone());
        }

        if self.step == 0 {
            self.context_vector = vector.clone();
        } else {
            let permuted_context = self.context_vector.permute(1);
            self.context_vector = permuted_context.bundle(vector);
        }

        self.step += 1;

        if self.step % 1000 == 0 {
            debug!("SequenceResonator Step {}: Context Energy preserved.", self.step);
        }
    }

    pub fn reconstruct(&self, steps_back: usize) -> HyperVector {
        if steps_back < self.window.len() {
            let index = self.window.len() - 1 - steps_back;
            return self.window[index].clone();
        }

        let shift = if steps_back == 0 { 0 } else { DIMENSION - (steps_back % DIMENSION) };
        self.context_vector.permute(shift)
    }

    pub fn fold_sequence(vectors: &[HyperVector]) -> HyperVector {
        if vectors.is_empty() {
            return HyperVector::deterministic(0);
        }
        let mut context = vectors[0].clone();
        for v in &vectors[1..] {
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

        let r0 = resonator.reconstruct(0);
        assert!(r0.similarity(&v3) > 0.99);

        let r1 = resonator.reconstruct(1);
        assert!(r1.similarity(&v2) > 0.99);

        let r2 = resonator.reconstruct(2);
        assert!(r2.similarity(&v1) > 0.99);
    }

    #[test]
    fn test_deep_context_decay() {
        let mut resonator = SequenceResonator::new(0);
        let v1 = HyperVector::random();
        let v2 = HyperVector::random();

        resonator.add(&v1);
        resonator.add(&v2);

        let r0 = resonator.reconstruct(0);
        let sim0 = r0.similarity(&v2);
        assert!(sim0 > 0.4, "Similarity to last item should be > 0.4, got {}", sim0);

        let r1 = resonator.reconstruct(1);
        let sim1 = r1.similarity(&v1);
        assert!(sim1 > 0.4, "Similarity to penultimate item should be > 0.4, got {}", sim1);
    }

    #[test]
    fn test_long_context_stability() {
        let mut resonator = SequenceResonator::new(10);
        for _ in 0..1000 {
            resonator.add(&HyperVector::random());
        }
        assert_eq!(resonator.step, 1000);
    }
}
