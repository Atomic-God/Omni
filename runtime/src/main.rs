use perception::Beagle;
use hte;

fn main() {
    println!("Omni Forge Mind Factory Starting...");

    // 1. Hardware Truth Engine Check
    println!("Initializing Hardware Truth Engine...");
    let profile = hte::detect();
    println!("HTE Profile Detected:\n{:#?}", profile);

    // 2. Initialize Cognitive Core (Beagle Mind)
    println!("Initializing BEAGLE Mind...");
    let mut mind = Beagle::new();

    // 3. Train on example sentences
    let corpus = [
        "The cat runs fast",
        "The dog runs fast",
        "The fish swims deep",
        "The bird flies high"
    ];

    println!("Training Mind on {} sentences...", corpus.len());
    for sentence in corpus {
        mind.learn_sentence(sentence);
    }

    println!("Mind trained");

    // 4. Verification
    if let Some(sim) = mind.similarity("cat", "dog") {
        println!("Similarity(cat, dog) = {:.4}", sim);
    }

    // 5. Semantic Query
    println!("\nQuery: Most similar to 'dog':");
    let results = mind.most_similar("dog");
    for (word, score) in results {
        println!(" - {}: {:.4}", word, score);
    }
}
