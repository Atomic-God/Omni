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
    let mind = match Beagle::load("memory.json") {
        Ok(m) => {
            println!("Memory Loaded from 'memory.json'.");
            m
        },
        Err(_) => {
            println!("No existing memory found. Creating new mind.");
            let mut m = Beagle::new();

            // 3. Train on example sentences
            let corpus = [
                "the dog is an animal",
                "the animal breathes air"
            ];

            println!("Training Mind on {} sentences...", corpus.len());
            for sentence in corpus {
                m.learn_sentence(sentence);
            }
            println!("Mind trained");
            m
        }
    };

    // 4. Relation Inference
    let a = "dog";
    let b = "animal";
    let c = "breathes";

    // Debug similarities
    if let Some(s) = mind.similarity(a, b) { println!("Sim({}, {}): {}", a, b, s); }
    if let Some(s) = mind.similarity(b, c) { println!("Sim({}, {}): {}", b, c, s); }

    let inferred = mind.infer_relation(a, b, c);
    println!("\nInfer Relation: {} -> {} -> {}? {}", a, b, c, inferred);

    // 6. Persist Memory
    if let Err(e) = mind.save("memory.json") {
        eprintln!("Failed to save memory: {}", e);
    } else {
        println!("Memory saved to 'memory.json'.");
    }
}
