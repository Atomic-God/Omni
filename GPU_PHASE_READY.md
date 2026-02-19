# GPU Phase Readiness

## 1. Export Strategy
The `gpu-bridge` crate provides `KaggleExporter`.
- Exports `SymbolGraph` to JSON/Parquet.
- Exports Training Dataset to text/JSONL.

## 2. Training (Kaggle)
- Upload exported data to Kaggle.
- Train Transformer/Diffusion models using Python/PyTorch.
- Export weights to `weights.bin`.

## 3. Import Strategy
- Use `omniforge import-gpu --weights weights.bin`.
- `NeuralMapper` adapts external embeddings to VSA HyperVectors.
