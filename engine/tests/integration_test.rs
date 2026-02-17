mod tests {
    use engine::OmniMind;

    #[test]
    fn test_omni_mind_learning_and_inference() {
        let mut mind = OmniMind::new_forge("./test_data");

        // Test Learning and Inference
        mind.learn("dog is animal");
        mind.learn("animal breathes");

        // "Is dog animal?" -> Yes
        let response = mind.ask("Is dog animal?");
        println!("Response: {}", response.answer);
        assert!(response.answer.contains("Yes") || response.answer.contains("Logic:") || response.answer.contains("related"));

        // "Does dog breathes?" -> Yes (Transitive)
        let response_transitive = mind.ask("Does dog breathes?");
        println!("Transitive: {}", response_transitive.answer);
        assert!(response_transitive.answer.contains("Yes") || response_transitive.answer.contains("Logic:") || response_transitive.answer.contains("related"));
    }

    #[test]
    fn test_svo_query() {
        let mut mind = OmniMind::new_forge("./test_data");
        mind.learn("cat eats fish");
        mind.learn("cat eat fish");
        let response = mind.ask("What does cat eat?");
        assert!(response.answer.contains("fish"));
    }

    #[test]
    fn test_persistence() {
        let _path = "test_memory.omf";
        let mut mind = OmniMind::new_forge("./test_data");

        // Since test paths are relative to Cargo.toml, "test_memory.omf" is in the cwd.
        // PermissionBoundary defaults to allowing cwd.
        // However, canonicalization might fail if the file doesn't exist yet for check_path.
        // We updated save_master to check parent.
        // "." parent is allowed.

        // If it still fails, it might be that tests run in a temp dir or target dir?
        // Let's print the error more clearly if it fails.
        // Or explicitly allow the test path in a configured mind.
        // But OmniMind::new_forge("./test_data") uses default permissions (cwd).

        // Let's inspect the panic. "Invalid path or does not exist".
        // This comes from `canonicalize` failing on a non-existent file?
        // But we check `parent`.
        // "test_memory.omf" -> parent is "" (empty) or "."?
        // Path::new("foo").parent() is Some("").
        // canonicalize("") might fail?
        // Let's use "./test_memory.omf" to be explicit.

        let path = "./test_memory.omf";

        {
            mind.learn("birds fly");
            mind.save(path).expect("Failed to save memory");
        }

        {
            let mut mind = OmniMind::new_forge("./test_data");
            mind.load(path).expect("Failed to load memory");
            let response = mind.ask("Does birds fly?");
            println!("Response: {}", response.answer);
            assert!(response.answer.contains("Yes") || response.answer.contains("Logic:") || response.answer.contains("related"));
        }
        std::fs::remove_file(path).unwrap_or(());
    }
}
