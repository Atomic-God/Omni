use core_vsa::hypervector::{Hypervector, VsaOps};

#[test]
fn test_binding_reversibility() {
    let dim = 1000;
    let a = Hypervector::random(dim);
    let b = Hypervector::random(dim);

    // XOR Binding is its own inverse: (A * B) * B = A
    let bound = a.bind(&b);
    let unbound = bound.bind(&b);

    assert_eq!(a, unbound);
}

#[test]
fn test_bundling_similarity() {
    let dim = 1000;
    let a = Hypervector::random(dim);
    let b = Hypervector::random(dim);

    let bundle = a.bundle(&b);

    // Bundle should be somewhat similar to constituents (distance < 0.5)
    // Note: With bitwise OR, distance might be different than superposition
    // For random vectors (0.5 density), ORing them results in 0.75 density.
    // Distance from A (0.5) to A|B (0.75) involves the bits where B is 1 and A is 0 (0.25).
    // So distance should be around 0.25.

    let dist_a = bundle.distance(&a);
    let dist_b = bundle.distance(&b);

    assert!(dist_a < 0.4);
    assert!(dist_b < 0.4);
}
