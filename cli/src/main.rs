use fabricator::facade::OmniForge;
use fabricator::live::LiveFabricator;
use log::{info, error};
use std::env;
use std::io::{self, Write};
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

fn main() {
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
            println!(" Omni Forge v6.0 Industrial Fabricator");
            println!("=======================================");
            println!("Ingesting reality from: {}", data_path);

            let pb = ProgressBar::new_spinner();
            pb.set_style(ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .unwrap()
                .tick_chars("-/|\\"));
            pb.set_message("Ingesting & Learning (Structural Analysis)...");
            pb.enable_steady_tick(Duration::from_millis(100));

            match forge.fabricate_mind(data_path, "mind.omf") {
                Ok(_) => {
                    pb.finish_with_message("Fabrication Complete!");
                    println!("Success: Sovereign Mind fabricated to 'mind.omf' (Binary Pack).");
                },
                Err(e) => {
                    pb.finish_with_message("Fabrication Failed");
                    error!("Error: {}", e);
                }
            }
        },
        "learn" => {
             if args.len() < 4 || args[2] != "--watch" {
                 println!("Usage: omni-forge learn --watch <path>");
                 return;
             }
             let watch_path = &args[3];
             println!(" Omni Forge v6.0 Live Learner");
             println!("============================");
             let live = LiveFabricator::new();
             if let Err(e) = live.watch(watch_path) {
                 error!("Live learning failed: {}", e);
             }
        },
        "run" => {
            let mind_path = if args.len() >= 3 { &args[2] } else { "mind.omf" };
            println!(" Omni Forge v6.0 Runtime");
            println!("=========================");
            println!("Loading sovereign mind from {}...", mind_path);

            if let Err(e) = forge.load_mind(mind_path) {
                error!("Failed to load mind: {}", e);
                println!("Error: Could not load mind artifact. Ensure integrity hash matches.");
                return;
            }

            println!("Mind Loaded. Entering Read-Only Mode (Sovereign).");
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
        "explain" => {
             if args.len() < 4 {
                println!("Usage: omni-forge explain <mind.omf> <concept>");
                return;
             }
             let mind_path = &args[2];
             let concept = &args[3];

             if let Err(e) = forge.load_mind(mind_path) {
                error!("Failed to load mind: {}", e);
                return;
             }

             match forge.explain_concept(concept) {
                 Ok(explanation) => println!("{}", explanation),
                 Err(e) => error!("Explanation failed: {}", e),
             }
        },
        "export" => {
             if args.len() < 4 {
                println!("Usage: omni-forge export <mind.omf> <output.json>");
                return;
             }
             let mind_path = &args[2];
             let output_path = &args[3];

             if let Err(e) = forge.load_mind(mind_path) {
                error!("Failed to load mind: {}", e);
                return;
             }

             if let Err(e) = forge.export_mind(output_path) {
                 error!("Export failed: {}", e);
             } else {
                 println!("Mind exported to {}", output_path);
             }
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
            println!(" Omni Forge v6.0 Status");
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
        "freeze" => {
             println!("Freeze command: Artifacts are frozen by default upon fabrication in v6.0.");
        },
        _ => print_usage(),
    }
}

fn print_usage() {
    println!("Omni Forge v6.0 (Industrial Compiler) Usage:");
    println!("  omni-forge fabricate <data_path>        # Build a sovereign mind from raw data");
    println!("  omni-forge learn --watch <path>         # Live learning mode (watches for changes)");
    println!("  omni-forge run <mind.omf>               # Interactive read-only runtime shell");
    println!("  omni-forge query <mind.omf> <question>  # Single-shot query execution");
    println!("  omni-forge explain <mind.omf> <concept> # Explain a specific concept");
    println!("  omni-forge export <mind.omf> <out.json> # Export knowledge graph");
    println!("  omni-forge inspect <mind.omf>           # Audit mind artifact metadata");
    println!("  omni-forge freeze                       # (No-op) Mark artifact as production ready");
    println!("  omni-forge status                       # System health and HTE report");
}
