# Omni Forge v5.1

Omni Forge is a standalone, decentralized AI fabrication system based on Neuro-Symbolic Vector Symbolic Architectures (VSA).

## Architecture

*   `core-vsa`: Mathematical engine for Hypervectors (bipolar, 10k dimensions) and Indexing.
*   `cognition`: Reasoning Core (BEAGLE-style learning, Graph reasoning, Inference).
*   `perception`: Encoding (Text -> HV) and Decoding (HV -> Text).
*   `memory`: Persistence layer (Zip-compressed MindPack).
*   `engine`: OmniMind Fabricator (Facade for all components).
*   `cli`: Interactive Runtime Shell.
*   `ingest`: Knowledge ingestion system.
*   `api`: REST API service.

## Fabrication-Runtime Duality

Omni Forge operates in two modes:
1.  **Fabrication**: Learning from text/files (`train`, `ingest`).
2.  **Runtime**: Reasoning and answering (`ask`, `infer`).

## Usage

### CLI

```bash
cargo run -p cli
```

Commands:
- `train <text>`: Learn from text.
- `ask <query>`: Ask a question (e.g., "What does dog eat?").
- `save-mind`: Save state to `mind.omf`.
- `load-mind`: Load state from `mind.omf`.

### API

```bash
# Build Docker image
docker build -t omni-forge .
# Run
docker run -p 3000:3000 omni-forge
```

## Development

*   **Test**: `cargo test`
*   **Benchmark**: `cargo bench`
