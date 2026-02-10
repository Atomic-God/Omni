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

## Registry System
Adapters are managed by `DataIngestionRegistry`.
- Auto-detection based on file extension.
- Fallback to `SymbolExtractor` (hex tokens) for unknown binaries.

## Hierarchy & Relations
Chunks are metadata-rich:
- `structure_type`: e.g., "markdown_section:Introduction", "function:main"
- `hash`: SHA-256 content deduplication.
