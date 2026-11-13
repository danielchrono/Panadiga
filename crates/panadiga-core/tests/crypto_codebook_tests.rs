use panadiga_core::crypto::{analyze_frm_container, dump_toc, print_codebook};
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
