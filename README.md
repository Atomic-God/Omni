# Omni Forge

Omni Forge is a standalone, decentralized AI fabrication system based on Neuro-Symbolic Vector Symbolic Architectures (VSA).

## Structure

*   `core-vsa`: Mathematical engine for Hypervectors (bipolar, 10k dimensions).
*   `cognition`: Cognitive logic (BEAGLE learning, SVO parsing, inference).
*   `memory`: Persistence layer (JSON with versioning).
*   `engine`: High-level facade (`OmniMind`).
*   `cli`: Interactive terminal interface.
*   `hte`: Hardware Truth Engine for platform detection.

## Build & Run

1.  **Install Rust**:
    ```bash
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    ```

2.  **Build**:
    ```bash
    cargo build --release
    ```

3.  **Run CLI**:
    ```bash
    cargo run -p cli
    ```

4.  **Run Library Demo**:
    ```bash
    cargo run -p engine --example demo
    ```

## Development

*   **Test**: `cargo test`
*   **Format**: `cargo fmt`
*   **Lint**: `cargo clippy`
