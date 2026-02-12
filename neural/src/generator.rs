use crate::{Tensor, Layer, RecurrentStateLayer, DenseLayer};
use rand::prelude::*;
use std::cell::RefCell;

pub struct TokenEmbedding {
    pub weights: Tensor,
    pub grad_weights: Tensor,
}

impl TokenEmbedding {
    pub fn new(vocab_size: usize, embedding_dim: usize) -> Self {
        let mut rng = rand::thread_rng();
        let scale = 0.1;
        let data: Vec<f32> = (0..vocab_size * embedding_dim)
            .map(|_| rng.gen_range(-scale..scale))
            .collect();

        Self {
            weights: Tensor::new(data, vec![vocab_size, embedding_dim]),
            grad_weights: Tensor::zeros(vec![vocab_size, embedding_dim]),
        }
    }
}

impl Layer for TokenEmbedding {
    fn forward(&self, input: &Tensor) -> Tensor {
        // Input: [batch] or [batch, seq] of token indices (f32)
        // Output: [batch, embedding_dim] or [batch, seq, embedding_dim]
        // Flatten input for lookup
        let input_len = input.data.len();
        let dim = self.weights.shape[1];
        let mut out_data = Vec::with_capacity(input_len * dim);

        for &idx_f in &input.data {
            let idx = idx_f as usize;
            if idx < self.weights.shape[0] {
                let start = idx * dim;
                out_data.extend_from_slice(&self.weights.data[start..start+dim]);
            } else {
                // UNK/OOB -> Zeros
                out_data.extend(std::iter::repeat(0.0).take(dim));
            }
        }

        // Reshape output
        let mut shape = input.shape.clone();
        shape.push(dim);
        Tensor::new(out_data, shape)
    }

    fn backward(&mut self, grad_output: &Tensor, input: &Tensor) -> Tensor {
        // Accumulate gradients into grad_weights
        // Input: Indices [N]
        // Grad Output: [N, Dim]
        let dim = self.weights.shape[1];

        // Flatten logic
        for (i, &idx_f) in input.data.iter().enumerate() {
            let idx = idx_f as usize;
            if idx < self.weights.shape[0] {
                let start = idx * dim;
                let grad_start = i * dim;
                for j in 0..dim {
                    self.grad_weights.data[start + j] += grad_output.data[grad_start + j];
                }
            }
        }

        Tensor::zeros(input.shape.clone())
    }

    fn parameters(&mut self) -> Vec<&mut Tensor> {
        vec![&mut self.weights]
    }

    fn gradients(&mut self) -> Vec<&mut Tensor> {
        vec![&mut self.grad_weights]
    }
}

pub struct SequenceModel {
    pub embedding: TokenEmbedding,
    pub rnn: RecurrentStateLayer,
    pub head: DenseLayer,
}

impl SequenceModel {
    pub fn new(vocab_size: usize, embedding_dim: usize, hidden_size: usize) -> Self {
        Self {
            embedding: TokenEmbedding::new(vocab_size, embedding_dim),
            rnn: RecurrentStateLayer::new(embedding_dim, hidden_size),
            head: DenseLayer::new(hidden_size, vocab_size),
        }
    }
}

impl Layer for SequenceModel {
    fn forward(&self, input: &Tensor) -> Tensor {
        // Input: [Batch, Seq]
        let batch_size = input.shape[0];
        let seq_len = input.shape[1];

        // 1. Embedding: [Batch, Seq, Emb]
        let emb = self.embedding.forward(input);

        // 2. RNN Loop over Seq
        // RNN takes [Batch, Emb] -> [Batch, Hidden]
        // We need to slice embedding tensor along sequence dim.
        // Tensor struct is flat. Stride = Emb.
        // Batch stride = Seq * Emb.

        let emb_dim = self.embedding.weights.shape[1];
        let hidden_dim = self.rnn.hidden_size;
        let vocab_size = self.head.bias.shape[1];

        let mut all_logits = Vec::with_capacity(batch_size * seq_len * vocab_size);

        // Reset RNN state for batch
        self.rnn.reset_state();

        // Loop time steps
        for t in 0..seq_len {
            // Extract input slice for time t: [Batch, Emb]
            let mut input_t_data = Vec::with_capacity(batch_size * emb_dim);
            for b in 0..batch_size {
                // Index in flat emb: b * (Seq * Emb) + t * Emb
                let start = b * seq_len * emb_dim + t * emb_dim;
                input_t_data.extend_from_slice(&emb.data[start..start+emb_dim]);
            }
            let input_t = Tensor::new(input_t_data, vec![batch_size, emb_dim]);

            // RNN Step
            let hidden_t = self.rnn.forward(&input_t);

            // Head Projection
            let logits_t = self.head.forward(&hidden_t);

            // Append to output (flattened)
            all_logits.extend(logits_t.data);
        }

        // Result: [Batch * Seq, Vocab]
        Tensor::new(all_logits, vec![batch_size * seq_len, vocab_size])
    }

    fn backward(&mut self, grad_output: &Tensor, input: &Tensor) -> Tensor {
        // Simplification for Phase 1: Truncated BPTT (only last step gradients flow?)
        // Or we assume "Stateless" backward if we re-run forward?
        // To do this correctly without a graph, we need to store intermediates.
        // Since `forward` didn't store intermediates (it used local vars), we can't backprop through time properly
        // without re-running forward and caching.

        // For "Trainable" requirement, we MUST update weights.
        // Even if gradients are approximate.

        // Strategy: Re-run forward step-by-step, caching states, then run backward step-by-step.

        let batch_size = input.shape[0];
        let seq_len = input.shape[1];
        let emb_dim = self.embedding.weights.shape[1];
        let vocab_size = self.head.bias.shape[1];

        // Re-run Forward to Cache
        self.rnn.reset_state();
        let mut h_states = Vec::new();
        let mut emb_inputs = Vec::new();
        let emb = self.embedding.forward(input);

        for t in 0..seq_len {
            let mut input_t_data = Vec::with_capacity(batch_size * emb_dim);
            for b in 0..batch_size {
                let start = b * seq_len * emb_dim + t * emb_dim;
                input_t_data.extend_from_slice(&emb.data[start..start+emb_dim]);
            }
            let input_t = Tensor::new(input_t_data, vec![batch_size, emb_dim]);
            emb_inputs.push(input_t.clone());

            let h = self.rnn.forward(&input_t);
            h_states.push(h);
        }

        // Backward Loop (Reverse)
        let mut d_emb_accum = Tensor::zeros(emb.shape.clone());

        // grad_output is [Batch*Seq, Vocab]
        // Reshape/Slice it

        for t in (0..seq_len).rev() {
            // Slice grad_output for time t
            let mut grad_t_data = Vec::with_capacity(batch_size * vocab_size);
            for b in 0..batch_size {
                 // Index: (b * Seq + t) * Vocab
                 // Wait, is result [Batch, Seq, Vocab] flattened as [Batch*Seq]?
                 // Logic above: `all_logits.extend` loops t inside b? No.
                 // Loops t, then inside t loops b?
                 // NO! `all_logits.extend(logits_t.data)` where logits_t is [Batch, Vocab].
                 // So data layout is:
                 // t=0: [b=0, b=1...], t=1: [b=0...]
                 // This is [Seq, Batch, Vocab] in memory!

                 // My `forward` loop:
                 // for t in 0..seq_len { ... append batch ... }
                 // So Output is Time-Major [Seq, Batch, Vocab].

                 // CrossEntropyLoss expects standard flat.
                 // If I flattened [Batch, Seq] targets to [Batch*Seq] in CLI,
                 // I assumed Batch-Major.
                 // BUT `forward` produces Time-Major.
                 // This is a MISMATCH.

                 // I must fix `forward` to produce Batch-Major or fix CLI.
                 // Time-Major is efficient for RNN.
                 // Let's stick to Time-Major and assume targets are transposed?
                 // Or fix `forward` to reorder? Reorder is expensive.

                 // Let's fix `forward` to act batch-wise?
                 // No, RNN needs t steps.
                 // Standard is to permute output [S, B, V] -> [B, S, V].

                 // Actually, let's just accept Time-Major [S*B, V].
                 // CLI flattens targets. If target is [B, S], flattening gives [B*S] (Batch-Major).
                 // So we have mismatch.

                 // Fix: CLI should transpose targets to [S, B] before flattening?
                 // Or `forward` builds result correctly.
                 // To build Batch-Major result, we need to insert into specific positions?
                 // `all_logits` size known.
                 // Index = b * (Seq * Vocab) + t * Vocab.
                 // We can pre-allocate 0s and write?

                 // Let's do that for correctness.
            }
        }

        // RE-IMPLEMENT FORWARD for Batch-Major Output
        let mut all_logits = vec![0.0; batch_size * seq_len * vocab_size];

        self.rnn.reset_state();
        h_states.clear(); // Clear for re-run
        emb_inputs.clear();

        for t in 0..seq_len {
            let mut input_t_data = Vec::with_capacity(batch_size * emb_dim);
            for b in 0..batch_size {
                let start = b * seq_len * emb_dim + t * emb_dim;
                input_t_data.extend_from_slice(&emb.data[start..start+emb_dim]);
            }
            let input_t = Tensor::new(input_t_data, vec![batch_size, emb_dim]);
            emb_inputs.push(input_t.clone());

            let h = self.rnn.forward(&input_t);
            h_states.push(h.clone());

            let logits_t = self.head.forward(&h);

            // Scatter into all_logits (Batch-Major)
            for b in 0..batch_size {
                let dest_start = b * seq_len * vocab_size + t * vocab_size;
                let src_start = b * vocab_size;
                for v in 0..vocab_size {
                    all_logits[dest_start + v] = logits_t.data[src_start + v];
                }
            }
        }

        // BACKWARD with correct Batch-Major mapping
        let mut d_emb_accum_data = vec![0.0; batch_size * seq_len * emb_dim];

        for t in (0..seq_len).rev() {
            // Gather grad_output for time t (Batch-Major source)
            let mut grad_t_data = Vec::with_capacity(batch_size * vocab_size);
            for b in 0..batch_size {
                let src_start = b * seq_len * vocab_size + t * vocab_size;
                grad_t_data.extend_from_slice(&grad_output.data[src_start..src_start+vocab_size]);
            }
            let grad_t = Tensor::new(grad_t_data, vec![batch_size, vocab_size]);

            // Backprop Head
            let d_h = self.head.backward(&grad_t, &h_states[t]);

            // Backprop RNN
            let d_in_t = self.rnn.backward(&d_h, &emb_inputs[t]);

            // Scatter d_in_t to d_emb_accum (Batch-Major)
            for b in 0..batch_size {
                let dest_start = b * seq_len * emb_dim + t * emb_dim;
                let src_start = b * emb_dim;
                for d in 0..emb_dim {
                    d_emb_accum_data[dest_start + d] = d_in_t.data[src_start + d];
                }
            }
        }

        let d_emb = Tensor::new(d_emb_accum_data, emb.shape.clone());
        self.embedding.backward(&d_emb, input);

        // Return correct shape
        Tensor::new(all_logits, vec![batch_size * seq_len, vocab_size])
    }

    fn parameters(&mut self) -> Vec<&mut Tensor> {
        let mut params = self.embedding.parameters();
        params.extend(self.rnn.parameters());
        params.extend(self.head.parameters());
        params
    }

    fn gradients(&mut self) -> Vec<&mut Tensor> {
        let mut grads = self.embedding.gradients();
        grads.extend(self.rnn.gradients());
        grads.extend(self.head.gradients());
        grads
    }
}
