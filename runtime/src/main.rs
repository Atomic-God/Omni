use runtime::RuntimeEngine;
use hte;
use core_vsa::hypervector::{Hypervector, VsaOps};

fn main() {
    println!("Omni Forge Mind Factory Starting...");
    println!("Initializing Hardware Truth Engine...");

    let profile = hte::detect();
    println!("HTE Profile Detected:\n{:#?}", profile);

    println!("Initializing Cognitive Core...");
    let hv_dim = 1000;
    let concept_a = Hypervector::random(hv_dim);
    let concept_b = Hypervector::random(hv_dim);
    let bound_concept = concept_a.bind(&concept_b);

    println!("VSA Test: Created and Bound Hypervectors (Dim: {})", hv_dim);
    println!("Distance(A, A*B) = {}", concept_a.distance(&bound_concept));

    let _engine = RuntimeEngine;
    println!("Mind Factory Initialized.");
}
