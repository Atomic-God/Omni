use ingestion::{ingest_path, SemanticChunk};
use std::fs::File;
use std::io::Write;
use zip::write::FileOptions;

#[test]
fn test_zip_ingestion() {
    let test_dir = "test_data_zip";
    std::fs::create_dir_all(test_dir).unwrap();
    let zip_path = format!("{}/archive.zip", test_dir);

    let file = File::create(&zip_path).unwrap();
    let mut zip = zip::ZipWriter::new(file);
    let options: FileOptions<()> = FileOptions::default().compression_method(zip::CompressionMethod::Stored);

    zip.start_file("hello.txt", options).unwrap();
    zip.write_all(b"Hello world inside zip.").unwrap();
    zip.finish().unwrap();

    let chunks = ingest_path(std::path::PathBuf::from(test_dir));
    assert!(!chunks.is_empty(), "Should ingest zip content");
    assert!(chunks.iter().any(|c| c.content.contains("Hello world")), "Content mismatch");
    assert_eq!(chunks[0].metadata.file_type, "zip_entry");

    std::fs::remove_dir_all(test_dir).unwrap();
}
