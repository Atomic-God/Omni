use neural::{Tensor, CrossEntropyLoss};

pub struct Evaluator;

impl Evaluator {
    /// Computes Perplexity: exp(CrossEntropy)
    pub fn calculate_perplexity(loss: f32) -> f32 {
        loss.exp()
    }

    // Stub for accuracy
    pub fn calculate_accuracy(logits: &Tensor, target: &Tensor) -> f32 {
        // Logits: [Batch, Classes]
        // Target: [Batch]
        let batch_size = logits.shape[0];
        let num_classes = logits.shape[1];
        let mut correct = 0;

        for i in 0..batch_size {
            let row_start = i * num_classes;
            let row = &logits.data[row_start..row_start+num_classes];

            // Argmax
            let mut max_val = f32::NEG_INFINITY;
            let mut max_idx = 0;
            for (j, &val) in row.iter().enumerate() {
                if val > max_val {
                    max_val = val;
                    max_idx = j;
                }
            }

            if max_idx == target.data[i] as usize {
                correct += 1;
            }
        }

        correct as f32 / batch_size as f32
    }
}
