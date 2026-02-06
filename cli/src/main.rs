use fabricator::facade::OmniForge;
use log::{info, error};
use std::env;
use std::io::{self, Write};
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

fn main() {
    // Only init logger if RUST_LOG is set, otherwise default to quiet for CLI cleanliness
    if env::var("RUST_LOG").is_ok() {
        env_logger::init();
    }

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
            println!(" Omni Forge v5.1 Fabricator");
            println!("===============================");
            println!("Ingesting reality from: {}", data_path);

            let pb = ProgressBar::new_spinner();
            pb.set_style(ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .unwrap()
                .tick_chars("-/|\\"));
            pb.set_message("Ingesting & Learning...");
            pb.enable_steady_tick(Duration::from_millis(100));

            // Fabrication happens here (blocking)
            match forge.fabricate_mind(data_path, "mind.omf") {
                Ok(_) => {
                    pb.finish_with_message("Fabrication Complete!");
                    println!("Success: Sovereign Mind fabricated to 'mind.omf'");
                },
                Err(e) => {
                    pb.finish_with_message("Fabrication Failed");
                    error!("Error: {}", e);
                }
            }
        },
        "run" => {
            let mind_path = if args.len() >= 3 { &args[2] } else { "mind.omf" };
            println!(" Omni Forge v5.1 Runtime");
            println!("===========================");
            println!("Loading sovereign mind from {}...", mind_path);

            if let Err(e) = forge.load_mind(mind_path) {
                error!("Failed to load mind: {}", e);
                println!("Error: Could not load mind artifact. Ensure path is correct and file is valid.");
                return;
            }

            println!("Mind Loaded. Entering Read-Only Mode.");
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
        "query" => {
             if args.len() < 4 {
                println!("Usage: omni-forge query <mind.omf> <question>");
                return;
             }
             let mind_path = &args[2];
             let question = &args[3..].join(" ");

             if let Err(e) = forge.load_mind(mind_path) {
                error!("Failed to load mind: {}", e);
                return;
             }
             let answer = forge.run_query(question);
             println!("{}", answer);
        },
        "inspect" => {
            let mind_path = if args.len() >= 3 { &args[2] } else { "mind.omf" };
            println!("Inspecting mind artifact: {}", mind_path);
            match forge.inspect_mind(mind_path) {
                Ok(info) => println!("{}", info),
                Err(e) => error!("Failed to inspect mind: {}", e),
            }
        },
        "status" => {
            println!(" Omni Forge v5.1 Status");
            println!("========================");
            println!("System: Operational");
            println!("Host Arch: {}", std::env::consts::ARCH);
            println!("Host OS:   {}", std::env::consts::OS);

            let profile = hte::detect();
            println!("\nHardware Truth Engine (HTE):");
            println!("  Cores: {} Physical / {} Logical", profile.physical_cores, profile.logical_cores);
            println!("  AVX2:  {}", if profile.avx2 { "Yes" } else { "No" });
            println!("  NEON:  {}", if profile.neon { "Yes" } else { "No" });

            if !profile.avx2 && !profile.neon {
                println!("\n[WARNING] No SIMD acceleration detected. Performance may be degraded.");
            }
        },
        _ => print_usage(),
    }
}

fn print_usage() {
    println!("Omni Forge v5.1 (Industrial Compiler) Usage:");
    println!("  omni-forge fabricate <data_path>        # Build a sovereign mind from raw data");
    println!("  omni-forge run <mind.omf>               # Interactive read-only runtime shell");
    println!("  omni-forge query <mind.omf> <question>  # Single-shot query execution");
    println!("  omni-forge inspect <mind.omf>           # Audit mind artifact metadata");
    println!("  omni-forge status                       # System health and HTE report");
}
