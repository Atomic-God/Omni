# Installing Omni Forge
**Version 8.9 (Product Release)**

Omni Forge is distributed as a single, portable binary.

## Quick Install (Linux/macOS)

1.  **Clone the Repository**:
    ```bash
    git clone https://github.com/omni-forge/core.git
    cd core
    ```

2.  **Run the Installer**:
    ```bash
    ./install.sh
    ```
    This will compile the project (requires Rust) and place the `omniforge` binary in the current directory.

## Manual Build

If you prefer to build manually:

```bash
cargo build --release -p omniforge
cp target/release/omniforge .
```

## System Requirements

*   **OS**: Linux, macOS, or Windows (via WSL/Powershell).
*   **Memory**: 4GB RAM recommended (2GB minimum for small minds).
*   **Disk**: 500MB+ for MindPacks.
*   **Dependencies**: None (statically linked).

## Verification

Run the following to verify your installation:

```bash
./omniforge doctor
```
This checks your CPU extensions (AVX/Neon) and memory health.
