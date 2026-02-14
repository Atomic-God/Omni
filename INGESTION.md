# Ingestion System (Universal)

## Supported Formats
- **Documents:** PDF, XLSX, CSV, Text
- **Media:** Images (Metadata + Color), Audio (Metadata)
- **Archives:** ZIP, TAR, GZ (Recursive)

## Features
- **Content Fingerprinting:** SHA256 hashes prevent duplicates.
- **Relational Graph:** Spreadsheets are mapped to Row-Column edges.
- **Multilingual:** Automatic language and script detection (via `whatlang`).
- **Safety:** 100MB file limit enforced.
