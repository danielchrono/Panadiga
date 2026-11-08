use panadiga::constants::*;

#[test]
fn test_system_constants_integrity() {
    assert_eq!(SPANSION_FLASH_SIZE_BYTES, 8 * 1024 * 1024);
    assert_eq!(SPANSION_BOOTLOADER_SIZE_BYTES, 0x007F_0000);
    assert_eq!(SPANSION_NVRAM_BASE_OFFSET, 0x007F_0000);
    assert_eq!(SPANSION_NVRAM_SIZE_BYTES, 64 * 1024);
    assert_eq!(SPANSION_MICROSECTOR_SIZE, 8 * 1024);
    assert_eq!(SPANSION_MICROSECTOR_COUNT, 8);
    assert_eq!(SPANSION_MICROSECTOR_BASE_INDEX, 134);

    assert_eq!(
        PANASONIC_HDD_MBR_SIGNATURE,
        [0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x53, 0x48]
    );
    assert_eq!(
        PANASONIC_HDD_PARTITION_MAGIC,
        [0x51, 0x87, 0x13, 0x8D, 0x5B, 0x0A, 0xFB, 0x22]
    );

    assert_eq!(LBA_SECTOR_SIZE_BYTES, 512);
    assert_eq!(LBA_MIRROR_MBR_OFFSET, 0x01C0_0000);
    assert_eq!(LBA_MIRROR_MBR_SECTOR, 57344);
    assert_eq!(LBA_PARTITION_1_OFFSET, 0x0200_0000);
    assert_eq!(LBA_PARTITION_2_OFFSET, 0x0400_0000);

    assert_eq!(
        ECB_NULL_BLOCK_8B,
        [0xE5, 0xD1, 0xF3, 0x4A, 0x51, 0x8B, 0x26, 0x90]
    );
    assert_eq!(
        ECB_VIRGIN_FLASH_8B,
        [0x03, 0x50, 0x65, 0xC8, 0x7B, 0xB8, 0xE0, 0x90]
    );
}

#[test]
fn test_constants_memory_map_and_candidate_paths() {
    // Verify 8 architectural regions defined in SSOT
    assert_eq!(AM34_SYSTEM_MEMORY_REGIONS.len(), 8);

    let names: Vec<&str> = AM34_SYSTEM_MEMORY_REGIONS.iter().map(|r| r.name).collect();
    assert_eq!(
        names,
        vec![
            "INTERNAL_SRAM",
            "CORE_MMIO",
            "MAIN_SDRAM",
            "FLASH_NOR_IPL",
            "NVRAM_U99",
            "SYSTEM_MMIO",
            "DWIRE32A_OCD",
            "ATA_IDE_MMIO",
        ]
    );

    // Verify ASCII table generation
    let table = format_memory_map_table();
    assert!(table.contains("INTERNAL_SRAM"));
    assert!(table.contains("0x00000000"));
    assert!(table.contains("DWIRE32A_OCD"));
    assert!(table.contains("0xC0004000"));

    // Verify JSON array generation
    let json_val = get_am34_memory_map_json();
    assert!(json_val.is_array());
    let arr = json_val.as_array().unwrap();
    assert_eq!(arr.len(), 8);
    assert_eq!(arr[0]["name"], "INTERNAL_SRAM");
    assert_eq!(arr[0]["perms"], "rwx");

    // Verify candidate paths non-empty
    assert!(!CANDIDATE_HDD_RAW_BACKUP_PATHS.is_empty());
    assert!(!CANDIDATE_MANUALS_DIRS.is_empty());
}
