use engine::OmniMind;
use log::{info, warn};
use std::io::{self, Write};
use std::sync::{Arc, Mutex};

fn main() {
    env_logger::init();
    info!("Omni Forge CLI v5.1 Starting...");

    // Hardware check
    println!("Initializing Hardware Truth Engine...");
    let profile = hte::detect();
    println!("HTE Profile Detected:\n{:#?}", profile);

    println!("Initializing OmniMind Fabricator...");
    let mind = OmniMind::new();
    let mind_arc = Arc::new(Mutex::new(mind));

    // Try load mind.omf
    {
        let mut m = mind_arc.lock().unwrap();
        if m.load("mind.omf").is_err() {
            warn!("No sovereign mind found. Starting fresh.");
            println!("No mind.omf found. Starting fresh fabrication.");
        } else {
            println!("Sovereign Mind Loaded.");
        }
    }

    println!("\nOmni Forge Interactive Shell.");
    println!("Commands:");
    println!("  train <text>   - Learn from text");
    println!("  ask <query>    - Query the mind");
    println!("  save-mind      - Save to mind.omf");
    println!("  load-mind      - Load from mind.omf");
    println!("  exit           - Quit");

    loop {
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

        let mut parts = input.splitn(2, ' ');
        let command = parts.next().unwrap_or("");
        let args = parts.next().unwrap_or("");

        let mut mind = mind_arc.lock().unwrap();

        match command {
            "train" => {
                if args.is_empty() {
                    println!("Usage: train <text>");
                } else {
                    mind.learn(args);
                    println!("Learned.");
                }
            }
            "ask" => {
                if args.is_empty() {
                    println!("Usage: ask <query>");
                } else {
                    let response = mind.ask(args);
                    println!("Mind: {}", response);
                }
            }
            "save-mind" => match mind.save("mind.omf") {
                Ok(_) => println!("Mind saved to mind.omf"),
                Err(e) => println!("Error saving: {}", e),
            },
            "load-mind" => match mind.load("mind.omf") {
                Ok(_) => println!("Mind loaded."),
                Err(e) => println!("Error loading: {}", e),
            },
            _ => {
                println!("Unknown command. Try: train, ask, save-mind, load-mind, exit");
            }
        }
    }
}
