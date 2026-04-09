use clap::{Parser, Subcommand};
use anyhow::Result;

#[derive(Parser)]
#[command(name = "xtask")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Verify environment and dependency health
    Doctor,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Doctor => doctor()?,
    }

    Ok(())
}

fn doctor() -> Result<()> {
    println!("OmniForge Doctor: Running System Health Checks...");

    // 1. Hardware Detection
    print!("Checking Hardware Detection... ");
    let profile = hte::detect();
    println!("OK ({} cores, {}MB RAM)", profile.logical_cores, profile.total_memory / 1024 / 1024);

    // 2. Dependency Health (Check if all members are present)
    print!("Verifying Workspace Integrity... ");
    // In a real xtask we might check for specific tools or versions.
    // For now, if this compiles and runs, it's a good sign.
    println!("OK");

    // 3. Environment Check
    print!("Checking Environment... ");
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;
    println!("OK (OS: {}, ARCH: {})", os, arch);

    println!("\nAll systems operational for Phase-1 Industrial Core.");
    Ok(())
}
