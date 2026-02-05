use engine::OmniMind;
use hte;

fn main() {
    println!("Omni Forge Mind Factory Starting...");

    // Hardware check
    println!("Initializing Hardware Truth Engine...");
    let profile = hte::detect();
    println!("HTE Profile Detected:\n{:#?}", profile);

    println!("Initializing OmniMind...");
    let mut mind = OmniMind::new();

    // Try to load existing memory
    if let Err(_) = mind.load("memory.json") {
        println!("No existing memory found. Creating new mind.");

        // Train on initial corpus
        let corpus = [
            "the dog is an animal",
            "the animal is living",
            "the living thing grows",
            "dog eats food"
        ];

        println!("Training Mind on {} sentences...", corpus.len());
        for sentence in corpus {
            mind.learn(sentence);
        }
        println!("Mind trained");
    } else {
        println!("Memory Loaded from 'memory.json'.");
    }

    println!("\nOmni Forge Interactive Mode. Type 'exit' to quit.");
    println!("Ask: 'Does dog grow?', 'Is dog animal?', 'What does dog eat?'");

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

        let response = mind.ask(input);
        println!("Mind: {}", response);
    }

    // Save on exit
    if let Err(e) = mind.save("memory.json") {
        eprintln!("Failed to save memory: {}", e);
    } else {
        println!("Memory saved to 'memory.json'.");
    }
}
