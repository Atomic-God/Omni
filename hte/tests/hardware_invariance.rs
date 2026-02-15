#[cfg(test)]
mod tests {
    use hte::HardwareProfile;

    #[test]
    fn test_hardware_invariance() {
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

        assert_eq!(p1.avx2, true);
        assert_eq!(p2.avx2, false);
    }
}
