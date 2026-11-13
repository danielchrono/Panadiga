use panadiga_core::constants::*;
use panadiga_core::nvram::clean_flash_nvram;
use std::fs;

#[test]
fn test_nvram_clean_lifecycle() {
    let temp_dir = std::env::temp_dir().join("panadiga_nvram_test_modular");
    fs::create_dir_all(&temp_dir).expect("Failed to create temp dir");

    // 1. Create a simulated 8MB flash dump with bootloader and dirty NVRAM
    let mut flash_sim = vec![0xAAu8; SPANSION_FLASH_SIZE_BYTES];

    // Seed microsectors with varying dirty data
    for idx in 0..SPANSION_MICROSECTOR_COUNT {
        let sec_offset = SPANSION_NVRAM_BASE_OFFSET + idx * SPANSION_MICROSECTOR_SIZE;
        flash_sim[sec_offset] = (0x10 + idx) as u8;
        flash_sim[sec_offset + 1] = 0x20;
        flash_sim[sec_offset + 2] = 0x30;
    }

    let input_path = temp_dir.join("test_spansion_dirty.bin");
    let output_path = temp_dir.join("test_spansion_clean.bin");
    fs::write(&input_path, &flash_sim).expect("Failed writing dirty flash");

    // 2. Execute clean
    let report = clean_flash_nvram(&input_path, Some(&output_path)).expect("Clean failed");

    assert!(report.bootloader_immutable);
    assert_eq!(report.microsectors.len(), 8);
    assert_eq!(report.microsectors[0].sector_name, "SA134");
    assert_eq!(report.microsectors[7].sector_name, "SA141");

    // 3. Verify clean image file content
    let clean_data = fs::read(&output_path).expect("Failed reading clean binary");
    assert_eq!(clean_data.len(), SPANSION_FLASH_SIZE_BYTES);

    // Bootloader must be untouched
    assert_eq!(
        &clean_data[..SPANSION_BOOTLOADER_SIZE_BYTES],
        &flash_sim[..SPANSION_BOOTLOADER_SIZE_BYTES]
    );

    // NVRAM must be all 0xFF
    assert!(
        clean_data[SPANSION_NVRAM_BASE_OFFSET..SPANSION_NVRAM_END_OFFSET]
            .iter()
            .all(|&b| b == 0xFF)
    );

    // 4. Verify backup was created
    let backup_path = temp_dir.join("nvram_corrupt_backup.bin");
    assert!(backup_path.exists());
    let backup_data = fs::read(&backup_path).expect("Failed reading backup");
    assert_eq!(backup_data.len(), SPANSION_NVRAM_SIZE_BYTES);

    // 5. Cross-device multi-model check: 4MB Flash (DMR-E85H / E95H / ES20)
    let flash_4mb_size = 4 * 1024 * 1024;
    let mut flash_4mb = vec![0x55u8; flash_4mb_size];
    let nvram_4mb_start = flash_4mb_size - SPANSION_NVRAM_SIZE_BYTES;
    flash_4mb[nvram_4mb_start] = 0x99;
    let in_4mb = temp_dir.join("test_4mb_dirty.bin");
    let out_4mb = temp_dir.join("test_4mb_clean.bin");
    fs::write(&in_4mb, &flash_4mb).unwrap();
    let rep_4mb = clean_flash_nvram(&in_4mb, Some(&out_4mb)).unwrap();
    assert_eq!(rep_4mb.microsectors[0].sector_name, "SA70");
    assert_eq!(rep_4mb.total_flash_bytes, flash_4mb_size);
    let clean_4mb = fs::read(&out_4mb).unwrap();
    assert!(clean_4mb[nvram_4mb_start..].iter().all(|&b| b == 0xFF));

    // Clean up
    let _ = fs::remove_dir_all(&temp_dir);
}
