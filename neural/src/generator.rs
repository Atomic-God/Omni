use crate::{Tensor, Layer, RecurrentStateLayer, DenseLayer};
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
        let batch_size = input.shape[0];
        let dim = self.weights.shape[1];
        let mut out_data = vec![0.0; batch_size * dim];

        for i in 0..batch_size {
            let idx = input.data[i] as usize;
            if idx < self.weights.shape[0] {
                let start = idx * dim;
                for j in 0..dim {
                    out_data[i * dim + j] = self.weights.data[start + j];
                }
            }
        }

        Tensor::new(out_data, vec![batch_size, dim])
    }

    fn backward(&mut self, grad_output: &Tensor, input: &Tensor) -> Tensor {
        let batch_size = input.shape[0];
        let dim = self.weights.shape[1];

        for i in 0..batch_size {
            let idx = input.data[i] as usize;
            if idx < self.weights.shape[0] {
                let start = idx * dim;
                for j in 0..dim {
                    self.grad_weights.data[start + j] += grad_output.data[i * dim + j];
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

    fn params_and_grads(&mut self) -> (Vec<&mut Tensor>, Vec<&mut Tensor>) {
        (
            vec![&mut self.weights],
            vec![&mut self.grad_weights]
        )
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
        let emb = self.embedding.forward(input);
        let hidden = self.rnn.forward(&emb);
        self.head.forward(&hidden)
    }
}

impl Layer for SequenceModel {
    fn forward(&self, input: &Tensor) -> Tensor {
        let emb = self.embedding.forward(input);
        let hidden = self.rnn.forward(&emb);
        self.head.forward(&hidden)
    }

    fn backward(&mut self, grad_output: &Tensor, input: &Tensor) -> Tensor {
        let emb = self.embedding.forward(input);
        let hidden = self.rnn.forward(&emb);

        let d_hidden = self.head.backward(grad_output, &hidden);
        let d_emb = self.rnn.backward(&d_hidden, &emb);
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

    fn params_and_grads(&mut self) -> (Vec<&mut Tensor>, Vec<&mut Tensor>) {
        let (mut p1, mut g1) = self.embedding.params_and_grads();
        let (p2, g2) = self.rnn.params_and_grads();
        let (p3, g3) = self.head.params_and_grads();

        p1.extend(p2);
        p1.extend(p3);
        g1.extend(g2);
        g1.extend(g3);

        (p1, g1)
    }
}
