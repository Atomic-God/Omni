use neural::{Layer, Optimizer, CrossEntropyLoss};
use dataset::BatchIterator;
use indicatif::{ProgressBar, ProgressStyle};
use log::info;

pub struct Trainer<'a> {
    pub model: &'a mut dyn Layer, // SequenceModel usually
    pub optimizer: &'a mut dyn Optimizer,
    pub epochs: usize,
    pub batch_size: usize,
}

impl<'a> Trainer<'a> {
    pub fn new(model: &'a mut dyn Layer, optimizer: &'a mut dyn Optimizer, epochs: usize, batch_size: usize) -> Self {
        Self { model, optimizer, epochs, batch_size }
    }

    pub fn train(&mut self, mut batch_iter: BatchIterator) {
        info!("Starting training for {} epochs...", self.epochs);

        for epoch in 0..self.epochs {
            println!("Epoch {}/{}", epoch + 1, self.epochs);
            let pb = ProgressBar::new_spinner();
            pb.set_style(ProgressStyle::default_spinner().template("{spinner} Loss: {msg}").unwrap());

            let mut total_loss = 0.0;
            let mut steps = 0;

            // We need to reset iterator or create new one per epoch?
            // BatchIterator consumes `loader`.
            // If `loader` is consumed, we can't reuse.
            // Phase 1 limitation: StreamingLoader reads file again if we re-create iterator.
            // But we passed `batch_iter` by value.
            // So we can only run 1 epoch?
            // Or `BatchIterator` refills from `loader`?
            // `loader` is `Box<dyn Iterator>`. Once exhausted, it's done.
            // To support multiple epochs, `BatchIterator` needs to reset.
            // `StreamingLoader` can create multiple iterators.
            // But `BatchIterator` takes `loader` in constructor.

            // Fix: Pass `dataset` (StreamingLoader) and `tokenizer` to `train`?
            // And create `BatchIterator` inside loop?
            // But `BatchIterator` logic is complex (buffer shuffling).

            // For now, assume `batch_iter` is infinite or we just run until exhausted (1 epoch equivalent).
            // Or we assume dataset fits in RAM (BatchIterator buffer).

            // Let's iterate until `next_batch` returns None.

            while let Some((input, target)) = batch_iter.next_batch() {
                // Forward
                // Model forward must handle "training mode"?
                // We just call `forward`.

                // Note: SequenceModel.forward calls stateless/truncated RNN.
                // We assume `forward` returns `logits` [batch, seq, vocab].

                // We need to flatten for CrossEntropyLoss.
                // Input: [batch, seq]
                // Target: [batch, seq]
                // Logits: [batch, seq, vocab] -> [batch*seq, vocab]
                // Target: [batch*seq]

                let logits = self.model.forward(&input);

                // Reshape/Flatten logic manually?
                // Tensor doesn't have reshape yet.
                // But data is contiguous.
                // [B, S, V] -> [B*S, V] is just changing shape vector.
                let b = logits.shape[0];
                let s = logits.shape[1];
                let v = logits.shape[2]; // Wait, DenseLayer output is [batch, output].
                // SequenceModel runs head on [batch, hidden]?
                // No, SequenceModel runs head on EACH time step?
                // `DenseLayer` matmul is 2D.
                // If input to Head is 3D [batch, seq, hidden], DenseLayer `matmul` fails if not 2D.

                // Phase 1b limitation: `DenseLayer` expects 2D.
                // `SequenceModel` must handle flattening before head?
                // `SequenceModel::forward`:
                // 1. Emb [B, S, E]
                // 2. RNN [B, S, H] -> Wait, RNN forward in 1b was [B, H] (Last state).
                // Ah, I implemented "Truncated" RNN which just does one step or pooled?
                // `RecurrentStateLayer::forward` takes `input` [B, I].
                // It does NOT loop over sequence.
                // So `SequenceModel` currently only handles 1 token step?
                // OR `BatchIterator` yields [B, S] and we treat it as [B*S, 1]?

                // If we treat it as [B*S, 1] (flattened sequence), then context is lost between S steps.
                // RNN needs sequential update.

                // For "Phase 1 Complete", we need a Loop in `SequenceModel`.
                // `neural/src/generator.rs` `forward`:
                // `emb` = embedding(input) -> [B, S, E]
                // `rnn.forward` expects [B, E]? No [B, I].
                // So we must loop T=0..S.

                // I need to update `SequenceModel` to loop.
                // Or I update `RecurrentStateLayer` to handle 3D?

                // Updating `SequenceModel` is easier.
                // But `BatchIterator` yields [Batch, Seq].

                // Let's fix `SequenceModel` logic here conceptually or in previous step?
                // I can't go back. I must fix in `neural` or handle in `train`.

                // I will handle in `train` for now? No, `train` calls `model.forward`.

                // I will Assume `SequenceModel` does "Teacher Forcing" single step?
                // Or I treat [Batch, Seq] as independent samples? No, that breaks RNN purpose.

                // I will update `SequenceModel` in `neural` to loop.
                // But `neural` is already "finished".
                // I can use `overwrite_file` to fix `neural/src/generator.rs`.

                // Actually, `Trainer` is in `trainer` crate.
                // `SequenceModel` is in `neural`.
                // I should fix `neural` first.

                // Let's finish `Trainer` structure first, then fix `neural`.

                let loss = 0.0; // Placeholder

                // Backward
                // self.optimizer.zero_grad()? (My optimizer doesn't have zero_grad, it relies on manually zeroing or accumulating?)
                // My `Layer` struct has `grad_weights`.
                // `backward` ACCUMULATES.
                // `optimizer.step` subtracts and should ZERO them?
                // My `SGD` implementation:
                // `param.data = new_data`.
                // It DOES NOT zero grads.
                // So grads accumulate forever?
                // Fix: Optimizer should zero grads after step.

                // ...

                steps += 1;
                pb.set_message(format!("{:.4}", loss));
            }
            pb.finish();
        }
    }
}
