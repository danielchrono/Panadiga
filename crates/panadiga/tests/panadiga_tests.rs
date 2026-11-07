use std::fs;
use std::path::Path;

use panadiga::constants::*;
use panadiga::crypto::{analyze_frm_container, dump_toc, print_codebook};
use panadiga::hdd::{calculate_entropy, decode_descriptors, decode_meihdfs_descriptors, inspect_hdd_image};
use panadiga::headers::{hex_dump, inspect_firmware_headers};
use panadiga::manuals::search_manuals;
use panadiga::nvram::clean_flash_nvram;
use panadiga::pico::{
    classify_usb_device, list_pico_devices, HostControllerPlatform, PicoArchitecture,
    ARDUINO_NANO_RP2040_CONNECT_PID, ARDUINO_VID, PICO_BOOTSEL_PID, PICO_RP2040_CDC_PID,
    PICO_RP2350_CDC_PID, RASPBERRY_PI_VID, SEEED_VID, SEEED_XIAO_RP2040_PID,
    SEEED_XIAO_RP2350_PID,
};
use panadiga::scan::scan_data_strings;

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
fn test_nvram_clean_lifecycle() {
    let temp_dir = std::env::temp_dir().join("panadiga_nvram_test");
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
    assert!(clean_data[SPANSION_NVRAM_BASE_OFFSET..SPANSION_NVRAM_END_OFFSET]
        .iter()
        .all(|&b| b == 0xFF));

    // 4. Verify backup was created
    let backup_path = temp_dir.join("nvram_corrupt_backup.bin");
    assert!(backup_path.exists());
    let backup_data = fs::read(&backup_path).expect("Failed reading backup");
    assert_eq!(backup_data.len(), SPANSION_NVRAM_SIZE_BYTES);

    // Clean up
    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_real_hdd_inspect() {
    let hdd_path = Path::new("firmware_and_images/panasonic DMR EH-55 Firmware hdd_100.bin");
    if !hdd_path.exists() {
        return;
    }

    let report = inspect_hdd_image(hdd_path, false, None).expect("HDD inspect failed");

    assert_eq!(report.file_size_bytes, 104_857_600);
    assert_eq!(report.total_sectors, 204_800);
    assert_eq!(report.mbr_tables.magic_ascii, "SH");
    assert!(report.mbr_tables.mirror_is_bit_identical);
    assert_eq!(
        report.mbr_tables.lba0_sha256,
        "61162c24c9240408ac807cd4307c9c5833d440715a982126c536aa10aa7530f2"
    );
    assert_eq!(report.partitions.len(), 2);
    assert_eq!(
        report.partitions[0].exact_sha256,
        "6ec63de059c00b287a0e8a1a4102a8007bbccf3210a552bfb45118a03b85c898"
    );
    assert_eq!(
        report.partitions[1].exact_sha256,
        "487d5951d4aa7bc45e5f634c4ca079f369db06d5cb62e41a173575cecd68dbae"
    );
}

#[test]
fn test_real_frm_cryptanalysis() {
    let frm_path = Path::new("firmware_and_images/PANA_DVD.FRM");
    if !frm_path.exists() {
        return;
    }

    let report = analyze_frm_container(frm_path, None, 10, None).expect("FRM analysis failed");

    assert_eq!(report.file_size_bytes, 7_531_744);
    assert!((report.global_entropy - 7.99957973).abs() < 0.001);
    assert!(report.is_ecb_leak_confirmed);

    // Verify top frequent block is virgin flash (0xFF * 8)
    assert_eq!(report.top_blocks[0].hex, "035065C87BB8E090");
    assert_eq!(report.top_blocks[0].count, 3812);

    // Verify null block is present
    let null_blk = report.top_blocks.iter().find(|b| b.hex == "E5D1F34A518B2690");
    assert!(null_blk.is_some());
    assert_eq!(null_blk.unwrap().count, 196);
}

#[test]
fn test_real_hdd_decode_descriptors() {
    let hdd_path = Path::new("firmware_and_images/panasonic DMR EH-55 Firmware hdd_100.bin");
    if !hdd_path.exists() {
        return;
    }
    assert!(decode_descriptors(hdd_path).is_ok());
}

#[test]
fn test_real_frm_dump_toc_and_codebook() {
    let frm_path = Path::new("firmware_and_images/PANA_DVD.FRM");
    if !frm_path.exists() {
        return;
    }

    // 1. Verify TOC Dump
    let toc_report = dump_toc(frm_path, None).expect("TOC dump failed");
    assert_eq!(toc_report.file_size_bytes, 7_531_744);
    assert_eq!(
        toc_report.global_header_hex,
        "492F0560CEDE508CE3C08AFD49B64A9A"
    );
    assert_eq!(toc_report.toc_entries.len(), 3);
    assert!(toc_report.toc_invariants_valid);

    // Entry 0 checks
    assert_eq!(toc_report.toc_entries[0].offset_hex, "0x0010");
    assert_eq!(toc_report.toc_entries[0].processor_family_id, "EFDC1CDA13881BAB");
    assert_eq!(toc_report.toc_entries[0].target_descriptor_id, "74A78ACD789B84FF");

    // Entry 1 checks
    assert_eq!(toc_report.toc_entries[1].offset_hex, "0x0040");
    assert_eq!(toc_report.toc_entries[1].processor_family_id, "EFDC1CDA13881BAB");
    assert_eq!(toc_report.toc_entries[1].target_descriptor_id, "3B7E7EDBB6C57A53");

    // Entry 2 checks
    assert_eq!(toc_report.toc_entries[2].offset_hex, "0x0070");
    assert_eq!(toc_report.toc_entries[2].processor_family_id, "EFDC1CDA13881BAB");
    assert_eq!(toc_report.toc_entries[2].target_descriptor_id, "74A78ACD789B84FF");

    // Modules verification
    assert_eq!(toc_report.modules.len(), 3);
    assert_eq!(toc_report.modules[0].start_offset_hex, "0x000000A0");
    assert_eq!(toc_report.modules[0].end_offset_hex, "0x000B1800");
    assert_eq!(toc_report.modules[1].start_offset_hex, "0x000B1800");
    assert_eq!(toc_report.modules[1].end_offset_hex, "0x000F1818");
    assert_eq!(toc_report.modules[2].start_offset_hex, "0x000F1818");

    // 2. Verify Codebook Dump
    let codebook_report = print_codebook(frm_path, None, None).expect("Codebook dump failed");
    assert!(codebook_report.is_ecb_leak_confirmed);
    assert!(codebook_report.cross_medium_key_identity_proven);

    // Check virgin flash block in dictionary
    let virgin = codebook_report
        .dictionary
        .iter()
        .find(|e| e.block_hex == "035065C87BB8E090")
        .expect("Virgin flash block missing from codebook");
    assert_eq!(virgin.frm_occurrences, 3812);
    assert_eq!(virgin.hdd_occurrences, 1280);

    // Check null block in dictionary
    let null_blk = codebook_report
        .dictionary
        .iter()
        .find(|e| e.block_hex == "E5D1F34A518B2690")
        .expect("Null block missing from codebook");
    assert_eq!(null_blk.frm_occurrences, 196);
    assert_eq!(null_blk.hdd_occurrences, 3);
}

#[test]
fn test_headers_and_hex_dump() {
    let sample = b"Panasonic DIGA DMR-EH55 Engineering Suite Test Vector!";
    let dump = hex_dump(sample, 0x1000, 32);
    assert!(dump.contains("00001000"));
    assert!(dump.contains("Panasonic DIGA"));

    // Verify inspecting actual firmware headers runs cleanly
    assert!(inspect_firmware_headers(None, None, 64).is_ok());
}

#[test]
fn test_scan_strings() {
    let mut data = vec![0u8; 128];
    // Write valid ASCII strings with padding
    data[10..24].copy_from_slice(b"panasonic_diga");
    data[50..65].copy_from_slice(b"error_u99_fatal");

    let keywords = vec!["u99".to_string(), "panasonic".to_string()];
    let (total, matches) = scan_data_strings(&data, 6, &keywords, 10);

    assert_eq!(total, 2);
    assert_eq!(matches.len(), 2);
    assert_eq!(matches[0].text, "panasonic_diga");
    assert_eq!(matches[0].matched_keyword.as_deref(), Some("panasonic"));
    assert_eq!(matches[1].text, "error_u99_fatal");
    assert_eq!(matches[1].matched_keyword.as_deref(), Some("u99"));
}

#[test]
fn test_manuals_search_pdf() {
    let manuals_dir = Path::new("manuals_and_schematics");
    if !manuals_dir.exists() {
        return;
    }

    // Search for U99 with filter for dmreh55.pdf or eh55
    let terms = vec!["U99".to_string(), "FIRMWARE".to_string()];
    let res = search_manuals(manuals_dir, Some(terms), Some("dmr-eh55p9")).expect("Manual search failed");

    assert!(!res.is_empty());
    let r = &res[0];
    assert!(r.file_name.contains("eh55"));
}

#[test]
fn test_decode_meihdfs_descriptors_standalone() {
    let desc_path = Path::new("firmware_and_images/extracted_partitions/hdd_allocation_descriptors_0x00003000.bin");
    if !desc_path.exists() {
        return;
    }

    let report = decode_meihdfs_descriptors(desc_path, false, None)
        .expect("Failed to decode standalone descriptors");

    assert_eq!(report.total_records, 256);
    assert_eq!(
        report.descriptors_sha256,
        "4b604ad2a689bd39cc8667b2479e70e304d574a78fcf72bfdeb91ca138fdce24"
    );

    // Verify tag distribution counts:
    // 0x5D: 65, 0x55: 64, 0x57: 47, 0x5F: 12, 0xE0: 42, 0xE1: 26
    let dist = &report.tag_distribution;
    assert_eq!(dist.get("0x5D (ExtentHeader)").copied().unwrap_or(0), 65);
    assert_eq!(dist.get("0x55 (ExtentPointer)").copied().unwrap_or(0), 64);
    assert_eq!(dist.get("0x57 (ExtentContinuation)").copied().unwrap_or(0), 47);
    assert_eq!(dist.get("0x5F (ExtentBoundary)").copied().unwrap_or(0), 12);
    assert_eq!(dist.get("0xE0 (VNodeEntry)").copied().unwrap_or(0), 42);
    assert_eq!(dist.get("0xE1 (VInodeLink)").copied().unwrap_or(0), 26);

    let sum_tags: usize = dist.values().sum();
    assert_eq!(sum_tags, 256);
    assert!(!report.extents.is_empty());
}

#[test]
fn test_decode_meihdfs_descriptors_full_hdd_with_journal() {
    let hdd_path = Path::new("firmware_and_images/panasonic DMR EH-55 Firmware hdd_100.bin");
    if !hdd_path.exists() {
        return;
    }

    let report = decode_meihdfs_descriptors(hdd_path, true, None)
        .expect("Failed to decode descriptors from full HDD image");

    assert!(report.is_full_hdd_image);
    assert!(report.is_mirror_verified);
    assert_eq!(report.total_records, 256);
    assert_eq!(
        report.descriptors_sha256,
        "4b604ad2a689bd39cc8667b2479e70e304d574a78fcf72bfdeb91ca138fdce24"
    );

    assert_eq!(report.journal_blocks.len(), 10);
    for (idx, jb) in report.journal_blocks.iter().enumerate() {
        assert_eq!(jb.block_index, idx);
        assert!(jb.is_active_header);
        assert_eq!(jb.transaction_seq, (idx + 1) as u32);
    }
}

#[test]
fn test_pico_architecture_and_device_listing() {
    assert_eq!(RASPBERRY_PI_VID, 0x2E8A);
    assert_eq!(PICO_RP2040_CDC_PID, 0x000A);
    assert_eq!(PICO_RP2350_CDC_PID, 0x000F);
    assert_eq!(PICO_BOOTSEL_PID, 0x0003);

    assert_eq!(
        PicoArchitecture::Rp2040.to_string(),
        "Raspberry Pi Pico (RP2040)"
    );
    assert_eq!(
        PicoArchitecture::Rp2350.to_string(),
        "Raspberry Pi Pico 2 (RP2350)"
    );
    assert_eq!(
        PicoArchitecture::Bootloader.to_string(),
        "Raspberry Pi BOOTSEL Mode"
    );

    // Multi-Platform classification tests across vendors
    let (plat_pico, _) = classify_usb_device(RASPBERRY_PI_VID, PICO_RP2040_CDC_PID, None);
    assert_eq!(plat_pico, HostControllerPlatform::RaspberryPiPico);
    assert_eq!(plat_pico.sram_size_bytes(), 264 * 1024);
    assert_eq!(plat_pico.sector_cache_capacity_bytes(), 8 * 1024);
    assert!(plat_pico.supports_dwire32a_pio());

    let (plat_pico2, _) = classify_usb_device(RASPBERRY_PI_VID, PICO_RP2350_CDC_PID, None);
    assert_eq!(plat_pico2, HostControllerPlatform::RaspberryPiPico2Arm);
    assert_eq!(plat_pico2.sram_size_bytes(), 520 * 1024);
    assert_eq!(plat_pico2.sector_cache_capacity_bytes(), 64 * 1024);
    assert!(plat_pico2.supports_dwire32a_pio());

    let (plat_arduino, _) = classify_usb_device(ARDUINO_VID, ARDUINO_NANO_RP2040_CONNECT_PID, None);
    assert_eq!(plat_arduino, HostControllerPlatform::ArduinoNanoRp2040);
    assert_eq!(plat_arduino.architecture(), PicoArchitecture::Rp2040);

    let (plat_seeed2040, _) = classify_usb_device(SEEED_VID, SEEED_XIAO_RP2040_PID, None);
    assert_eq!(plat_seeed2040, HostControllerPlatform::SeeedXiaoRp2040);

    let (plat_seeed2350, _) = classify_usb_device(SEEED_VID, SEEED_XIAO_RP2350_PID, None);
    assert_eq!(plat_seeed2350, HostControllerPlatform::SeeedXiaoRp2350);
    assert_eq!(plat_seeed2350.architecture(), PicoArchitecture::Rp2350);

    // Enumerating system serial devices must execute cleanly without crashing
    let devices = list_pico_devices();
    assert!(devices.is_ok());
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
        if i % 2 == 1 { *b = 0xFF; }
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
fn test_all_host_controller_platforms_matrix() {
    // Pico W
    let (plat_w, _) = classify_usb_device(0x2E8A, 0x0014, None);
    assert_eq!(plat_w, HostControllerPlatform::RaspberryPiPicoW);
    assert_eq!(plat_w.sram_size_bytes(), 264 * 1024);
    assert_eq!(plat_w.sector_cache_capacity_bytes(), 8 * 1024);

    // Pico 2 W
    let (plat_2w, _) = classify_usb_device(0x2E8A, 0x001F, None);
    assert_eq!(plat_2w, HostControllerPlatform::RaspberryPiPico2W);
    assert_eq!(plat_2w.sram_size_bytes(), 520 * 1024);
    assert_eq!(plat_2w.sector_cache_capacity_bytes(), 64 * 1024);

    // Pico 2 RISC-V Hazard3
    let (plat_riscv, _) = classify_usb_device(0x2E8A, 0x000E, None);
    assert_eq!(plat_riscv, HostControllerPlatform::RaspberryPiPico2Riscv);
    assert_eq!(plat_riscv.sram_size_bytes(), 520 * 1024);
    assert_eq!(plat_riscv.sector_cache_capacity_bytes(), 64 * 1024);

    // Adafruit Feather RP2040
    let (plat_ada, _) = classify_usb_device(0x239A, 0x80F0, Some("Adafruit Feather RP2040"));
    assert_eq!(plat_ada, HostControllerPlatform::AdafruitRp2040);
    assert_eq!(plat_ada.architecture(), PicoArchitecture::Rp2040);

    // SparkFun Pro Micro RP2040
    let (plat_spark, _) = classify_usb_device(0x1B4F, 0x0025, None);
    assert_eq!(plat_spark, HostControllerPlatform::SparkfunRp2040);
    assert_eq!(plat_spark.architecture(), PicoArchitecture::Rp2040);

    // Waveshare RP2040
    let (plat_wave, _) = classify_usb_device(0x1209, 0x2040, Some("Waveshare RP2040-Zero"));
    assert_eq!(plat_wave, HostControllerPlatform::WaveshareRp2040);
    assert_eq!(plat_wave.architecture(), PicoArchitecture::Rp2040);

    // Custom DWire32A probe with banner string
    let (plat_custom, _) = classify_usb_device(0x9999, 0x0001, Some("Panasonic DWire32A OCD Host Bridge [DW32A]"));
    assert_eq!(plat_custom, HostControllerPlatform::GenericCompatibleCdc);
    assert_eq!(plat_custom.architecture(), PicoArchitecture::Unknown);
}

#[test]
fn test_gdb_rsp_checksum_calculation() {
    fn calc_gdb_checksum(payload: &str) -> String {
        let sum: u32 = payload.bytes().map(|b| b as u32).sum();
        format!("{:02x}", (sum % 256) as u8)
    }

    assert_eq!(calc_gdb_checksum("qSupported"), "37");
    assert_eq!(calc_gdb_checksum("g"), "67");
    assert_eq!(calc_gdb_checksum("?"), "3f");
    assert_eq!(calc_gdb_checksum("c"), "63");
    assert_eq!(calc_gdb_checksum("s"), "73");
    assert_eq!(calc_gdb_checksum("OK"), "9a");
}
