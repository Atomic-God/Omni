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
                "the animal is living",
                "the living thing grows"
            ];

            println!("Training Mind on {} sentences...", corpus.len());
            for sentence in corpus {
                m.learn_sentence(sentence);
            }
            println!("Mind trained");
            m
        }
    };

    println!("\nOmni Forge Interactive Mode. Type 'exit' to quit.");
    println!("Ask: 'Does dog grow?', 'Is dog animal?'");

    loop {
        use std::io::{self, Write};
        print!("> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            break;
        }

        let input = input.trim();
        if input == "exit" {
            break;
        }

        let response = mind.answer(input);
        println!("Mind: {}", response);
    }

    // 6. Persist Memory
    if let Err(e) = mind.save("memory.json") {
        eprintln!("Failed to save memory: {}", e);
    } else {
        println!("Memory saved to 'memory.json'.");
    }
}
