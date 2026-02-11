# Performance & Observability
**Metrics & Tuning**

Omni Forge provides runtime metrics for optimization.

## Key Metrics
The `RuntimeMind` tracks `RuntimeMetrics`:
- **Uptime**: Seconds since load.
- **Query Count**: Total interactions.
- **Learning Events**: Number of chunks ingested.
- **Memory Pressure**: Ratio of usage to capability limit.

## Optimization
- **Execution Policy**:
    - `LowPower`: Single-thread, lazy loading (Laptop/Phone).
    - `Balanced`: Default.
    - `HighPerformance`: Max concurrency (Server).
- **SIMD**: Auto-detected (AVX-512 > AVX2 > Neon > Scalar).

## Benchmarking
Run `cargo bench` to test:
- VSA operations (Bind/Bundle).
- Ingestion throughput.
- Inference latency.
