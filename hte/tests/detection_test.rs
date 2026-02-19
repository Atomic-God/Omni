use hte;

#[test]
fn test_hardware_detection() {
    let profile = hte::detect();
    println!("Detected Profile: {:?}", profile);

    // Basic assertions
    // Physical cores should be at least 1
    assert!(profile.physical_cores >= 1);
    assert!(profile.logical_cores >= 1);

    // Cache sizes might be None in some environments (like CI/containers), but shouldn't panic
    if let Some(l1) = profile.l1_cache_size {
        assert!(l1 > 0);
    }
}
