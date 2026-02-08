use fabricator::facade::OmniForge;
use log::{info, error};
use std::env;
use std::io::{self, Write};
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;
use directories::ProjectDirs;
use std::path::PathBuf;

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
        "init" => {
            let path = get_forge_master_path();
            if path.exists() {
                println!("Forge Master already exists at {:?}", path);
                return;
            }
            println!("Initializing Sovereign Forge Master...");
            if let Err(e) = forge.save_forge_master(path.to_str().unwrap()) {
                error!("Failed to initialize Forge: {}", e);
            } else {
                println!("Forge Master initialized at {:?}", path);
            }
        },
        "ingest" => {
            if args.len() < 3 {
                println!("Usage: omniforge ingest <data_path>");
                return;
            }
            let data_path = &args[2];
            let master_path = get_forge_master_path();

            if !master_path.exists() {
                println!("Forge Master not found. Run 'omniforge init' first.");
                return;
            }

            println!("Loading Forge Master...");
            if let Err(e) = forge.load_forge_master(master_path.to_str().unwrap()) {
                error!("Failed to load Forge Master: {}", e);
                return;
            }

            println!("Ingesting reality from: {}", data_path);
            let pb = ProgressBar::new_spinner();
            pb.set_style(ProgressStyle::default_spinner().template("{spinner:.green} {msg}").unwrap().tick_chars("-/|\\"));
            pb.set_message("Learning...");
            pb.enable_steady_tick(Duration::from_millis(100));

            let chunks = ingestion::ingest_path(PathBuf::from(data_path));
            {
                let mut mind = forge.forge_mind.lock().unwrap();
                for chunk in chunks {
                    mind.learn(&chunk.content);
                }
            }

            pb.finish_with_message("Ingestion Complete!");

            if let Err(e) = forge.save_forge_master(master_path.to_str().unwrap()) {
                error!("Failed to save Forge Master: {}", e);
            } else {
                println!("Forge Master updated.");
            }
        },
        "snapshot" => {
             if args.len() < 3 {
                println!("Usage: omniforge snapshot <version> [output_path]");
                return;
             }
             let version = &args[2];
             let output_path = if args.len() >= 4 { args[3].clone() } else { format!("snapshot_v{}.omf", version) };
             let master_path = get_forge_master_path();

             if !master_path.exists() {
                println!("Forge Master not found. Run 'omniforge init' first.");
                return;
             }

             println!("Loading Forge Master...");
             if let Err(e) = forge.load_forge_master(master_path.to_str().unwrap()) {
                 error!("Failed to load Forge Master: {}", e);
                 return;
             }

             println!("Creating Snapshot v{}...", version);
             if let Err(e) = forge.snapshot(version, "Forge CLI", &output_path) {
                 error!("Snapshot failed: {}", e);
             } else {
                 println!("Snapshot created at {}", output_path);
             }
        },
        "pack" => {
             if args.len() < 3 {
                println!("Usage: omniforge pack --target=<mobile|desktop|server> [output_path]");
                return;
             }
             let target_arg = &args[2];
             let target = if target_arg.starts_with("--target=") { &target_arg[9..] } else { "desktop" };
             let output_path = if args.len() >= 4 { args[3].clone() } else { format!("mind_{}.omf", target) };

             let master_path = get_forge_master_path();
             if !master_path.exists() {
                println!("Forge Master not found.");
                return;
             }

             println!("Packing Mind for Target: {}", target);

             // Load master temporarily to pack it
             if let Err(e) = forge.load_forge_master(master_path.to_str().unwrap()) {
                 error!("Failed load master: {}", e);
                 return;
             }

             let session = fabricator::ForgeSession {
                 mind: std::mem::take(&mut *forge.forge_mind.lock().unwrap()),
                 config: fabricator::ForgeConfig::default(),
                 learning_engine: learning::LearningEngine::new(),
             };

             if let Err(e) = session.pack(target, &output_path) {
                 error!("Packing failed: {}", e);
             } else {
                 println!("Mind packed successfully to {}", output_path);
             }
             // Restore mind? No, CLI exits.
        },
        "clone" => {
             if args.len() < 4 {
                println!("Usage: omniforge clone <source.omf> <dest.omf>");
                return;
             }
             let source = &args[2];
             let dest = &args[3];
             println!("Cloning mind artifact...");
             if let Err(e) = std::fs::copy(source, dest) {
                 error!("Clone failed: {}", e);
             } else {
                 println!("Cloned {} to {}. (Independent Sovereign Instance)", source, dest);
             }
        },
        "run" => {
            if args.len() < 3 {
                println!("Usage: omniforge run <snapshot.omf>");
                return;
            }
            let snapshot_path = &args[2];
            let overlay_path = format!("{}.local", snapshot_path);

            println!(" Omni Forge v8.6 Runtime");
            println!("=========================");
            println!("Loading Base: {}", snapshot_path);

            if let Err(e) = forge.load_runtime(snapshot_path, Some(&overlay_path)) {
                error!("Failed to load runtime: {}", e);
                return;
            }

            println!("Runtime Ready. (Type 'exit' to quit)");

            loop {
                print!("> ");
                io::stdout().flush().unwrap();
                let mut input = String::new();
                if io::stdin().read_line(&mut input).is_err() { break; }
                let input = input.trim();
                if input == "exit" { break; }
                if input.is_empty() { continue; }

                if input.starts_with("/learn ") {
                    let content = &input[7..];
                    if let Err(e) = forge.learn_runtime(content) {
                        error!("Learning failed: {}", e);
                    } else {
                        println!("(Remembered in Local Overlay)");
                    }
                    continue;
                }

                if input == "/save" {
                     if let Err(e) = forge.save_runtime_overlay(&overlay_path) {
                         error!("Save failed: {}", e);
                     } else {
                         println!("Local Overlay saved to {}", overlay_path);
                     }
                     continue;
                }

                let answer = forge.run_query(input);
                println!("{}", answer);
            }

            println!("Saving session...");
            let _ = forge.save_runtime_overlay(&overlay_path);
        },
        "inspect" => {
            let path = if args.len() >= 3 { &args[2] } else { "mind.omf" };
            println!("Inspecting artifact: {}", path);
            match forge.inspect_mind(path) {
                Ok(info) => println!("{}", info),
                Err(e) => error!("Failed to inspect: {}", e),
            }
        },
        "status" | "health" => {
            let profile = hte::detect();
            println!("System Status (Health Check):");
            println!("  OS: {} {}", profile.os_name, profile.kernel_version);
            println!("  Memory: {}/{} KB (Margin: {:.2})", profile.used_memory, profile.total_memory, profile.memory_budget_margin);
            println!("  Cores: {}", profile.logical_cores);
            if profile.memory_budget_margin < 0.1 {
                println!("  WARNING: Low Memory Margin!");
            } else {
                println!("  Status: HEALTHY");
            }
        },
        _ => print_usage(),
    }
}

fn get_forge_master_path() -> PathBuf {
    if let Some(proj_dirs) = ProjectDirs::from("com", "omni-forge", "omni-forge") {
        let mut path = proj_dirs.data_dir().to_path_buf();
        std::fs::create_dir_all(&path).unwrap_or(());
        path.push("forge_master.omf");
        path
    } else {
        PathBuf::from("forge_master.omf")
    }
}

fn print_usage() {
    println!("Omni Forge v8.6 Usage:");
    println!("  omniforge init                  # Initialize new Forge Master");
    println!("  omniforge ingest <path>         # Feed data to Forge Master");
    println!("  omniforge snapshot <ver> [out]  # Export frozen Mind Artifact");
    println!("  omniforge pack --target=<t>     # Create optimized artifact");
    println!("  omniforge clone <src> <dst>     # Duplicate an artifact");
    println!("  omniforge run <mind.omf>        # Run sovereign mind with local overlay");
    println!("  omniforge inspect <mind.omf>    # View metadata");
    println!("  omniforge health                # System diagnostics");
}
