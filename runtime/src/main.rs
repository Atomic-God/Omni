use runtime::RuntimeEngine;
use hte;
use core_vsa::HyperVector;

fn main() {
    println!("Omni Forge Mind Factory Starting...");
    println!("Initializing Hardware Truth Engine...");

    let profile = hte::detect();
    println!("HTE Profile Detected:\n{:#?}", profile);

    println!("Initializing Cognitive Core...");
    // HyperVector dimension is now fixed at 10,000 internally
    let concept_a = HyperVector::random();
    let concept_b = HyperVector::random();
    let bound_concept = concept_a.bind(&concept_b);

    println!("VSA Test: Created and Bound Hypervectors (Dim: 10000)");
    println!("Similarity(A, A*B) = {}", concept_a.similarity(&bound_concept));

    let _engine = RuntimeEngine;
    println!("Mind Factory Initialized.");
}
