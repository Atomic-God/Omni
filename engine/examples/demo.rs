use engine::OmniMind;

fn main() {
    println!("Running OmniMind Library Demo...");

    let mut mind = OmniMind::new();

    // 1. Learn
    println!("Learning: 'The sun is hot'");
    mind.learn("The sun is hot");

    // 2. Query
    let query = "Is sun hot?";
    let answer = mind.ask(query);
    println!("Query: '{}' -> Answer: '{}'", query, answer);

    assert_eq!(answer, "Yes");
    println!("Demo successful!");
}
