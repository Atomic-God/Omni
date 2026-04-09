use hte;

#[test]
fn test_hardware_detection() {
    let profile = hte::detect();
    println!("Detected Profile: {:?}", profile);

    // Basic assertions
    assert!(profile.physical_cores >= 1);
    assert!(profile.logical_cores >= 1);
}
