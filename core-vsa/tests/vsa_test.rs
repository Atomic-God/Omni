use core_vsa::HyperVector;

#[test]
fn test_binding_reversibility() {
    let a = HyperVector::random();
    let b = HyperVector::random();

    // XOR (Mult) Binding is its own inverse: (A * B) * B = A
    let bound = a.bind(&b);
    let recovered = bound.bind(&b);

    // Similarity should be 1.0 (or very close due to float precision)
    assert!(a.similarity(&recovered) > 0.99);
}

#[test]
fn test_bundling_similarity() {
    let a = HyperVector::random();
    let b = HyperVector::random();

    let bundle = a.bundle(&b);

    // Bundle should be similar to constituents
    // For bipolar vectors with random tie-breaking, similarity is ~0.5

    let sim_a = bundle.similarity(&a);
    let sim_b = bundle.similarity(&b);

    assert!(sim_a > 0.45);
    assert!(sim_b > 0.45);
}
