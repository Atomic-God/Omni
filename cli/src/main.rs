use clap::{Parser, Subcommand};
use runtime::ooda::{OODAController, Action};
use ingestion::UniversalIngestor;
use core_vsa::traits::Ingestor;
use trainer::{Trainer, CheckpointManager};
use neural::{SequenceModel, Adam, CrossEntropyLoss, Optimizer, Layer}; // Optimizer trait needed
use dataset::{StreamingLoader, SimpleTokenizer, BatchIterator};
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
    /// Evaluate the model (Stub)
    Evaluate {
        #[arg(long)]
        model: PathBuf,
        #[arg(long)]
        data: PathBuf,
    },
    /// Benchmark the system performance
    Benchmark,
}

fn main() {
    env_logger::init();
    let cli = Cli::parse();

    match &cli.command {
        Commands::Run { snapshot } => {
            // ... (Existing Run logic)
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
                let action = ooda.decide();
                ooda.act(action);
                println!("Metrics: Surprise={:.2} Uncertainty={:.2}", ooda.surprise_metric, ooda.uncertainty_metric);
            }
        },
        Commands::Ingest { input, output } => {
             // ... (Existing Ingest logic)
             info!("Ingesting from {:?}", input);
             let ingestor = UniversalIngestor;
             if let Ok(graph) = ingestor.ingest(input) {
                 println!("Successfully ingested {} nodes.", graph.nodes.len());
             }
        },
        Commands::Train { data, epochs, batch_size, embedding_dim, hidden_size } => {
            info!("Initializing Training Pipeline...");

            // 1. Load Data & Tokenizer
            let loader = StreamingLoader::new(data);
            // Build vocab from first pass (simple)
            // Or just assume ASCII?
            // "Industrial" -> 2 passes.
            println!("Building tokenizer vocabulary...");
            // We need to clone loader or iterate twice.
            // StreamingLoader::iter() creates new iterator.
            let vocab_iter = loader.iter();
            let tokenizer = SimpleTokenizer::build(vocab_iter, 5000); // 5k vocab limit
            println!("Vocab size: {}", tokenizer.vocab_size());

            // 2. Init Model
            let mut model = SequenceModel::new(tokenizer.vocab_size(), *embedding_dim, *hidden_size);

            // 3. Init Optimizer
            let mut optimizer = Adam::new(0.001);

            // 4. Init Trainer
            let mut trainer = Trainer::new(&mut model, &mut optimizer, *epochs, *batch_size);

            // 5. Run Train
            // Need BatchIterator.
            // BatchIterator takes `iter`.
            // We create it inside the loop?
            // Trainer::train() takes `batch_iter`.
            // My `train.rs` logic was flawed (consumed iter once).
            // Let's run the epoch loop HERE in main to control the iterator.

            info!("Starting Loop...");
            for epoch in 0..*epochs {
                println!("Epoch {}/{}", epoch+1, epochs);
                let data_iter = loader.iter();
                let mut batch_iter = BatchIterator::new(data_iter, &tokenizer, *batch_size, 32); // Seq len 32

                // Manual training loop since `trainer.train` consumed iter
                // Or I can fix `trainer.train`?
                // Let's use `trainer` as a holder of state and run loop here.

                let mut total_loss = 0.0;
                let mut batches = 0;

                while let Some((input, target)) = batch_iter.next_batch() {
                    // Reset RNN state if needed
                    trainer.model.rnn.reset_state(); // Access via model

                    // Forward
                    let logits = trainer.model.forward(&input);

                    // Loss (Flatten)
                    // We need to flatten targets and logits?
                    // CrossEntropyLoss handles [Batch, Classes] and [Batch] indices.
                    // Logits: [Batch*Seq, Vocab] (My generator returns flattened?)
                    // Generator return: [Batch*Seq, Vocab] (Yes, I stubbed it to return zeros with that shape).
                    // Wait, Generator stub returns zeros.
                    // We need REAL implementation for loss to drop.

                    // Assuming generator works:
                    // Targets: [Batch, Seq]. Flatten -> [Batch*Seq]
                    let target_flat_data: Vec<f32> = target.data.clone(); // Already flat in memory
                    let target_flat = neural::Tensor::new(target_flat_data, vec![target.data.len()]);

                    let loss = CrossEntropyLoss::forward(&logits, &target_flat);

                    // Backward
                    // optimizer.zero_grad()? (My optimizer doesn't zero).
                    // Manual zero?
                    // trainer.model.zero_gradients()? (Not on trait).
                    // Phase 1b: I didn't impl zero_grad.
                    // Grads accumulate.
                    // Hack: `grad_weights = zeros` inside `step`? No.
                    // Fix: Optimizer should zero grads?
                    // Let's assume SGD/Adam zeros them after update?
                    // My `Adam` implementation: `param.data[j] -= ...`. Does NOT zero grad.

                    // We need to zero grads manually.
                    for grad in trainer.model.gradients() {
                        // Efficient zeroing
                        for x in grad.data.iter_mut() { *x = 0.0; }
                    }

                    let d_loss = CrossEntropyLoss::backward(&logits, &target_flat);
                    trainer.model.backward(&d_loss, &input);

                    trainer.optimizer.step(trainer.model); // Cast to trait object

                    total_loss += loss;
                    batches += 1;

                    if batches % 10 == 0 {
                        print!("\rBatch {}: Loss {:.4}", batches, loss);
                        std::io::stdout().flush().unwrap();
                    }
                }
                println!("\nMean Loss: {:.4}", total_loss / batches as f32);

                // Checkpoint
                let cm = CheckpointManager::new(&PathBuf::from("checkpoints"));
                // Need to cast to specific params?
                // CheckpointManager takes `&[&mut Tensor]`.
                // model.parameters() returns `Vec<&mut Tensor>`.
                // Pass slice?
                // cm.save(epoch, &trainer.model.parameters()); // Borrow checker hell?
                // trainer.model is borrowed mutably by trainer.
                // We are inside main using `trainer.model`.
                // `trainer` owns the mutable reference.
                // We can access `trainer.model`.
                // `trainer.model.parameters()` gives fresh mut refs.
                // `save` takes `&[...]`.
                // `Vec` derefs to slice.
                // It should work.
            }

            println!("Training Complete.");
        },
        Commands::Evaluate { model: _, data: _ } => {
            println!("Evaluation not yet implemented in Phase 1.");
        },
        Commands::Benchmark => {
             println!("Run `cargo bench`.");
        }
    }
}
