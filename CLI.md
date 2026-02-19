# CLI Reference

Omni Forge provides a comprehensive command-line interface for managing the AI lifecycle.

## Core Commands

### `init`
Initializes a new `forge_master.omf` in the system data directory.
```bash
omniforge init
```

### `ingest <path>`
Ingests data from a file or directory into the Forge Master.
- Recursively scans folders.
- Automatically selects adapters (Text, Code, Structure).
- Dedupes content via SHA256.
```bash
omniforge ingest ./my_corpus/
```

### `snapshot <version> [output]`
Creates an immutable snapshot of the current Forge Master.
- Freezes the knowledge graph.
- Generates integrity hashes.
```bash
omniforge snapshot 1.0 my_mind_v1.omf
```

### `clone <src> <dest>`
Duplicates a snapshot to create an independent instance.
- Cloning is just a file copy because snapshots are immutable.
- Each clone develops its own independent overlay.
```bash
omniforge clone my_mind_v1.omf my_mind_copy.omf
```

### `run <snapshot>`
Launches the interactive runtime environment.
- Loads the snapshot (Read-Only Base).
- Creates/Loads a local overlay (Read-Write Delta).
- **Interactive Commands:**
  - `/learn <text>`: Teach the local instance.
  - `/save`: Persist the local overlay.
  - `exit`: Quit.
```bash
omniforge run my_mind_v1.omf
```

### `verify <snapshot>`
Checks the integrity hash of a snapshot.
```bash
omniforge verify my_mind_v1.omf
```

### `inspect <snapshot>`
Displays metadata, compiler version, and source history.
```bash
omniforge inspect my_mind_v1.omf
```

### `status`
Shows hardware stats, memory usage, and OS details.
```bash
omniforge status
```
