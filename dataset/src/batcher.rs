use crate::tokenizer::SimpleTokenizer;
use neural::Tensor;

pub struct BatchIterator<'a, I>
where I: Iterator<Item = String>
{
    iter: I,
    tokenizer: &'a SimpleTokenizer,
    batch_size: usize,
    seq_len: usize,
}

impl<'a, I> BatchIterator<'a, I>
where I: Iterator<Item = String>
{
    pub fn new(iter: I, tokenizer: &'a SimpleTokenizer, batch_size: usize, seq_len: usize) -> Self {
        Self {
            iter,
            tokenizer,
            batch_size,
            seq_len,
        }
    }

    pub fn next_batch(&mut self) -> Option<(Tensor, Tensor)> {
        let mut inputs = Vec::new();
        let mut targets = Vec::new();

        for _ in 0..self.batch_size {
            if let Some(text) = self.iter.next() {
                let ids = self.tokenizer.encode(&text);
                if ids.len() > self.seq_len {
                    for i in 0..self.seq_len {
                        inputs.push(ids[i] as f32);
                        targets.push(ids[i+1] as f32);
                    }
                } else {
                    // Padding or skip
                    continue;
                }
            } else {
                break;
            }
        }

        if inputs.is_empty() {
            return None;
        }

        let actual_batch = inputs.len() / self.seq_len;
        Some((
            Tensor::new(inputs, vec![actual_batch, self.seq_len]),
            Tensor::new(targets, vec![actual_batch, self.seq_len]),
        ))
    }
}
