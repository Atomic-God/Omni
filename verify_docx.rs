use engine::OmniMind;

fn main() {
    let mut mind = OmniMind::new_forge("./test_docx_verify");
    mind.ingest_file("test_knowledge.docx").expect("Ingestion failed");
    let response = mind.ask("What is in DOCX?");
    println!("Response: {}", response.answer);
    if response.answer.contains("Industrial") {
        println!("VERIFICATION SUCCESS");
    } else {
        println!("VERIFICATION FAILURE");
    }
}
