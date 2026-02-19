# Usage Guide

Omni Forge is a command-line tool for fabricating and running AI Minds.

## Installation
Currently, compile from source:
```bash
cargo build --release
```

## Basic Workflow

### 1. Initialize the Forge
Create a new, empty Forge Master.
```bash
omniforge init
```

### 2. Ingest Data (Teach the Forge)
Feed the Forge with raw text files, code, or data.
```bash
omniforge ingest ./my_knowledge_base
```
The Forge will recursively scan the directory and learn continuously.

### 3. Create a Mind Snapshot (Birth)
Freeze the Forge's knowledge into a portable Mind Artifact.
```bash
omniforge snapshot 1.0 my_mind_v1.omf
```

### 4. Run the Mind (Life)
Start a runtime session.
```bash
omniforge run my_mind_v1.omf
```
- **Interactive Mode:** Type queries (`Is sky blue?`, `Explain gravity`).
- **Learning:** Use `/learn <text>` to teach the local instance.
- **Saving:** Use `/save` to persist local memories to `my_mind_v1.omf.local`.

### 5. Inspect a Mind
View metadata and structure.
```bash
omniforge inspect my_mind_v1.omf
```

### 6. Check System Status
View hardware capabilities and memory usage.
```bash
omniforge status
```

## Advanced Commands (Future)
- **Compress:** `omniforge compress` (Optimize for mobile).
- **Merge:** `omniforge merge` (Combine two minds).
