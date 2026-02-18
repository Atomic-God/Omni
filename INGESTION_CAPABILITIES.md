# OmniForge Ingestion Capabilities

The `ingestion` crate provides a unified pipeline for extracting symbolic meaning from heterogeneous data sources.

## Supported Formats
- **Text**: TXT, Markdown (section-aware).
- **Documents**: PDF (metadata + layout), DOCX (text extraction), XML, RTF.
- **Structured Data**: CSV, XLSX, JSON, YAML.
- **Code**: Automatic fact extraction from Rust, Python, C++, and more.
- **Media**:
    - **Vision**: Image analysis for structural entropy, symmetry, and symbolic OCR.
    - **Audio**: Acoustic signature extraction (High-Fi vs Compressed).
    - **Video**: Scene transition detection and frame sampling.
- **Archives**: ZIP/Tar extraction for batch ingestion.

## Deep Meaning Extraction
Unlike simple scrapers, OmniForge uses `SymbolicNLP` to extract Subject-Verb-Object (SVO) triples from text and `CodeAnalyzer` to map function dependencies. These are stored directly in the Knowledge Graph for reasoning.

## Industrial Features
- **Duplicate detection**: SHA-256 content hashing prevents redundant ingestion.
- **Progress Reporting**: Atomic counters provide real-time status for batch operations.
- **Language Detection**: Automatically detects and normalizes Unicode content.
- **Security**: Validates ingestion paths to prevent traversal attacks.
