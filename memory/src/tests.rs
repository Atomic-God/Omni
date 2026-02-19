#[cfg(test)]
mod tests {
    use super::*;
    use core_vsa::HyperVector;
    use crate::{MemoryManager, HierarchicalMemory, MemoryLayer, CorruptionRecovery};
    use std::path::PathBuf;

    #[test]
    fn test_memory_hierarchy() {
        let root = PathBuf::from("test_mem_hierarchy");
        let _ = std::fs::remove_dir_all(&root);
        let mut mem = MemoryManager::new(&root);

        let v = HyperVector::random();
        mem.store_in_layer("item1", v.clone(), MemoryLayer::Working).unwrap();

        // Access 50 times
        for _ in 0..50 {
            mem.retrieve_with_metrics("item1");
        }

        // Consolidate
        mem.consolidate_layers();

        let entry = mem.metadata.get("item1").unwrap();
        assert_eq!(entry.layer, "episodic"); // Should promote to episodic

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn test_bit_flip_recovery() {
        // Stub for recovery test
        // Verify that CorruptionRecovery::check_integrity returns true for clean memory
        let root = PathBuf::from("test_recovery");
        let _ = std::fs::remove_dir_all(&root);
        let mem = MemoryManager::new(&root);

        assert!(CorruptionRecovery::check_integrity(&mem.storage));
        let _ = std::fs::remove_dir_all(&root);
    }
}
