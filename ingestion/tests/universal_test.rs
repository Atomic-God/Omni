use ingestion::universal::UniversalAdapter;
use ingestion::IngestionAdapter;
use std::io::Write;

#[test]
fn test_universal_adapter_text() {
    let path = std::path::Path::new("test_univ_text.txt");
    std::fs::write(path, "Hello Universal World").unwrap();

    let adapter = UniversalAdapter;
    let chunks = adapter.ingest(path);

    assert!(!chunks.is_empty());
    assert_eq!(chunks[0].metadata.structure_type, "generic_text");
    assert!(chunks[0].content.contains("Hello Universal World"));

    std::fs::remove_file(path).unwrap();
}

#[test]
fn test_universal_adapter_binary() {
    let path = std::path::Path::new("test_univ_bin.bin");
    let mut data = vec![0u8; 100];
    data[0] = 0xCA; data[1] = 0xFE; // Magic
    // Embed a string
    let s = b"HIDDEN_SECRET";
    for (i, &b) in s.iter().enumerate() {
        data[10 + i] = b;
    }

    std::fs::write(path, &data).unwrap();

    let adapter = UniversalAdapter;
    let chunks = adapter.ingest(path);

    // Should have metadata chunk and strings chunk
    assert!(chunks.len() >= 1);

    let metadata_chunk = &chunks[0];
    assert!(metadata_chunk.content.contains("File Analysis"));
    assert!(metadata_chunk.content.contains("CA FE"));

    if chunks.len() > 1 {
        let strings_chunk = &chunks[1];
        assert!(strings_chunk.content.contains("HIDDEN_SECRET"));
    }

    std::fs::remove_file(path).unwrap();
}
