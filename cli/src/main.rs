use fabricator::facade::OmniForge;
use log::error;
use std::env;
use std::io::{self, Write};
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;
use std::path::{Path, PathBuf};

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
                std::process::exit(1);
            } else {
                println!("Forge Master initialized at {:?}", path);
                println!("Portable Data Directory: ./omniforge_data/");
            }
        },
        "ingest" => {
            if args.len() < 3 {
                eprintln!("Usage: omniforge ingest <data_path>");
                std::process::exit(1);
            }
            let data_path = &args[2];
            let master_path = get_forge_master_path();

            if !master_path.exists() {
                eprintln!("Forge Master not found. Run 'omniforge init' first.");
                std::process::exit(1);
            }

            println!("Loading Forge Master...");
            if let Err(e) = forge.load_forge_master(master_path.to_str().unwrap()) {
                error!("Failed to load Forge Master: {}", e);
                std::process::exit(1);
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
                std::process::exit(1);
            } else {
                println!("Forge Master updated.");
            }
        },
        "snapshot" => {
             // omniforge snapshot --name <name>
             let mut name = "snapshot".to_string();
             if let Some(idx) = args.iter().position(|x| x == "--name") {
                 if let Some(val) = args.get(idx + 1) {
                     name = val.clone();
                 }
             } else if args.len() >= 3 && !args[2].starts_with("--") {
                 // Fallback: omniforge snapshot <name>
                 name = args[2].clone();
             }

             let master_path = get_forge_master_path();
             if !master_path.exists() {
                eprintln!("Forge Master not found. Run 'omniforge init' first.");
                std::process::exit(1);
             }

             println!("Loading Forge Master...");
             if let Err(e) = forge.load_forge_master(master_path.to_str().unwrap()) {
                 error!("Failed to load Forge Master: {}", e);
                 std::process::exit(1);
             }

             let output_path = format!("{}.mindpack", name);
             println!("Freezing Forge into MindPack: {}", output_path);

             // Version hardcoded for product release
             if let Err(e) = forge.snapshot("1.0.0", "User Snapshot", &output_path) {
                 error!("Snapshot failed: {}", e);
                 std::process::exit(1);
             } else {
                 println!("Success! Artifact created: {}", output_path);
                 println!("To distribute, run: omniforge compress {}", output_path);
             }
        },
        "export" => {
            if args.len() < 4 {
                eprintln!("Usage: omniforge export <mindpack> <output.json>");
                std::process::exit(1);
            }
            let source = &args[2];
            let dest = &args[3];

            println!("Exporting Knowledge Graph from {}...", source);
            let pack = memory::load_snapshot(source).expect("Failed to load mindpack");

            let json = serde_json::to_string_pretty(&pack.memory.core.relation_graph).unwrap();
            std::fs::write(dest, json).expect("Failed to write output");
            println!("Exported to {}", dest);
        },
        "compress" => {
            // omniforge compress <name>.mindpack
            if args.len() < 3 {
                eprintln!("Usage: omniforge compress <name>.mindpack");
                std::process::exit(1);
            }
            let source = &args[2];
            if !std::path::Path::new(source).exists() {
                 eprintln!("File not found: {}", source);
                 std::process::exit(1);
            }

            let dest = format!("{}.zip", source);
            println!("Compressing {} -> {} ...", source, dest);

            // In our case, .mindpack IS a zip. We just copy it to .zip for "friend" usage.
            if let Err(e) = std::fs::copy(source, &dest) {
                error!("Compression failed: {}", e);
                std::process::exit(1);
            }
            println!("Done. Send {} to your friend.", dest);
        },
        "run" => {
            // omniforge run <name>.mindpack.zip
            if args.len() < 3 {
                eprintln!("Usage: omniforge run <name>.mindpack[.zip]");
                std::process::exit(1);
            }
            let path_str = &args[2];
            let path = Path::new(path_str);

            if !path.exists() {
                eprintln!("MindPack not found: {}", path_str);
                std::process::exit(1);
            }

            // Calculate overlay path in ./omniforge_data/overlays/
            let file_stem = path.file_stem().unwrap().to_str().unwrap();
            // Handle double extension .mindpack.zip -> .mindpack -> stem
            let clean_stem = if file_stem.ends_with(".mindpack") {
                Path::new(file_stem).file_stem().unwrap().to_str().unwrap()
            } else {
                file_stem
            };

            let mut overlay_dir = PathBuf::from("omniforge_data");
            overlay_dir.push("overlays");
            std::fs::create_dir_all(&overlay_dir).unwrap_or(());

            let overlay_path = overlay_dir.join(format!("{}.local", clean_stem));
            let overlay_path_str = overlay_path.to_str().unwrap();

            println!(" Omni Forge Runtime");
            println!("====================");
            println!("Base Mind: {}", path_str);
            println!("Personal Memory: {}", overlay_path_str);

            if let Err(e) = forge.load_runtime(path_str, Some(overlay_path_str)) {
                error!("Failed to load runtime: {}", e);
                std::process::exit(1);
            }

            // Safe Shutdown Handler
            let runtime_ref = forge.runtime_mind.clone();
            let save_path = overlay_path_str.to_string();
            ctrlc::set_handler(move || {
                println!("\n[System] Graceful Shutdown initiated...");
                let slot = runtime_ref.lock().unwrap();
                if let Some(runtime) = slot.as_ref() {
                    if let Err(e) = memory::save_personal(&runtime.overlay, &save_path) {
                        println!("[Error] Failed to save session: {}", e);
                    } else {
                        println!("[System] Session saved successfully.");
                    }
                }
                std::process::exit(0);
            }).expect("Error setting signal handler");

            println!("Mind Online. (Type 'exit' to quit)");

            loop {
                print!("> ");
                io::stdout().flush().unwrap();
                let mut input = String::new();
                match io::stdin().read_line(&mut input) {
                    Ok(0) => break,
                    Ok(_) => {},
                    Err(_) => break,
                }
                let input = input.trim();
                if input == "exit" || input == "quit" { break; }
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
                     if let Err(e) = forge.save_runtime_overlay(overlay_path_str) {
                         error!("Save failed: {}", e);
                     } else {
                         println!("Local Overlay saved.");
                     }
                     continue;
                }

                if input == "/goals" {
                     let slot = forge.runtime_mind.lock().unwrap();
                     if let Some(runtime) = slot.as_ref() {
                         let goals = runtime.overlay.core.active_goals();
                         if goals.is_empty() {
                             println!("No active goals.");
                         } else {
                             println!("Active Goals:");
                             for g in goals {
                                 println!("- {} (Prio: {})", g.description, g.priority);
                             }
                         }
                     }
                     continue;
                }

                let answer = forge.run_query(input);
                println!("{}", answer);
            }

            println!("Saving session...");
            if let Err(e) = forge.save_runtime_overlay(overlay_path_str) {
                error!("Failed to save final session: {}", e);
            } else {
                println!("Session saved.");
            }
        },
        "verify" => {
            if args.len() < 3 {
                eprintln!("Usage: omniforge verify <mindpack>");
                std::process::exit(1);
            }
            let path = &args[2];
            println!("Verifying integrity of {}...", path);
            match memory::verify_integrity(path) {
                Ok(_) => {
                     // Load again for metadata display (verify_integrity returns bool)
                     let pack = memory::load_snapshot(path).unwrap();
                     println!(" Integrity OK: [MATCH]");
                     println!(" Core Hash: {}", pack.metadata.core_hash);
                     println!(" Version: {}", pack.version);
                     println!(" Source: {}", pack.metadata.source);
                     println!(" Arch: {}", pack.metadata.arch);
                },
                Err(e) => {
                     eprintln!(" Verification FAILED: {}", e);
                     std::process::exit(1);
                }
            }
        },
        "status" => {
            let data_dir = PathBuf::from("omniforge_data");
            println!("Omni Forge Status");
            println!("=================");
            println!("Version: {}", env!("CARGO_PKG_VERSION"));
            println!("Mode: Portable");
            println!("Data Directory: {:?}", std::fs::canonicalize(&data_dir).unwrap_or(data_dir));

            let profile = hte::detect();
            println!("Memory Usage: {}/{} KB", profile.used_memory, profile.total_memory);
            println!("Safety Margin: {:.2}", profile.memory_budget_margin);
        },
        "doctor" => {
            let profile = hte::detect();
            println!("System Doctor (Health Check)");
            println!("============================");
            println!("Host OS: {} {}", profile.os_name, profile.kernel_version);
            println!("CPU Cores: {} (Physical: {})", profile.logical_cores, profile.physical_cores);
            println!("Instruction Sets: AVX2={}, NEON={}, AMX={}", profile.avx2, profile.neon, profile.amx);
            println!("Memory Health: {:.2}% free", profile.memory_budget_margin * 100.0);

            if profile.memory_budget_margin < 0.1 {
                println!("[WARNING] Low Memory! Performance may degrade.");
            } else {
                println!("[OK] System healthy.");
            }
        },
        "inspect" => {
            let path = if args.len() >= 3 { &args[2] } else { "mind.omf" };
            println!("Inspecting artifact: {}", path);
            match forge.inspect_mind(path) {
                Ok(info) => println!("{}", info),
                Err(e) => {
                    error!("Failed to inspect: {}", e);
                    std::process::exit(1);
                }
            }
        },
        "version" => {
            println!("Omni Forge v{}", env!("CARGO_PKG_VERSION"));
        },
        _ => {
            print_usage();
            std::process::exit(1);
        }
    }
}

fn get_forge_master_path() -> PathBuf {
    let mut path = PathBuf::from("omniforge_data");
    std::fs::create_dir_all(&path).unwrap_or(());
    path.push("forge_master.omf");
    path
}

fn print_usage() {
    println!("Omni Forge v8.9 Usage:");
    println!("  omniforge init                  # Initialize new Forge Master in ./omniforge_data");
    println!("  omniforge ingest <path>         # Feed data to Forge Master");
    println!("  omniforge snapshot --name <n>   # Freeze Forge into <n>.mindpack");
    println!("  omniforge export <f> <out>      # Export Knowledge Graph to JSON");
    println!("  omniforge compress <f>          # Compress .mindpack -> .mindpack.zip");
    println!("  omniforge run <f>               # Run MindPack (creates local overlay)");
    println!("  omniforge verify <f>            # Verify MindPack integrity");
    println!("  omniforge status                # Show directory and memory info");
    println!("  omniforge doctor                # Deep system health check");
    println!("  omniforge inspect <f>           # View metadata");
}
