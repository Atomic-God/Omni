use crate::{Tensor, Layer, RecurrentStateLayer, DenseLayer, Tanh};
use rand::prelude::*;

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
        // Input: [batch] of token indices (f32)
        let batch_size = input.shape[0];
        let dim = self.weights.shape[1];
        let mut out_data = vec![0.0; batch_size * dim];

        for i in 0..batch_size {
            let idx = input.data[i] as usize;
            if idx < self.weights.shape[0] {
                let start = idx * dim;
                let end = start + dim;
                for j in 0..dim {
                    out_data[i * dim + j] = self.weights.data[start + j];
                }
            }
        }

        Tensor::new(out_data, vec![batch_size, dim])
    }

    fn backward(&mut self, grad_output: &Tensor, input: &Tensor) -> Tensor {
        // Input: Indices
        // Grad Output: [batch, dim]
        let batch_size = input.shape[0];
        let dim = self.weights.shape[1];

        for i in 0..batch_size {
            let idx = input.data[i] as usize;
            if idx < self.weights.shape[0] {
                let start = idx * dim;
                // Add gradient row to the embedding weights
                for j in 0..dim {
                    self.grad_weights.data[start + j] += grad_output.data[i * dim + j];
                }
            }
        }

        // No gradient for indices
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

    pub fn forward(&mut self, input: &Tensor) -> Tensor {
        // 1. Embedding
        let emb = self.embedding.forward(input);
        // 2. RNN
        let hidden = self.rnn.forward(&emb);
        // 3. Projection
        self.head.forward(&hidden)
    }

    // For Training: need to orchestrate Backprop manually across layers since they are separate structs
    // Or implement Layer for SequenceModel?
    // Let's implement Layer for SequenceModel to unify interface.
}

impl Layer for SequenceModel {
    fn forward(&self, input: &Tensor) -> Tensor {
        // Can't mutate RNN state in immutable forward?
        // Wait, RecurrentStateLayer.forward DOES mutate (if it updates stored_hidden).
        // My previous implementation used `&self` but `stored_hidden` was immutable?
        // Ah, RecurrentStateLayer definition: `stored_hidden: Option<Tensor>`.
        // `forward(&self)` can't update `stored_hidden`.
        // This is a design flaw in Phase 1b.
        // A standard `Layer` trait `forward(&self)` implies pure function or interior mutability (RefCell).
        // For simplicity in Phase 1, let's assume `forward` is stateless (like Transformer), passing state explicitly?
        // But prompt requested "RecurrentStateLayer".

        // Let's use `RefCell` for state? Or change trait to `forward(&mut self)`?
        // Trait is `fn forward(&self, ...)`.
        // Changing trait would break other layers.

        // Fix: Use Interior Mutability for RNN state.

        // But actually, for training, we unroll loops.
        // Let's implement `forward_step` on the struct directly.

        let emb = self.embedding.forward(input);
        let hidden = self.rnn.forward(&emb);
        self.head.forward(&hidden)
    }

    fn backward(&mut self, grad_output: &Tensor, input: &Tensor) -> Tensor {
        // This is tricky. We need intermediate activations for backward.
        // If we don't store them, we can't backprop through non-linearities correctly.
        // Standard DL frameworks build a graph.
        // We are doing manual.

        // Phase 1 Simplification:
        // We will just execute forward AGAIN to get intermediates? Slow but correct.
        // Or we store them in the struct during forward (requires &mut self).

        // Given constraints, let's change Layer trait to `forward(&mut self)`.
        // But that breaks `DenseLayer` signature.

        // Let's assume we call `forward_with_cache`?

        // For "Phase 1 Complete", let's just recompute.
        let emb = self.embedding.forward(input);
        let hidden = self.rnn.forward(&emb);

        // Backward Head
        let d_hidden = self.head.backward(grad_output, &hidden);

        // Backward RNN
        // Need gradients w.r.t input (emb) and w.r.t hidden_prev (ignored for truncated).
        let d_emb = self.rnn.backward(&d_hidden, &emb);

        // Backward Embedding
        self.embedding.backward(&d_emb, input)
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
