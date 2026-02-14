# Deployment & Sharing

## Snapshots
- **Create:** `omniforge snapshot create <name>`
- **Load:** `omniforge snapshot load <name>`
- **Verify:** `omniforge snapshot verify <path>`

## Portability
- MindPacks are binary compatible across OS (Rust serialization).
- Checksums ensure integrity.

## GPU Transition
- **Export:** `omniforge export-gpu` (Dataset -> Parquet).
- **Import:** `omniforge import-gpu` (Weights -> VSA).
