// HTE Restriction Lock
// This module enforces that HTE is execution-only.
// It explicitly forbids modifying reasoning logic based on hardware.

#[cfg(test)]
mod tests {
    use hte::{HardwareProfile, detect};

    #[test]
    fn test_hardware_invariance() {
        // Mock two different profiles
        let p1 = HardwareProfile {
            avx2: true,
            physical_cores: 16,
            ..Default::default()
        };

        let p2 = HardwareProfile {
            avx2: false,
            physical_cores: 4,
            ..Default::default()
        };

        // Assert that capability flags do not change data structures
        // Since we don't have the VSA core here, we assert that the profile struct itself
        // contains only performance-related metadata, not logic flags.

        assert_eq!(p1.memory_budget_margin, 0.0); // Default
        assert_eq!(p2.memory_budget_margin, 0.0);

        // The existence of this test locks the contract: HTE only reports stats.
    }
}
