# Limitations (v9.0)

While Omni Forge is "Production Ready" as a CPU-based system, it has explicit limits:

1.  **Context Scaling**:
    - Currently limited by RAM. Paged memory is architected but not fully implemented.
    - Large minds (>4GB) may be slow on load.

2.  **Perception**:
    - Visual/Audio encoders are *stubs* or basic implementations.
    - No real-time video processing (requires GPU).

3.  **Language**:
    - No Transformer-based fluency. Output is structured/symbolic, not poetic.
    - Grammar induction is heuristic-based.

4.  **Tools**:
    - Tool interface is architectural. Real OS commands are disabled for safety.

5.  **Performance**:
    - `find_path` (inference) depth is limited (3 hops) to prevent CPU stall.
