use crate::Tensor;

pub struct CrossEntropyLoss;

impl CrossEntropyLoss {
    /// Computes Softmax Cross Entropy Loss.
    /// Input: Logits [batch, classes]
    /// Target: Indices [batch] (as f32 for now, e.g. 5.0 -> index 5)
    pub fn forward(logits: &Tensor, target: &Tensor) -> f32 {
        let batch_size = logits.shape[0];
        let num_classes = logits.shape[1];

        let mut total_loss = 0.0;

        for i in 0..batch_size {
            // 1. Softmax for this row
            let row_start = i * num_classes;
            let row_end = row_start + num_classes;
            let row = &logits.data[row_start..row_end];

            let max_val = row.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
            let exp_sum: f32 = row.iter().map(|x| (x - max_val).exp()).sum();

            // 2. Select target class probability
            let target_idx = target.data[i] as usize;
            if target_idx < num_classes {
                let logit = row[target_idx];
                let log_prob = logit - max_val - exp_sum.ln();
                total_loss -= log_prob;
            }
        }

        total_loss / batch_size as f32
    }

    pub fn backward(logits: &Tensor, target: &Tensor) -> Tensor {
        let batch_size = logits.shape[0];
        let num_classes = logits.shape[1];
        let mut grad_data = vec![0.0; batch_size * num_classes];

        for i in 0..batch_size {
            let row_start = i * num_classes;
            let row_end = row_start + num_classes;
            let row = &logits.data[row_start..row_end];

            let max_val = row.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
            let exp_sum: f32 = row.iter().map(|x| (x - max_val).exp()).sum();

            let target_idx = target.data[i] as usize;

            for j in 0..num_classes {
                let softmax = (row[j] - max_val).exp() / exp_sum;
                if j == target_idx {
                    grad_data[row_start + j] = (softmax - 1.0) / batch_size as f32;
                } else {
                    grad_data[row_start + j] = softmax / batch_size as f32;
                }
            }
        }

        Tensor::new(grad_data, logits.shape.clone())
    }
}
