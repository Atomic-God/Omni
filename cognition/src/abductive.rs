use core_vsa::{HyperVector, DIMENSION};
use log::{info, debug};

/// Engine for Abductive Reasoning and Pattern Mining using VSA operations.
///
/// Principles:
/// 1. Abduction: Given Rule (A -> B) and Result (B), infer A.
///    In VSA: M = A * B. Query M * B = A.
/// 2. Pattern Mining: Extract commonalities from a set of observations.
///    In VSA: C = Bundle(O1, O2, ...).
/// 3. Constraint Validation: Ensure hypothesis satisfies known invariants.
pub struct AbductiveReasoner {
    pub min_confidence: f32,
}

impl AbductiveReasoner {
    pub fn new(min_confidence: f32) -> Self {
        Self { min_confidence }
    }

    /// Induces a hypothesis (Antecedent) given a Consequent and a Rule Memory.
    ///
    /// # Arguments
    /// * `consequent` - The observed result (B).
    /// * `rule_memory` - The superposition of known rules (Sum(A_i * B_i)).
    ///
    /// # Returns
    /// * `HyperVector` - The inferred antecedent (A).
    pub fn induce_hypothesis(&self, consequent: &HyperVector, rule_memory: &HyperVector) -> HyperVector {
        // Operation: Unbind (XOR is its own inverse)
        // H = M * C
        // H = (Sum(A * B)) * B = A + Noise
        let hypothesis = rule_memory.bind(consequent);
        debug!("Inducing hypothesis from consequent.");
        hypothesis
    }

    /// Mines a common pattern (Centroid) from a set of observations.
    /// This extracts the "invariant" structure shared by the inputs.
    pub fn mine_pattern(&self, observations: &[HyperVector]) -> Option<HyperVector> {
        if observations.is_empty() {
            return None;
        }

        // Bundle all observations
        let mut centroid = observations[0].clone();
        for i in 1..observations.len() {
            centroid = centroid.bundle(&observations[i]);
        }

        // In a real VSA, we might want to normalize or threshold here.
        // But binary bundle is self-normalizing (majority).
        Some(centroid)
    }

    /// Generates a set of potential rules from a sequence of events.
    /// If we see A then B multiple times, we induce Rule R = A * B.
    ///
    /// # Arguments
    /// * `sequence` - List of (Event, Consequent) pairs.
    pub fn learn_rules(&self, sequence: &[(HyperVector, HyperVector)]) -> HyperVector {
        let mut rule_memory = HyperVector::deterministic(0); // Zero-like? Or random?
        // Ideally "Zero" vector for bundling, but VSA usually starts with random.
        // If we start with random, it adds noise.
        // We need a "Neutral" element for Bundle? Binary VSA doesn't have one easily (except 0, but 0 is not valid usually).
        // We'll assume the first pair establishes the memory.

        if sequence.is_empty() {
            return HyperVector::random();
        }

        rule_memory = sequence[0].0.bind(&sequence[0].1);

        for i in 1..sequence.len() {
            let pair_vector = sequence[i].0.bind(&sequence[i].1);
            rule_memory = rule_memory.bundle(&pair_vector);
        }

        rule_memory
    }

    /// Validates if a hypothesis satisfies a set of constraints.
    ///
    /// # Arguments
    /// * `hypothesis` - The vector to check.
    /// * `constraints` - List of (ConstraintVector, IsPositive).
    ///                   If IsPositive is true, Sim(H, C) must be > threshold.
    ///                   If IsPositive is false, Sim(H, C) must be < threshold.
    pub fn validate(&self, hypothesis: &HyperVector, constraints: &[(HyperVector, bool)]) -> bool {
        for (constraint, required) in constraints {
            let sim = hypothesis.similarity(constraint);
            if *required {
                // Must be similar
                if sim < self.min_confidence {
                    debug!("Validation Failed: Required similarity {}, got {}", self.min_confidence, sim);
                    return false;
                }
            } else {
                // Must be dissimilar (Orthogonal)
                // "Dissimilar" in VSA means Sim ~ 0.
                // If Sim is high, it violates the negative constraint.
                if sim.abs() > self.min_confidence {
                    debug!("Validation Failed: Forbidden similarity > {}, got {}", self.min_confidence, sim);
                    return false;
                }
            }
        }
        true
    }

    /// Scores the structural similarity of a hypothesis against a prototype.
    pub fn score_similarity(&self, hypothesis: &HyperVector, prototype: &HyperVector) -> f32 {
        hypothesis.similarity(prototype)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_abduction_simple() {
        let reasoner = AbductiveReasoner::new(0.4);

        let a = HyperVector::random();
        let b = HyperVector::random();

        // Rule: A -> B (represented as A * B)
        let rule = a.bind(&b);

        // Infer A from B
        let hypothesis = reasoner.induce_hypothesis(&b, &rule);

        // H should be A
        assert!(hypothesis.similarity(&a) > 0.99);
    }

    #[test]
    fn test_pattern_mining() {
        let reasoner = AbductiveReasoner::new(0.4);

        let common = HyperVector::random();
        let noise1 = HyperVector::random();
        let noise2 = HyperVector::random();

        // Obs1 = Common * Noise1
        let obs1 = common.bind(&noise1); // Wait, pattern mining usually finds Common in a Bundle (Superposition).
        // If we bind, it's "Structure". If we bundle, it's "Class".
        // Let's assume the observations are variations of a class: O = C + noise.
        // Bundle(C+N1, C+N2) -> C.

        // But if we use Bind (XOR), we change the vector completely.
        // "Pattern Mining" in VSA usually means finding the common component in a superposition.
        // Let's try: Obs1 and Obs2 are just "similar" vectors.
        // Create 3 vectors that are "close" to a prototype.
        // Since we can't easily "perturb" a boolean vector to be "close" without specific logic,
        // let's just bundle Common with distinct noise vectors?
        // Bundle(C, N1) -> 50% C.

        let obs1 = common.bundle(&noise1);
        let obs2 = common.bundle(&noise2);
        let obs3 = common.bundle(&HyperVector::random());

        let mined = reasoner.mine_pattern(&[obs1, obs2, obs3]).unwrap();

        // Mined should be close to Common
        let sim = mined.similarity(&common);
        assert!(sim > 0.4, "Mined pattern should resemble common prototype, got {}", sim);
    }

    #[test]
    fn test_rule_learning_and_abduction() {
        let reasoner = AbductiveReasoner::new(0.2); // Lower threshold for noisy bundle

        let a1 = HyperVector::random();
        let b1 = HyperVector::random();

        let a2 = HyperVector::random(); // Different cause
        let b2 = HyperVector::random(); // Different effect

        // Learn: A1->B1, A2->B2
        let rules = reasoner.learn_rules(&[(a1.clone(), b1.clone()), (a2.clone(), b2.clone())]);

        // Abduce A1 from B1
        let h1 = reasoner.induce_hypothesis(&b1, &rules);

        // H1 should be close to A1.
        // Rules = (A1*B1) + (A2*B2)
        // H1 = Rules * B1 = A1 + (A2*B2*B1)
        // The second term is noise.
        assert!(h1.similarity(&a1) > 0.35, "Abduction failed in superposition.");

        // Abduce A2 from B2
        let h2 = reasoner.induce_hypothesis(&b2, &rules);
        assert!(h2.similarity(&a2) > 0.35);
    }
}
