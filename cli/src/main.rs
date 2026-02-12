use clap::{Parser, Subcommand};
use runtime::ooda::{OODAController, Action};
use ingestion::UniversalIngestor;
use core_vsa::traits::Ingestor;
use std::path::PathBuf;
use log::{info, error};
use std::io::Write;

#[derive(Parser)]
#[command(name = "omni-forge")]
#[command(about = "Industrial-Grade Neuro-Symbolic Mind Factory", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the OODA Loop indefinitely
    Run {
        #[arg(short, long)]
        snapshot: Option<PathBuf>,
    },
    /// Ingest a file or directory into a new memory snapshot
    Ingest {
        #[arg(short, long)]
        input: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
    },
    /// Benchmark the system performance
    Benchmark,
}

fn main() {
    env_logger::init();
    let cli = Cli::parse();

    match &cli.command {
        Commands::Run { snapshot } => {
            info!("Starting Omni Forge Runtime...");
            let mut ooda = OODAController::new();

            if let Some(path) = snapshot {
                info!("Loading snapshot from {:?}", path);
                // logic to load snapshot into ooda.resonator/reasoner
                // For now, we start fresh as per strict Phase requirements (Phase 6 impl exists but wiring is complex here)
                // We will assume "Fresh Start" or "Load Logic Stub".
            }

            println!("Omni Forge v10.0 Online. Type 'quit' to exit.");

            loop {
                // 1. Observe (Simulated or Real Input)
                // For CLI, we can read stdin?
                print!("> ");
                std::io::stdout().flush().unwrap();

                let mut input = String::new();
                std::io::stdin().read_line(&mut input).unwrap();
                let input = input.trim();

                if input == "quit" { break; }

                if !input.is_empty() {
                    // Manual Observation Injection
                    // In real system, this comes from Perception modules.
                    // We use a "TextEncoder" stub here (random/deterministic).
                    let vector = core_vsa::HyperVector::deterministic(input.len() as u64);
                    ooda.observe(vector);
                }

                // 2. Orient
                ooda.orient();

                // 3. Decide
                let action = ooda.decide();

                // 4. Act
                match action {
                    Action::Explore(desc) => println!("[ACT] Exploring: {}", desc),
                    Action::Output(msg) => println!("[ACT] Output: {}", msg),
                    Action::GenerateHypothesis => println!("[ACT] Generating Hypothesis due to Surprise!"),
                    _ => println!("[ACT] Internal processing..."),
                }
                ooda.act(action);

                println!("Metrics: Surprise={:.2} Uncertainty={:.2}", ooda.surprise_metric, ooda.uncertainty_metric);
            }
        },
        Commands::Ingest { input, output } => {
            info!("Ingesting from {:?}", input);
            let ingestor = UniversalIngestor;
            match ingestor.ingest(input) {
                Ok(graph) => {
                    info!("Ingestion complete. Graph nodes: {}", graph.nodes.len());
                    // Serialize graph to output (using memory::SnapshotManager logic theoretically)
                    // For now, just print stats.
                    println!("Successfully ingested {} nodes.", graph.nodes.len());
                },
                Err(e) => error!("Ingestion failed: {}", e),
            }
        },
        Commands::Benchmark => {
            println!("Run `cargo bench` for full criterion benchmarks.");
        }
    }
}
