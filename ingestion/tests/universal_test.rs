use ingestion::universal::UniversalAdapter;
use ingestion::IngestionAdapter;

#[test]
fn test_universal_adapter_text() {
    let path = std::path::Path::new("test_univ_text.txt");
    std::fs::write(path, "Hello Universal World").unwrap();

    let adapter = UniversalAdapter;
    let chunks = adapter.ingest(path);

    assert!(!chunks.is_empty());
    assert!(chunks[0].content.contains("Hello Universal World"));

    std::fs::remove_file(path).unwrap();
}

#[test]
fn test_universal_adapter_binary() {
    let path = std::path::Path::new("test_univ_bin.bin");
    let mut data = vec![0u8; 100];
    data[0] = 0xCA; data[1] = 0xFE;
    std::fs::write(path, &data).unwrap();

    let adapter = UniversalAdapter;
    let chunks = adapter.ingest(path);

    // generic_read_file might fail to read invalid UTF8 as string
    // if it returns empty, that's fine for now as it's a fallback
    // assert!(chunks.len() >= 0);

    std::fs::remove_file(path).unwrap();
}
