use panadiga::hdd::calculate_entropy;

#[test]
fn test_entropy_calculation() {
    let zeros = vec![0u8; 1024];
    assert_eq!(calculate_entropy(&zeros), 0.0);

    let all_ff = vec![0xFFu8; 1024];
    assert_eq!(calculate_entropy(&all_ff), 0.0);

    let mut uniform = Vec::with_capacity(256 * 10);
    for _ in 0..10 {
        for b in 0..=255u8 {
            uniform.push(b);
        }
    }
    let ent = calculate_entropy(&uniform);
    assert!((ent - 8.0).abs() < 0.0001);
}

#[test]
fn test_entropy_mathematical_boundaries() {
    // Pure null vector: entropy must be mathematically 0.0
    let zeros = vec![0x00u8; 1024];
    assert_eq!(calculate_entropy(&zeros), 0.0);

    // Pure virgin flash vector (all bits set): entropy must be 0.0
    let virgin = vec![0xFFu8; 1024];
    assert_eq!(calculate_entropy(&virgin), 0.0);

    // 2-symbol equal distribution (0x00 and 0xFF): exactly 1.0 bit of entropy
    let mut binary = vec![0u8; 1024];
    for (i, b) in binary.iter_mut().enumerate() {
        if i % 2 == 1 {
            *b = 0xFF;
        }
    }
    let ent_bin = calculate_entropy(&binary);
    assert!((ent_bin - 1.0).abs() < 1e-4);

    // Uniform 256-symbol distribution: maximum theoretical entropy (8.0 bits)
    let mut uniform = Vec::with_capacity(256 * 10);
    for _ in 0..10 {
        for b in 0..=255u8 {
            uniform.push(b);
        }
    }
    let ent_uni = calculate_entropy(&uniform);
    assert!((ent_uni - 8.0).abs() < 1e-4);
}
