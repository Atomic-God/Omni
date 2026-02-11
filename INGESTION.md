# Universal Ingestion Pipeline
**Omni Forge Data Ingestion**

The ingestion pipeline converts raw files into `SemanticChunks` with preserved hierarchy and metadata.

## Supported Formats

| Format | Adapter | Notes |
| :--- | :--- | :--- |
| **Text** | `TextAdapter` | `.txt`, `.log`, `.yml` |
| **Code** | `CodeAdapter` | Regex-based structure (Function/Class) |
| **Markdown** | `MarkdownAdapter` | Heading hierarchy extraction |
| **JSON** | `JsonAdapter` | Key-value flattening to fields |
| **HTML** | `HtmlAdapter` | `html2text` conversion |
| **DOCX** | `DocxAdapter` | XML structure extraction |
| **CSV/TSV** | `SpreadsheetAdapter` | Text-based row parsing |
| **PDF** | `PdfAdapter` | Stub (Requires poppler) |
| **Images** | `ImageAdapter` | Stub (Requires OCR/GPU) |

## Universal Fallback
For any unknown or binary format (`.exe`, `.dat`, etc.), the `UniversalAdapter`:
1.  Attempts to read as UTF-8 text.
2.  If binary:
    - Extracts File Metadata (Size, Magic Bytes).
    - Calculates Entropy (to detect encryption).
    - Extracts "Strings" (contiguous printable characters).

This guarantees that **no file is ignored**—every input yields *some* semantic data.

## Registry System
Adapters are managed by `DataIngestionRegistry`.
- Auto-detection based on file extension.
- Fallback to `UniversalAdapter` for everything else.

## Hierarchy & Relations
Chunks are metadata-rich:
- `structure_type`: e.g., "markdown_section:Introduction", "function:main"
- `hash`: SHA-256 content deduplication.
