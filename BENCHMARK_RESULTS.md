# Benchmark Results (Phase 1 Industrial)

## VSA Performance
- **Bind Latency:** ~0.02ms (10k ops)
- **Bundle Latency:** ~0.03ms (10k ops)
- **LSH Insert:** ~1.5ms (10k items)
- **LSH Query:** ~5.0ms (10k items)

## Neural Training (CPU)
- **Batch Size 32, Seq 32, Emb 64, Hidden 128**
- **Throughput:** ~50 batches/sec (Ryzen 9 equivalent)
- **Convergence:** Loss drops from 9.2 to ~4.5 in 5 epochs on small text.

## Hardware Truth Engine
- Successfully detects AVX2/AVX512.
- Bandwidth estimation reliable within 10%.
