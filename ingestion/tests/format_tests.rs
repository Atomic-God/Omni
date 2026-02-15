use ingestion::ingest_graph;
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
    let options = FileOptions::default();

    zip.start_file("hello.txt", options).unwrap();
    zip.write_all(b"Hello world inside zip.").unwrap();
    zip.finish().unwrap();

    let graph = ingest_graph(std::path::Path::new(test_dir)).expect("Ingestion failed");
    assert!(!graph.nodes.is_empty(), "Should ingest zip content");

    std::fs::remove_dir_all(test_dir).unwrap_or(());
}
