use fabricator::facade::OmniForge;
use log::{info, error};
use std::env;
use std::io::{self, Write};

fn main() {
    env_logger::init();
    info!("Omni Forge CLI v5.1 Production Starting...");

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage();
        return;
    }

    let command = &args[1];
    let forge = OmniForge::new();

    match command.as_str() {
        "fabricate" => {
            if args.len() < 3 {
                println!("Usage: omni-forge fabricate <data_path>");
                return;
            }
            let data_path = &args[2];
            println!("Fabricating mind from: {}", data_path);
            match forge.fabricate_mind(data_path, "mind.omf") {
                Ok(_) => println!("Success: Mind fabricated to mind.omf"),
                Err(e) => error!("Fabrication failed: {}", e),
            }
        },
        "run" => {
            let mind_path = if args.len() >= 3 { &args[2] } else { "mind.omf" };
            println!("Loading sovereign mind from {}...", mind_path);
            if let Err(e) = forge.load_mind(mind_path) {
                error!("Failed to load mind: {}", e);
                return;
            }
            println!("Mind Loaded. Entering Runtime Mode (Read-Only).");
            println!("Type 'exit' to quit.");

            loop {
                print!("> ");
                io::stdout().flush().unwrap();
                let mut input = String::new();
                if io::stdin().read_line(&mut input).is_err() { break; }
                let input = input.trim();
                if input == "exit" { break; }
                if input.is_empty() { continue; }

                let answer = forge.run_query(input);
                println!("Mind: {}", answer);
            }
        },
        "inspect" => {
            println!("Inspect feature coming soon.");
        },
        "status" => {
            println!("Omni Forge Status: Operational");
        },
        _ => print_usage(),
    }
}

fn print_usage() {
    println!("Omni Forge CLI Usage:");
    println!("  omni-forge fabricate <data_path>");
    println!("  omni-forge run <mind.omf>");
    println!("  omni-forge inspect <mind.omf>");
    println!("  omni-forge status");
}
