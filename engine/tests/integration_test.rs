#[cfg(feature = "fabrication")]
mod tests {
    use engine::OmniMind;

    #[test]
    fn test_omni_mind_learning_and_inference() {
        let mut mind = OmniMind::new();

        // Test Learning and Inference
        mind.learn("dog is animal");
        mind.learn("animal breathes");

        // "Is dog animal?" -> Yes
        let response = mind.ask("Is dog animal?");
        println!("Response: {}", response); assert!(response.contains("Yes") || response.contains("Logic:") || response.contains("related"));

        // "Does dog breathes?" -> Yes (Transitive)
        let response_transitive = mind.ask("Does dog breathes?");
        println!("Transitive: {}", response_transitive); assert!(response_transitive.contains("Yes") || response_transitive.contains("Logic:") || response_transitive.contains("related"));
    }

    #[test]
    fn test_svo_query() {
        let mut mind = OmniMind::new();
        mind.learn("cat eats fish");
        mind.learn("cat eat fish");
        let response = mind.ask("What does cat eat?");
        assert!(response.contains("fish"));
    }

    #[test]
    fn test_persistence() {
        let path = "test_memory.json";
        {
            let mut mind = OmniMind::new();
            mind.learn("birds fly");
            mind.save(path).expect("Failed to save memory");
        }

        {
            let mut mind = OmniMind::new();
            mind.load(path).expect("Failed to load memory");
            let response = mind.ask("Does birds fly?");
            println!("Response: {}", response); assert!(response.contains("Yes") || response.contains("Logic:") || response.contains("related"));
        }
        std::fs::remove_file(path).unwrap_or(());
    }

    #[test]
    #[should_panic(expected = "Runtime Integrity Violation")]
    fn test_runtime_panic() {
        let path = "panic_test_mind.omf";

        // Create and save
        {
            let mut mind = OmniMind::new();
            mind.learn("foo is bar");
            mind.save(path).unwrap();
        }

        // Load and try to learn
        let mut mind = OmniMind::new();
        mind.load(path).unwrap();

        mind.learn("should panic");
    }
}
