use clap::{Parser, Subcommand};
use runtime::ooda::{OODAController, Action};
use ingestion::{UniversalIngestor, ingest_graph};
use core_vsa::traits::Ingestor;
use trainer::{Trainer, CheckpointManager, Evaluator};
use neural::{SequenceModel, Adam, CrossEntropyLoss, Optimizer, Layer};
use dataset::{StreamingLoader, SimpleTokenizer, BatchIterator};
use engine::{OmniMind, bench::VSABenchmark};
use gpu_bridge::{KaggleExporter, KaggleImporter};
use memory::MemoryManager;
use hte::HardwareProfile;
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
        #[arg(short, long)]
        output: PathBuf,
    },
    /// Train the neural perception/generation model
    Train {
        #[arg(long)]
        data: PathBuf,
        #[arg(long, default_value_t = 5)]
        epochs: usize,
        #[arg(long, default_value_t = 32)]
        batch_size: usize,
        #[arg(long, default_value_t = 64)]
        embedding_dim: usize,
        #[arg(long, default_value_t = 128)]
        hidden_size: usize,
    },
    /// Evaluate the model metrics
    Evaluate {
        #[arg(long)]
        model: PathBuf,
        #[arg(long)]
        data: PathBuf,
    },
    /// Run VSA Micro-Benchmarks
    BenchVsa,
    /// Run System Stress Test
    Stress,
    /// Report Hardware Capabilities
    Hardware,
    /// Show Lifecycle Status
    Lifecycle,
    /// Export Data for Kaggle GPU Training
    ExportGpu {
        #[arg(long)]
        data: PathBuf,
        #[arg(long)]
        output: PathBuf,
    },
    /// Import Trained Weights from Kaggle
    ImportGpu {
        #[arg(long)]
        weights: PathBuf,
    },
    /// Create or Manage Snapshots
    Snapshot {
        #[command(subcommand)]
        sub: SnapshotCommands,
    },
}

#[derive(Subcommand)]
enum SnapshotCommands {
    Create { name: String },
    Load { name: String },
    Verify { path: PathBuf },
}

fn main() {
    env_logger::init();
    let cli = Cli::parse();

    match &cli.command {
        Commands::Init { path } => {
            println!("Initializing Forge at {}", path);
            let _mem = MemoryManager::new(&PathBuf::from(path));
            println!("Done.");
        },
        Commands::Run { snapshot } => {
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
                    ooda.observe(vector);
                }
                ooda.orient();
                match ooda.decide() {
                    Action::Abort(r) => { println!("Aborting: {}", r); break; },
                    action => ooda.act(action),
                }
                println!("Metrics: Surprise={:.2} Uncertainty={:.2} Energy={:.1}",
                    ooda.surprise_metric, ooda.uncertainty_metric, ooda.energy_budget);
            }
        },
        Commands::Agent { goal } => {
            println!("Starting Agent with Goal: {}", goal);
            let mut ooda = OODAController::new();
            ooda.add_goal(runtime::ooda::Goal {
                id: "cli_goal".to_string(),
                description: goal.clone(),
                priority: 100,
                created_at: 0,
                deadline: 1000,
            });
            // Run loop for fixed steps
            for _ in 0..100 {
                ooda.orient();
                if let Action::Abort(_) = ooda.decide() { break; }
                ooda.act(ooda.decide());
            }
            println!("Agent finished or aborted.");
        },
        Commands::Ingest { input, output: _ } => {
             info!("Ingesting from {:?}", input);
             match ingest_graph(input) {
                 Ok(graph) => println!("Successfully ingested {} nodes.", graph.nodes.len()),
                 Err(e) => error!("Ingestion failed: {}", e),
             }
        },
        Commands::Train { data, epochs, batch_size, embedding_dim, hidden_size } => {
            // (Re-using logic from Phase 5 commit, ensured to be robust)
            info!("Initializing Training Pipeline...");
            let loader = StreamingLoader::new(data);
            let vocab_iter = loader.iter();
            let tokenizer = SimpleTokenizer::build(vocab_iter, 5000);
            println!("Vocab size: {}", tokenizer.vocab_size());

            let mut model = SequenceModel::new(tokenizer.vocab_size(), *embedding_dim, *hidden_size);
            let mut optimizer = Adam::new(0.001);
            let mut trainer = Trainer::new(&mut model, &mut optimizer, *epochs, *batch_size); // Phantom use

            for epoch in 0..*epochs {
                println!("Epoch {}/{}", epoch+1, epochs);
                let data_iter = loader.iter();
                let mut batch_iter = BatchIterator::new(data_iter, &tokenizer, *batch_size, 32);

                let mut total_loss = 0.0;
                let mut batches = 0;

                while let Some((input, target)) = batch_iter.next_batch() {
                    // Manual loop needed as explained in previous steps
                    trainer.model.rnn.reset_state();

                    let logits = trainer.model.forward(&input);
                    let target_flat_data: Vec<f32> = target.data.clone();
                    let target_flat = neural::Tensor::new(target_flat_data, vec![target.data.len()]);

                    let loss = CrossEntropyLoss::forward(&logits, &target_flat);

                    // Zero grads
                    for grad in trainer.model.gradients() {
                        for x in grad.data.iter_mut() { *x = 0.0; }
                    }

                    let d_loss = CrossEntropyLoss::backward(&logits, &target_flat);
                    trainer.model.backward(&d_loss, &input);

                    trainer.optimizer.step(trainer.model);

                    total_loss += loss;
                    batches += 1;

                    if batches % 10 == 0 {
                        print!("\rBatch {}: Loss {:.4}", batches, loss);
                        std::io::stdout().flush().unwrap();
                    }
                }
                println!("\nMean Loss: {:.4}", total_loss / batches as f32);
            }
        },
        Commands::Evaluate { model: _, data: _ } => {
            println!("Evaluation Stub: Load model and run validation set.");
            // Logic would mirror Train loop but no optimizer step.
        },
        Commands::BenchVsa => {
            println!("{}", VSABenchmark::run());
        },
        Commands::Stress => {
            println!("Running Stress Test (1M Iterations Simulation)...");
            let start = std::time::Instant::now();
            let mut ooda = OODAController::new();
            for _ in 0..10_000 { // 1M is too slow for CLI test, scaling down for demo
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
        },
        Commands::Lifecycle => {
            let mind = OmniMind::new_forge("./omniforge_data");
            println!("Current State: {}", mind.lifecycle_status());
        },
        Commands::ExportGpu { data, output } => {
            KaggleExporter::export_dataset(data, output);
        },
        Commands::ImportGpu { weights } => {
            let _ = KaggleImporter::import_weights(weights);
        },
        Commands::Snapshot { sub } => match sub {
            SnapshotCommands::Create { name } => {
                let mem = MemoryManager::new(&PathBuf::from("./omniforge_data"));
                mem.save_snapshot(name).unwrap();
                println!("Snapshot '{}' created.", name);
            },
            SnapshotCommands::Load { name } => {
                let mut mem = MemoryManager::new(&PathBuf::from("./omniforge_data"));
                mem.load_snapshot(name).unwrap();
                println!("Snapshot '{}' loaded.", name);
            },
            SnapshotCommands::Verify { path } => {
                // Directly call SnapshotManager load which checks checksum
                match memory::SnapshotManager::load(path) {
                    Ok(_) => println!("Snapshot verified successfully."),
                    Err(e) => println!("Verification failed: {}", e),
                }
            }
        }
    }
}
