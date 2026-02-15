use engine::OmniMind;

fn main() {
    println!("Running OmniMind Library Demo...");

    let mut mind = OmniMind::new_forge("./demo_data");

    // 1. Learn
    println!("Learning: 'The sun is hot'");
    mind.learn("The sun is hot");

    // 2. Query
    let query = "The sun is hot";
    let answer = mind.ask(query);
    println!("Query: '{}' -> Answer: '{}'", query, answer);

    assert!(answer.contains("Closest match"));
    println!("Demo successful!");

    std::fs::remove_dir_all("./demo_data").unwrap_or(());
}
