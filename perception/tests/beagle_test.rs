use perception::Beagle;

#[test]
fn test_beagle_context_learning() {
    let mut beagle = Beagle::new();

    // Train: Cat and Dog share "The" and "chases/bites" somewhat, but let's make it distinct first.
    // "The cat meows"
    // "The dog barks"
    // "The car drives"

    // Contexts:
    // Cat: {The, meows}
    // Dog: {The, barks}
    // Car: {The, drives}

    // "The" is common to all.
    // "meows", "barks", "drives" are random distinct vectors.
    // Cat and Dog should have similarity based on "The".

    // Let's add more overlap.
    // "The cat runs"
    // "The dog runs"
    // "The car drives"

    beagle.learn_sentence("The cat runs");
    beagle.learn_sentence("The dog runs");
    beagle.learn_sentence("The car drives");

    let sim_cat_dog = beagle.similarity("cat", "dog").expect("Words should be learned");
    let sim_cat_car = beagle.similarity("cat", "car").expect("Words should be learned");

    println!("Sim Cat-Dog: {}", sim_cat_dog);
    println!("Sim Cat-Car: {}", sim_cat_car);

    // Cat context: {The, runs}
    // Dog context: {The, runs} -> Identical context -> High similarity
    // Car context: {The, drives} -> Shared {The} only -> Lower similarity

    assert!(sim_cat_dog > sim_cat_car);
}
