use fabricator::facade::OmniForge;
use log::{info, warn};
use std::io::{self, Write};

fn main() {
    env_logger::init();
    info!("Omni Forge CLI v5.1 Starting...");

    // Hardware check
    println!("Initializing Hardware Truth Engine...");
    let profile = hte::detect();
    println!("HTE Profile Detected:\n{:#?}", profile);

    println!("Initializing OmniForge Facade...");
    let forge = OmniForge::new();

    println!("\nOmni Forge Interactive Shell.");
    println!("Commands:");
    println!("  forge fabricate <source>  - Create mind from source");
    println!("  forge run                 - Enter interactive mode (load default mind.omf)");
    println!("  forge export <path>       - Export current mind");
    println!("  forge inspect             - Show stats");
    println!("  exit                      - Quit");

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

        let mut parts = input.splitn(3, ' ');
        let cmd = parts.next().unwrap_or("");
        let subcmd = parts.next().unwrap_or("");
        let args = parts.next().unwrap_or("");

        if cmd == "forge" {
            match subcmd {
                "fabricate" => {
                    let source = if args.is_empty() { "default" } else { args };
                    match forge.fabricate_mind(source, "mind.omf") {
                        Ok(_) => println!("Fabrication complete. Saved to mind.omf"),
                        Err(e) => println!("Fabrication failed: {}", e),
                    }
                },
                "run" => {
                    // Load default
                    if let Err(_) = forge.load_mind("mind.omf") {
                        warn!("Could not load mind.omf. Running with empty mind.");
                    }
                    println!("Entering runtime loop. Type 'back' to return.");
                    loop {
                        print!("(run) > ");
                        io::stdout().flush().unwrap();
                        let mut q = String::new();
                        if io::stdin().read_line(&mut q).is_err() { break; }
                        let q = q.trim();
                        if q == "back" { break; }
                        let ans = forge.run_query(q);
                        println!("Mind: {}", ans);
                    }
                },
                "export" => {
                    let path = if args.is_empty() { "exported_mind.omf" } else { args };
                    // We need to expose save on facade.
                    // But fabricate saves. Export current memory?
                    // Impl save logic or skip for prototype.
                    println!("Export to {} not implemented in Facade yet.", path);
                },
                "inspect" => {
                    println!("Inspection not implemented.");
                },
                _ => println!("Unknown forge command."),
            }
        } else {
            println!("Use 'forge <command>'");
        }
    }
}
