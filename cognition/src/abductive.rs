use core_vsa::{HyperVector, DIMENSION};
use log::{info, debug};

pub struct AbductiveReasoner {
    pub min_confidence: f32,
}

impl AbductiveReasoner {
    pub fn new(min_confidence: f32) -> Self {
        Self { min_confidence }
    }

    pub fn induce_hypothesis(&self, consequent: &HyperVector, rule_memory: &HyperVector) -> HyperVector {
        let hypothesis = rule_memory.bind(consequent);
        debug!("Inducing hypothesis from consequent.");
        hypothesis
    }

    pub fn mine_pattern(&self, observations: &[HyperVector]) -> Option<HyperVector> {
        if observations.is_empty() {
            return None;
        }

        let mut centroid = observations[0].clone();
        for i in 1..observations.len() {
            centroid = centroid.bundle(&observations[i]);
        }

        Some(centroid)
    }

    pub fn learn_rules(&self, sequence: &[(HyperVector, HyperVector)]) -> HyperVector {
        if sequence.is_empty() {
            return HyperVector::random();
        }

        let mut rule_memory = sequence[0].0.bind(&sequence[0].1);

        for i in 1..sequence.len() {
            let pair_vector = sequence[i].0.bind(&sequence[i].1);
            rule_memory = rule_memory.bundle(&pair_vector);
        }

        rule_memory
    }

    pub fn validate(&self, hypothesis: &HyperVector, constraints: &[(HyperVector, bool)]) -> bool {
        for (constraint, required) in constraints {
            let sim = hypothesis.similarity(constraint);
            if *required {
                if sim < self.min_confidence {
                    debug!("Validation Failed: Required similarity {}, got {}", self.min_confidence, sim);
                    return false;
                }
            } else {
                if sim.abs() > self.min_confidence {
                    debug!("Validation Failed: Forbidden similarity > {}, got {}", self.min_confidence, sim);
                    return false;
                }
            }
        }
        true
    }

    pub fn detect_vsa_contradiction(&self, a: &HyperVector, b: &HyperVector) -> bool {
        // In VSA, contradiction can be seen as very high similarity to an inverse or very low similarity where it should be high.
        // Or if we define contradiction as A and Not A.
        // If B is expected to be A but is orthogonal, it's a contradiction in some contexts.
        // For Phase 1, let's say if similarity < -0.5, it's a contradiction (Opposite in bipolar, but here we use BSC 0/1).
        // In BSC, similarity ranges from -1 to 1?
        // similarity = 1 - 2 * (hamming / total)
        // hamming = 0 => sim = 1
        // hamming = total/2 => sim = 0 (Orthogonal)
        // hamming = total => sim = -1 (Opposite/Inverse)

        let sim = a.similarity(b);
        sim < -0.7 // High degree of inversion
    }

    pub fn score_similarity(&self, hypothesis: &HyperVector, prototype: &HyperVector) -> f32 {
        hypothesis.similarity(prototype)
    }
}
