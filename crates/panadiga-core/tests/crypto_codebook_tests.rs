use panadiga_core::constants::{
    ECB_NULL_BLOCK_8B, ECB_VIRGIN_FLASH_8B, PANA_DIGADIAG_DEFAULT_LOAD_ADDR,
    PANA_DIGADIAG_FRM_SIZE, PANA_EXTRHEAD_DEFAULT_ADLER_STEP, PANA_FEISTEL_KEY_PART1,
    PANA_FEISTEL_KEY_PART2, PANA_FEISTEL_KEYSCHED_IV, PANA_FEISTEL_MASTER_KEY_7B,
    PANA_FEISTEL_ROUND_KEY_32B, PANA_FEISTEL_SBOX_256,
};
use panadiga_core::crypto::{
    ExtrheadCryptMode, FrmContainerMode, FrmPackModule, adler32_panasonic_strided,
    analyze_frm_container, build_digadiag_frm, decrypt_and_parse_frm_toc, derive_round_key_32b,
    dump_toc, feistel_decrypt_block, feistel_decrypt_buffer, feistel_encrypt_block,
    feistel_encrypt_buffer, pack_extrhead, pack_frm, parse_sd_frm_toc, print_codebook,
    reconstruct_master_key_7b, unpack_extrhead,
};
use std::path::Path;

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
    let null_blk = report
        .top_blocks
        .iter()
        .find(|b| b.hex == "E5D1F34A518B2690");
    assert!(null_blk.is_some());
    assert_eq!(null_blk.unwrap().count, 196);
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
    assert_eq!(
        toc_report.toc_entries[0].processor_family_id,
        "EFDC1CDA13881BAB"
    );
    assert_eq!(
        toc_report.toc_entries[0].target_descriptor_id,
        "74A78ACD789B84FF"
    );

    // Entry 1 checks
    assert_eq!(toc_report.toc_entries[1].offset_hex, "0x0040");
    assert_eq!(
        toc_report.toc_entries[1].processor_family_id,
        "EFDC1CDA13881BAB"
    );
    assert_eq!(
        toc_report.toc_entries[1].target_descriptor_id,
        "3B7E7EDBB6C57A53"
    );

    // Entry 2 checks
    assert_eq!(toc_report.toc_entries[2].offset_hex, "0x0070");
    assert_eq!(
        toc_report.toc_entries[2].processor_family_id,
        "EFDC1CDA13881BAB"
    );
    assert_eq!(
        toc_report.toc_entries[2].target_descriptor_id,
        "74A78ACD789B84FF"
    );

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
fn test_feistel_cipher_and_real_toc_decryption() {
    let master = reconstruct_master_key_7b(&PANA_FEISTEL_KEY_PART1, &PANA_FEISTEL_KEY_PART2);
    assert_eq!(master, PANA_FEISTEL_MASTER_KEY_7B);
    assert_eq!(master, [0xDF, 0x62, 0x15, 0xC7, 0x52, 0x7F, 0xD0]);

    let rk = derive_round_key_32b(&master, &PANA_FEISTEL_KEYSCHED_IV, &PANA_FEISTEL_SBOX_256);
    assert_eq!(rk, PANA_FEISTEL_ROUND_KEY_32B);

    // Known plaintext <-> ciphertext ECB blocks observed in PANA_DVD.FRM and HDD partitions
    assert_eq!(
        feistel_decrypt_block(&ECB_NULL_BLOCK_8B, &rk, &PANA_FEISTEL_SBOX_256),
        [0x00u8; 8]
    );
    assert_eq!(
        feistel_decrypt_block(&ECB_VIRGIN_FLASH_8B, &rk, &PANA_FEISTEL_SBOX_256),
        [0xFFu8; 8]
    );
    assert_eq!(
        feistel_encrypt_block(&[0x00u8; 8], &rk, &PANA_FEISTEL_SBOX_256),
        ECB_NULL_BLOCK_8B
    );
    assert_eq!(
        feistel_encrypt_block(&[0xFFu8; 8], &rk, &PANA_FEISTEL_SBOX_256),
        ECB_VIRGIN_FLASH_8B
    );

    // Arbitrary multi-block roundtrip + trailing byte preservation
    let original = b"EXTRHEADMAIN_TEST_VEC_123".to_vec();
    let mut work = original.clone();
    feistel_encrypt_buffer(&mut work, &rk, &PANA_FEISTEL_SBOX_256);
    assert_ne!(work[..24], original[..24]);
    assert_eq!(work[24], b'3');
    feistel_decrypt_buffer(&mut work, &rk, &PANA_FEISTEL_SBOX_256);
    assert_eq!(work, original);

    // Strided Adler-32 sanity check (empty & step=16)
    assert_eq!(adler32_panasonic_strided(&[], 16), 1);
    assert_eq!(
        adler32_panasonic_strided(&[0x41; 32], 16),
        (131 | (197 << 16))
    );

    // Real PANA_DVD.FRM TOC validation when fixture exists in workspace
    let frm_path = Path::new("firmware_and_images/PANA_DVD.FRM");
    if frm_path.exists() {
        let head = std::fs::read(frm_path).expect("Failed reading PANA_DVD.FRM");
        let entries = decrypt_and_parse_frm_toc(&head);
        assert_eq!(entries.len(), 3);

        assert_eq!(entries[0].module_name, "DRV S211");
        assert_eq!(entries[0].module_offset, 0x0000_1800);
        assert_eq!(entries[0].module_size, 0x000B_0000);
        assert_eq!(entries[0].list_sum_u16, 0x0AE1);
        assert!(entries[0].list_sum_valid);

        assert_eq!(entries[1].module_name, "PROG061 ");
        assert_eq!(entries[1].module_offset, 0x000B_1800);
        assert_eq!(entries[1].module_size, 0x0004_0000);
        assert_eq!(entries[1].list_sum_u16, 0x08F9);
        assert!(entries[1].list_sum_valid);

        assert_eq!(entries[2].module_name, "MAIN3060NP");
        assert_eq!(entries[2].module_offset, 0x000F_1800);
        assert_eq!(entries[2].module_size, 0x0063_D4E0);
        assert_eq!(entries[2].list_sum_u16, 0x0B26);
        assert!(entries[2].list_sum_valid);
    }
}

fn deterministic_pseudo_random(size: usize, mut state: u32) -> Vec<u8> {
    (0..size)
        .map(|_| {
            state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            (state >> 24) as u8
        })
        .collect()
}

#[test]
fn test_extrhead_pack_unpack_and_frm_packing() {
    // 1. Plaintext EXTRHEAD / DIGADIAG.FRM (128 KiB padded, target 0x50400000)
    let diag_payload = b"MN103S_DIGADIAG_STUB_".repeat(32);
    let (diag_frm, diag_report) = build_digadiag_frm(
        &diag_payload,
        PANA_DIGADIAG_DEFAULT_LOAD_ADDR,
        "FVU ",
        "DIAG99",
        PANA_EXTRHEAD_DEFAULT_ADLER_STEP as u32,
        PANA_DIGADIAG_FRM_SIZE,
    )
    .expect("build_digadiag_frm failed");
    assert_eq!(diag_frm.len(), PANA_DIGADIAG_FRM_SIZE);
    assert_eq!(diag_report.mode, "digadiag");
    assert!(diag_frm.starts_with(b"EXTRHEADFVU "));

    let unpacked_diag = unpack_extrhead(&diag_frm, ExtrheadCryptMode::Plain).unwrap();
    assert_eq!(unpacked_diag.header.dst_addr, 0x5040_0000);
    assert_eq!(unpacked_diag.header.build_name, "DIAG99");
    assert_eq!(unpacked_diag.decompressed, diag_payload);
    assert!(unpacked_diag.checksum_valid);
    assert!(unpacked_diag.footer_present);

    // 2. Partial Feistel DRV prefix mode (> 0x2800 B compressed) & MAIN split mode (> 0x32000 B compressed)
    let drv_plain = deterministic_pseudo_random(16 * 1024, 0xCAFE_BABE);
    let drv_container = pack_extrhead(
        &drv_plain,
        "DRV ",
        0x9020_2000,
        "S211",
        ExtrheadCryptMode::Auto,
        16,
        1,
    )
    .unwrap();
    assert_ne!(&drv_container[..8], b"EXTRHEAD");
    assert!(drv_container[0x2800..].windows(8).any(|w| w == b"EXTRFOOT"));
    let unpacked_drv = unpack_extrhead(&drv_container, ExtrheadCryptMode::Auto).unwrap();
    assert_eq!(unpacked_drv.header.tag, "DRV ");
    assert_eq!(unpacked_drv.decompressed, drv_plain);
    assert!(unpacked_drv.checksum_valid);
    assert!(unpacked_drv.footer_present);

    let main_plain = deterministic_pseudo_random(240 * 1024, 0xDEAD_5558);
    let main_container = pack_extrhead(
        &main_plain,
        "MAIN",
        0x5700_0000,
        "3410PQ",
        ExtrheadCryptMode::Auto,
        16,
        1,
    )
    .unwrap();
    assert!(main_container.len() > 0x3_2000);
    assert_ne!(&main_container[..8], b"EXTRHEAD");
    assert!(!main_container.ends_with(b"EXTRFOOT"));
    let unpacked_main = unpack_extrhead(&main_container, ExtrheadCryptMode::Auto).unwrap();
    assert_eq!(unpacked_main.header.build_name, "3410PQ");
    assert_eq!(unpacked_main.decompressed, main_plain);
    assert!(unpacked_main.checksum_valid);
    assert!(unpacked_main.footer_present);

    // 3. PANA_DVD.FRM (encrypted 0x30-byte TOC + modules) and PANA__SD.FRM (plaintext 0x40-byte TOC)
    let mut m_prog = FrmPackModule::new(
        "PROG061 ",
        vec![0xDC, 0x14, 0x00, 0x00, 0xAA, 0xBB, 0xCC, 0xDD],
    );
    m_prog.version_u16 = 0x1202;
    m_prog.target_flag = 1;
    let mut m_drv = FrmPackModule::new("DRV S211", drv_container.clone());
    m_drv.version_u16 = 0x975E;
    let mut m_main = FrmPackModule::new("MAIN3060NP", main_container.clone());
    m_main.version_u16 = 0x96AA;
    let modules = vec![m_drv, m_prog, m_main];

    let (dvd_frm, dvd_report) = pack_frm(&modules, FrmContainerMode::Dvd, None).unwrap();
    assert_ne!(&dvd_frm[..8], b"DRV S211");
    assert_eq!(dvd_report.modules.len(), 3);
    let dvd_toc = decrypt_and_parse_frm_toc(&dvd_frm);
    assert_eq!(dvd_toc.len(), 3);
    assert!(dvd_toc.iter().all(|e| e.list_sum_valid));
    assert_eq!(dvd_toc[0].module_offset, 0x1800);
    assert_eq!(dvd_toc[1].target_flag, 1);

    let mut dvd_dec = dvd_frm;
    feistel_decrypt_buffer(
        &mut dvd_dec,
        &PANA_FEISTEL_ROUND_KEY_32B,
        &PANA_FEISTEL_SBOX_256,
    );
    for (entry, spec) in dvd_toc.iter().zip(modules.iter()) {
        let start = entry.module_offset as usize;
        let end = start + entry.module_size as usize;
        let sum = dvd_dec[start..end]
            .iter()
            .fold(0u32, |acc, &b| acc.wrapping_add(u32::from(b)));
        assert_eq!(sum, entry.fvu_checksum_u32);
        assert!(dvd_dec[start..end].starts_with(&spec.payload));
    }

    let (sd_frm, sd_report) = pack_frm(&modules, FrmContainerMode::Sd, None).unwrap();
    assert!(sd_frm.starts_with(b"DRV S211"));
    assert_eq!(sd_report.toc_region_size, 0x2000);
    let sd_toc = parse_sd_frm_toc(&sd_frm);
    assert_eq!(sd_toc.len(), 3);
    assert!(sd_toc.iter().all(|e| e.list_sum_valid));
    assert_eq!(sd_toc[0].module_offset, 0x2000);
    let sd_main_start = sd_toc[2].module_offset as usize;
    let sd_main_end = sd_main_start + sd_toc[2].module_size as usize;
    assert_eq!(
        &sd_frm[sd_main_start..sd_main_end],
        main_container.as_slice()
    );

    let sd_path = std::env::temp_dir().join("panadiga_test_sd_toc_dump.frm");
    std::fs::write(&sd_path, &sd_frm).unwrap();
    let sd_dump = panadiga_core::crypto::toc::dump_toc(&sd_path, None);
    let _ = std::fs::remove_file(&sd_path);
    assert_eq!(sd_dump.unwrap().file_size_bytes, sd_frm.len());
}
