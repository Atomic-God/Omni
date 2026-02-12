use neural::Tensor;
use crate::tokenizer::SimpleTokenizer;
use rand::prelude::*;

pub struct BatchIterator<'a> {
    data: Vec<String>, // Phase 1: We load dataset into RAM for shuffling (violates strict rule? "Must support large datasets without loading entire file into RAM")
    // If we want shuffling without full RAM, we need a buffer or memory mapping.
    // For "Phase 1 Complete", let's assume we load *chunks*?
    // StreamingLoader supports iteration.
    // We can reservoir sample?
    // Or shuffle indices?

    // To strictly follow "No load entire file", we should:
    // 1. Iterate StreamingLoader
    // 2. Fill a buffer (e.g. 10k lines)
    // 3. Shuffle buffer
    // 4. Yield batches
    // 5. Repeat

    // Let's implement Buffered Shuffling.

    loader: Box<dyn Iterator<Item = String> + 'a>, // We need to own the iterator
    tokenizer: &'a SimpleTokenizer,
    batch_size: usize,
    seq_len: usize,
    buffer: Vec<String>,
    buffer_size: usize,
}

impl<'a> BatchIterator<'a> {
    pub fn new(
        loader: impl Iterator<Item = String> + 'a,
        tokenizer: &'a SimpleTokenizer,
        batch_size: usize,
        seq_len: usize,
    ) -> Self {
        Self {
            loader: Box::new(loader),
            tokenizer,
            batch_size,
            seq_len,
            buffer: Vec::new(),
            buffer_size: 10000,
        }
    }

    // Yields (Input, Target) tensors
    pub fn next_batch(&mut self) -> Option<(Tensor, Tensor)> {
        // Refill buffer if needed
        while self.buffer.len() < self.batch_size * self.seq_len { // Heuristic: need enough for at least 1 batch?
             // Actually, we need batch_size lines? Or lines are split into tokens?
             // Simplest: 1 line = 1 sample. Pad/Truncate.
             if let Some(line) = self.loader.next() {
                 self.buffer.push(line);
             } else {
                 break; // EOF
             }
        }

        if self.buffer.is_empty() {
            return None;
        }

        // Shuffle buffer
        let mut rng = rand::thread_rng();
        self.buffer.shuffle(&mut rng);

        // Take batch_size items
        let count = self.batch_size.min(self.buffer.len());
        let batch_lines: Vec<String> = self.buffer.drain(0..count).collect();

        // Tokenize & Pad
        let mut inputs = Vec::new();
        let mut targets = Vec::new();

        for line in batch_lines {
            let tokens = self.tokenizer.encode(&line);
            if tokens.len() <= 1 { continue; } // Skip too short

            // Truncate
            let len = tokens.len().min(self.seq_len + 1);
            let slice = &tokens[0..len];

            // Input: 0..N-1
            // Target: 1..N
            let inp = &slice[0..len-1];
            let tgt = &slice[1..len];

            // Pad
            let mut inp_vec = inp.to_vec();
            let mut tgt_vec = tgt.to_vec();

            while inp_vec.len() < self.seq_len { inp_vec.push(0); }
            while tgt_vec.len() < self.seq_len { tgt_vec.push(0); } // 0 is PAD

            // Convert to f32 for Tensor
            inputs.extend(inp_vec.into_iter().map(|x| x as f32));
            targets.extend(tgt_vec.into_iter().map(|x| x as f32));
        }

        // Check actual batch size (filtering short lines might reduce it)
        let actual_batch = inputs.len() / self.seq_len;
        if actual_batch == 0 { return None; }

        let input_tensor = Tensor::new(inputs, vec![actual_batch, self.seq_len]);
        let target_tensor = Tensor::new(targets, vec![actual_batch, self.seq_len]); // Flattened?
        // CrossEntropyLoss expects [batch, indices] or [batch*seq, indices]?
        // Usually Sequence loss is sum over T.
        // We will flatten time into batch for simple loss?
        // Or handle 3D tensors?
        // Phase 1 Tensor is arbitrary dim?
        // Tensor struct supports Vec shape.
        // But MatMul is 2D only.
        // RNN expects [batch, dim].

        // Sequence Model loop:
        // for t in 0..seq_len:
        //    input_t = input[:, t]
        //    out, h = rnn(input_t, h)
        //    loss += cross_entropy(out, target[:, t])

        // So we return [batch, seq_len].

        Some((input_tensor, target_tensor))
    }
}
