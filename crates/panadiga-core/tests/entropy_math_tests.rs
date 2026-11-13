use panadiga_core::crypto::analyze_8byte_frequencies;
use panadiga_core::hdd::calculate_entropy;
use panadiga_core::nvram::calculate_hashes;

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

#[test]
fn test_parallel_large_buffer_entropy_hashes_and_ecb_blocks() {
    // >= 1 MiB exercises parallel paths in entropy (>= 512 KiB), hashes (>= 64 KiB),
    // and 8-byte ECB frequency counting (>= 512 KiB).
    let size = 1024 * 1024;
    let constant = vec![0xA5u8; size];
    assert_eq!(calculate_entropy(&constant), 0.0);

    let (md5_a, sha_a) = calculate_hashes(&constant);
    let (md5_b, sha_b) = calculate_hashes(&constant);
    assert_eq!(md5_a.len(), 32);
    assert_eq!(sha_a.len(), 64);
    assert_eq!((md5_a, sha_a), (md5_b, sha_b));

    let blocks_const = analyze_8byte_frequencies(&constant, 4);
    assert_eq!(blocks_const.len(), 1);
    assert_eq!(blocks_const[0].hex, "A5A5A5A5A5A5A5A5");
    assert_eq!(blocks_const[0].count, size / 8);

    let ramp: Vec<u8> = (0..size).map(|idx| (idx & 0xFF) as u8).collect();
    assert!((calculate_entropy(&ramp) - 8.0).abs() < 1e-6);
    let blocks_ramp = analyze_8byte_frequencies(&ramp, 32);
    assert_eq!(blocks_ramp.len(), 32);
    assert_eq!(blocks_ramp[0].count, size / 256);
}
