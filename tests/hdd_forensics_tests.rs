use panadiga::hdd::{decode_descriptors, decode_meihdfs_descriptors, inspect_hdd_image};
use std::path::Path;

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
fn test_real_hdd_decode_descriptors() {
    let hdd_path = Path::new("firmware_and_images/panasonic DMR EH-55 Firmware hdd_100.bin");
    if !hdd_path.exists() {
        return;
    }
    assert!(decode_descriptors(hdd_path).is_ok());
}

#[test]
fn test_decode_meihdfs_descriptors_standalone() {
    let desc_path = Path::new(
        "firmware_and_images/extracted_partitions/hdd_allocation_descriptors_0x00003000.bin",
    );
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

    let dist = &report.tag_distribution;
    assert_eq!(dist.get("0x5D (ExtentHeader)").copied().unwrap_or(0), 65);
    assert_eq!(dist.get("0x55 (ExtentPointer)").copied().unwrap_or(0), 64);
    assert_eq!(
        dist.get("0x57 (ExtentContinuation)").copied().unwrap_or(0),
        47
    );
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
