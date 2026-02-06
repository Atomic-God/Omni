use fabricator::facade::OmniForge;
use log::{info, error};
use std::env;
use std::io::{self, Write};

fn main() {
    env_logger::init();

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
            info!("Omni Forge v5.1 Fabricator: Ingesting reality from {}", data_path);
            match forge.fabricate_mind(data_path, "mind.omf") {
                Ok(_) => println!("Success: Mind fabricated to mind.omf"),
                Err(e) => error!("Fabrication failed: {}", e),
            }
        },
        "run" => {
            let mind_path = if args.len() >= 3 { &args[2] } else { "mind.omf" };
            info!("Omni Forge v5.1 Runtime: Loading sovereign mind from {}...", mind_path);

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
            let mind_path = if args.len() >= 3 { &args[2] } else { "mind.omf" };
            println!("Inspecting mind artifact: {}", mind_path);
            match forge.inspect_mind(mind_path) {
                Ok(info) => println!("Metadata:\n{}", info),
                Err(e) => error!("Failed to inspect mind: {}", e),
            }
        },
        "status" => {
            println!("Omni Forge v5.1 Status: Operational");
            println!("Host Architecture: {}", std::env::consts::ARCH);
            println!("Host OS: {}", std::env::consts::OS);

            let profile = hte::detect();
            println!("Hardware Truth Engine Profile:");
            println!("  Physical Cores: {}", profile.physical_cores);
            println!("  Logical Cores: {}", profile.logical_cores);
            println!("  AVX2: {}", profile.avx2);
            println!("  NEON: {}", profile.neon);
        },
        _ => print_usage(),
    }
}

fn print_usage() {
    println!("Omni Forge v5.1 (Production) Usage:");
    println!("  omni-forge fabricate <data_path>  # Ingest data and build a mind");
    println!("  omni-forge run <mind.omf>         # Run the mind in read-only mode");
    println!("  omni-forge inspect <mind.omf>     # View mind metadata");
    println!("  omni-forge status                 # Check system status and HTE");
}
