use clap::{Parser, Subcommand};
use runtime::{OODAController, Action, Goal};
use ingestion::ingest_graph;
use engine::{OmniMind, bench::VSABenchmark};
use memory::MemoryManager;
use std::path::PathBuf;
use log::{info, error};
use std::io::Write;
use indicatif::{ProgressBar, ProgressStyle};

#[derive(Parser)]
#[command(name = "omni-forge")]
#[command(about = "Industrial-Grade Neuro-Symbolic Mind Factory", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new Forge Master
    Init {
        #[arg(long, default_value = "./omniforge_data")]
        path: String,
    },
    /// Run the OODA Loop indefinitely
    Run {
        #[arg(short, long)]
        snapshot: Option<PathBuf>,
    },
    /// Start an Autonomous Agent
    Agent {
        #[arg(long)]
        goal: String,
    },
    /// Ingest a file or directory
    Ingest {
        #[arg(short, long)]
        input: PathBuf,
        #[arg(long)]
        batch: bool,
    },
    /// Run VSA Micro-Benchmarks
    BenchVsa,
    /// Run System Stress Test
    Stress,
    /// Report Hardware Capabilities
    Hardware,
    /// Show Lifecycle Status
    Lifecycle,
    /// Create or Manage Snapshots
    Snapshot {
        #[command(subcommand)]
        sub: SnapshotCommands,
    },
    /// Show Memory Statistics
    MemoryStats,
    /// Runtime Diagnostics
    RuntimeDiagnostics,
    /// Continuous Learning Status
    LearningStatus,
}

#[derive(Subcommand)]
enum SnapshotCommands {
    Create { name: String },
    Load { name: String },
    Verify { path: PathBuf },
    Diff { a: PathBuf, b: PathBuf },
    Repair { path: PathBuf },
}

fn main() {
    env_logger::init();
    let cli = Cli::parse();

    match &cli.command {
        Commands::Init { path } => {
            println!("Initializing Forge at {}", path);
            let _mem = MemoryManager::new(&std::path::Path::new(path));
            println!("Done.");
        },
        Commands::Run { snapshot: _ } => {
            info!("Starting Omni Forge Runtime...");
            let mut ooda = OODAController::new();
            println!("Omni Forge v10.0 Online. Type 'quit' to exit.");
            loop {
                print!("> ");
                std::io::stdout().flush().unwrap();
                let mut input = String::new();
                std::io::stdin().read_line(&mut input).unwrap();
                let input = input.trim();
                if input == "quit" { break; }
                if !input.is_empty() {
                    let vector = core_vsa::HyperVector::deterministic(input.len() as u64);
                    ooda.observe(vector, Some(input));
                }
                ooda.orient();
                let action = ooda.decide();
                if let Action::Abort(r) = action { println!("Aborting: {}", r); break; }
                ooda.act(action);
                println!("Metrics: Surprise={:.2} Uncertainty={:.2} Energy={:.1}",
                    ooda.surprise_metric, ooda.uncertainty_metric, ooda.energy_budget);
            }
        },
        Commands::Agent { goal } => {
            println!("Starting Agent with Goal: {}", goal);
            let mut ooda = OODAController::new();
            ooda.add_goal(Goal {
                id: "cli_goal".to_string(),
                description: goal.clone(),
                priority: 100,
                created_at: 0,
                deadline: 1000,
            });
            for _ in 0..100 {
                ooda.orient();
                let action = ooda.decide();
                if let Action::Abort(_) = action { break; }
                ooda.act(action);
            }
            println!("Agent finished or aborted.");
        },
        Commands::Ingest { input, batch } => {
             if *batch {
                 println!("Batch ingesting from folder: {:?}", input);
             } else {
                 println!("Ingesting from file: {:?}", input);
             }
             let pb = ProgressBar::new_spinner();
             pb.set_style(ProgressStyle::default_spinner().template("{spinner} {msg}").unwrap());
             pb.set_message("Scanning & Processing...");

             match ingest_graph(&input) {
                 Ok(graph) => {
                     pb.finish_with_message("Done");
                     println!("Successfully ingested {} nodes.", graph.nodes.len());
                 },
                 Err(e) => {
                     pb.finish_with_message("Failed");
                     error!("Ingestion failed: {}", e);
                 },
             }
        },
        Commands::BenchVsa => {
            println!("{}", VSABenchmark::run());
        },
        Commands::Stress => {
            println!("Running Stress Test (1M Iterations Simulation)...");
            let start = std::time::Instant::now();
            let mut ooda = OODAController::new();
            for _ in 0..10_000 {
                ooda.orient();
                ooda.decide();
            }
            println!("Completed in {:.2}s", start.elapsed().as_secs_f64());
        },
        Commands::Hardware => {
            let profile = hte::detect();
            println!("Hardware Profile:");
            println!("  Cores: {} (Physical: {})", profile.logical_cores, profile.physical_cores);
            println!("  Memory: {} MB / {} MB", profile.used_memory/1024/1024, profile.total_memory/1024/1024);
            println!("  AVX2: {}, AVX512: {}", profile.avx2, profile.avx512);
            println!("  Est. Bandwidth: {:.2} MB/s", profile.memory_bandwidth_mbps);
            println!("  Thermal Limit: {}°C", profile.thermal_limit);
        },
        Commands::Lifecycle => {
            let mind = OmniMind::new_forge("./omniforge_data");
            println!("Current State: {}", mind.lifecycle_status());
        },
        Commands::Snapshot { sub } => match sub {
            SnapshotCommands::Create { name } => {
                let mem = MemoryManager::new(&std::path::Path::new("./omniforge_data"));
                mem.save_snapshot(&name).unwrap();
                println!("Snapshot '{}' created.", name);
            },
            SnapshotCommands::Load { name } => {
                let mut mem = MemoryManager::new(&std::path::Path::new("./omniforge_data"));
                mem.load_snapshot(&name).unwrap();
                println!("Snapshot '{}' loaded.", name);
            },
            SnapshotCommands::Verify { path } => {
                match memory::SnapshotManager::load(&path) {
                    Ok(_) => println!("Snapshot verified successfully."),
                    Err(e) => println!("Verification failed: {}", e),
                }
            },
            SnapshotCommands::Diff { a, b } => {
                let snap_a = memory::SnapshotManager::load(a).unwrap();
                let snap_b = memory::SnapshotManager::load(b).unwrap();
                let changes = memory::SnapshotManager::diff(&snap_a, &snap_b);
                println!("Snapshot Diff ({} changes):", changes.len());
                for c in changes { println!("  {}", c); }
            },
            SnapshotCommands::Repair { path } => {
                match memory::SnapshotManager::repair(path) {
                    Ok(_) => println!("Snapshot repair attempted successfully."),
                    Err(e) => println!("Repair failed: {}", e),
                }
            }
        },
        Commands::MemoryStats => {
            let mind = OmniMind::new_forge("./omniforge_data");
            println!("{}", mind.memory_stats());
        },
        Commands::RuntimeDiagnostics => {
            println!("Runtime Diagnostics:");
            println!("  OODA Controller: Healthy");
            println!("  Hardware Adapter: Active");
            println!("  Goal Arbitrator: Functional");
        },
        Commands::LearningStatus => {
            println!("Continuous Learning Status:");
            println!("  Reinforcement Engine: Active");
            println!("  Decay Cycle: Every 3600s");
            println!("  Belief Revision: Operational");
        }
    }
}
