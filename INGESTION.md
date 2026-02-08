# Universal Ingestion System

Omni Forge implements a modality-agnostic ingestion pipeline capable of absorbing arbitrary data structures without manual labeling.

## Architecture

### Ingestion Adapters
The system uses a trait-based adapter pattern (`IngestionAdapter`) to handle diverse file formats:

1. **TextAdapter:** Handles `.txt`, `.md`, `.log`, `.yml`, `.toml`.
   - Streaming-safe line reading.
   - Unicode normalization.
   - Paragraph chunking.

2. **CodeAdapter:** Handles `.rs`, `.py`, `.c`, `.cpp`, `.h`, `.js`, `.ts`, `.java`, `.go`.
   - Extracts functions and classes using Regex-based heuristics (AST-lite).
   - Preserves structure metadata (`structure_type: "function:main"`).

3. **StructureAdapter:** Handles `.json`, `.csv`, `.xml`.
   - Flattens hierarchical data into semantic triples.
   - converts rows to entities.

4. **PdfAdapter (Placeholder):**
   - Prepared for `poppler` integration in Phase 2.
   - Currently logs a warning and skips to prevent heavy dependencies in the core.

### Pipeline Process
1. **Scanning:** Recursive `walkdir` traversal of input paths.
2. **Detection:** `whatlang` language detection and file type inference.
3. **Chunking:** Splits content into `SemanticChunk` units (max 1000 chars or logical boundary).
4. **Hashing:** SHA256 content hashing for deduplication.
5. **Metadata:** Attaches `timestamp`, `source`, `file_type`, and `language`.

## Streaming & Deduplication
- **Memory Safety:** Files > 10MB are truncated or streamed (future improvement).
- **Deduplication:** A `HashSet` of content hashes prevents re-learning identical chunks within a session.

## Usage
```bash
omniforge ingest ./my_data_folder
```
This command automatically selects the correct adapter for each file.
