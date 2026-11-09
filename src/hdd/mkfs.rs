//! Procedural MEIHDFS (Matsushita High-Density File System) Synthesizer / mkfs
//!
//! Generates valid, mathematically exact Panasonic DIGA disk layouts from first principles
//! WITHOUT requiring copyrighted Panasonic firmware binary dumps:
//! - Master MBR Superblock (LBA 0) with signature "SH"
//! - Inverted Allocation Descriptors table (LBA 24..31)
//! - Transactional Journal Catalog (LBA 16384)
//! - Redundant Mirror MBR (LBA 57344) and Mirror Descriptors (LBA 57368)
//! - Dynamically sized Partition 3 (User Video Area) calculated for ANY drive size (80GB to 2TB+)

use crate::constants::*;
use anyhow::{Result, bail};
use byteorder::{ByteOrder, LittleEndian};

/// Holds the procedurally generated MEIHDFS structures in memory.
#[derive(Debug, Clone)]
pub struct MeihdfsImage {
    pub total_sectors: u64,
    pub volume_guid: [u8; 16],
    pub mbr_lba0: [u8; LBA_SECTOR_SIZE_BYTES],
    pub descriptors_lba24: [u8; ALLOCATION_DESCRIPTORS_SIZE],
    pub journal_header_lba16384: [u8; LBA_SECTOR_SIZE_BYTES],
    pub mirror_mbr_lba57344: [u8; LBA_SECTOR_SIZE_BYTES],
    pub mirror_descriptors_lba57368: [u8; ALLOCATION_DESCRIPTORS_SIZE],
    pub p1_lba_start: u64,
    pub p1_sectors: u64,
    pub p2_lba_start: u64,
    pub p2_sectors: u64,
    pub p3_lba_start: u64,
    pub p3_sectors: u64,
}

/// Generates a clean procedural MEIHDFS filesystem image for a target sector count.
pub fn generate_meihdfs_layout(
    total_sectors: u64,
    custom_guid: Option<[u8; 16]>,
) -> Result<MeihdfsImage> {
    const MIN_SECTORS: u64 = 200_000; // ~102 MB minimum required for system partitions
    if total_sectors < MIN_SECTORS {
        bail!(
            "Target drive size too small for MEIHDFS: {} sectors (minimum {} sectors / ~102 MB required)",
            total_sectors,
            MIN_SECTORS
        );
    }

    let volume_guid = custom_guid.unwrap_or([
        0x62, 0x8C, 0x6E, 0xF5, 0x1B, 0x2C, 0x36, 0x66, 0xAA, 0x55, 0x00, 0x16, 0x4D, 0x45, 0x49,
        0x48,
    ]);

    let p1_start = LBA_PARTITION_1_SECTOR; // 65536 (32 MB)
    let p1_sectors = 65536u64; // 32 MB
    let p2_start = LBA_PARTITION_2_SECTOR; // 131072 (64 MB)
    let p2_sectors = 65536u64; // 32 MB
    let p3_start = 196608u64; // 96 MB
    let p3_sectors = total_sectors.saturating_sub(p3_start);

    // 1. Generate LBA 0 (Master MBR Superblock)
    let mut mbr = [0u8; LBA_SECTOR_SIZE_BYTES];
    // Signature prefix: 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x53 ('S'), 0x48 ('H')
    mbr[0..8].copy_from_slice(&PANASONIC_HDD_MBR_SIGNATURE);
    // Volume GUID
    mbr[8..24].copy_from_slice(&volume_guid);
    // Geometry & Layout Pointers
    LittleEndian::write_u64(&mut mbr[24..32], total_sectors);
    LittleEndian::write_u64(&mut mbr[32..40], LBA_MIRROR_MBR_SECTOR);
    LittleEndian::write_u64(&mut mbr[40..48], LBA_JOURNAL_CATALOG_OFFSET / 512);
    LittleEndian::write_u64(&mut mbr[48..56], ALLOCATION_DESCRIPTORS_OFFSET / 512);

    // Partitions table
    LittleEndian::write_u64(&mut mbr[56..64], p1_start);
    LittleEndian::write_u64(&mut mbr[64..72], p1_sectors);
    LittleEndian::write_u64(&mut mbr[72..80], p2_start);
    LittleEndian::write_u64(&mut mbr[80..88], p2_sectors);
    LittleEndian::write_u64(&mut mbr[88..96], p3_start);
    LittleEndian::write_u64(&mut mbr[96..104], p3_sectors);

    // 2. Generate Allocation Descriptors (256 records of 16 bytes, bitwise inverted ^ 0xFF)
    let mut descriptors = [0u8; ALLOCATION_DESCRIPTORS_SIZE];
    // Helper closure to set an inverted record
    let mut write_record = |idx: usize, rec: &[u8; 16]| {
        let base = idx * 16;
        for j in 0..16 {
            descriptors[base + j] = rec[j] ^ 0xFF;
        }
    };

    // Partition 1 Extent Header (Tag 0x5D, ID 0x0001) & Pointer (Tag 0x5E)
    let mut p1_hdr = [0u8; 16];
    p1_hdr[0] = MEIHDFS_TAG_EXTENT_HEADER;
    p1_hdr[1] = 0x01; // Subtag
    LittleEndian::write_u16(&mut p1_hdr[2..4], 1); // Extent ID 1
    LittleEndian::write_u16(&mut p1_hdr[4..6], 0x0001); // Flags
    write_record(0, &p1_hdr);

    let mut p1_ptr = [0u8; 16];
    p1_ptr[0] = MEIHDFS_TAG_EXTENT_POINTER;
    p1_ptr[1] = 0x00;
    LittleEndian::write_u16(&mut p1_ptr[2..4], 1);
    LittleEndian::write_u16(&mut p1_ptr[4..6], (p1_sectors & 0xFFFF) as u16);
    LittleEndian::write_u16(&mut p1_ptr[6..8], (p1_start & 0xFFFF) as u16);
    write_record(1, &p1_ptr);

    // Partition 2 Extent Header (Tag 0x5D, ID 0x0002) & Pointer (Tag 0x5E)
    let mut p2_hdr = [0u8; 16];
    p2_hdr[0] = MEIHDFS_TAG_EXTENT_HEADER;
    p2_hdr[1] = 0x02;
    LittleEndian::write_u16(&mut p2_hdr[2..4], 2);
    LittleEndian::write_u16(&mut p2_hdr[4..6], 0x0001);
    write_record(2, &p2_hdr);

    let mut p2_ptr = [0u8; 16];
    p2_ptr[0] = MEIHDFS_TAG_EXTENT_POINTER;
    p2_ptr[1] = 0x00;
    LittleEndian::write_u16(&mut p2_ptr[2..4], 2);
    LittleEndian::write_u16(&mut p2_ptr[4..6], (p2_sectors & 0xFFFF) as u16);
    LittleEndian::write_u16(&mut p2_ptr[6..8], (p2_start & 0xFFFF) as u16);
    write_record(3, &p2_ptr);

    // Partition 3 Extent Header (Tag 0x5D, ID 0x0003) & Pointer (Tag 0x5E)
    let mut p3_hdr = [0u8; 16];
    p3_hdr[0] = MEIHDFS_TAG_EXTENT_HEADER;
    p3_hdr[1] = 0x03;
    LittleEndian::write_u16(&mut p3_hdr[2..4], 3);
    LittleEndian::write_u16(&mut p3_hdr[4..6], 0x0001);
    write_record(4, &p3_hdr);

    let mut p3_ptr = [0u8; 16];
    p3_ptr[0] = MEIHDFS_TAG_EXTENT_POINTER;
    p3_ptr[1] = 0x00;
    LittleEndian::write_u16(&mut p3_ptr[2..4], 3);
    LittleEndian::write_u16(&mut p3_ptr[4..6], (p3_sectors.min(0xFFFF)) as u16);
    LittleEndian::write_u16(&mut p3_ptr[6..8], (p3_start & 0xFFFF) as u16);
    write_record(5, &p3_ptr);

    // Records 6..255 are free unallocated slots (Tag 0xFF in clear -> 0x00 on disk)
    for i in 6..MEIHDFS_DESCRIPTOR_RECORDS_COUNT {
        let mut free_rec = [0xFFu8; 16];
        free_rec[0] = MEIHDFS_TAG_FREE;
        write_record(i, &free_rec);
    }

    // 3. Generate Transactional Journal Header (LBA 16384)
    let mut journal_hdr = [0u8; LBA_SECTOR_SIZE_BYTES];
    journal_hdr[0..4].copy_from_slice(b"MJRN"); // Matsushita Journal Magic
    LittleEndian::write_u32(&mut journal_hdr[4..8], 1); // Clean journal version
    LittleEndian::write_u32(&mut journal_hdr[8..12], JOURNAL_BLOCK_COUNT as u32);

    // 4. Mirror MBR (LBA 57344) and Mirror Descriptors (LBA 57368) - Bit-for-bit replicas
    let mirror_mbr = mbr;
    let mirror_desc = descriptors;

    Ok(MeihdfsImage {
        total_sectors,
        volume_guid,
        mbr_lba0: mbr,
        descriptors_lba24: descriptors,
        journal_header_lba16384: journal_hdr,
        mirror_mbr_lba57344: mirror_mbr,
        mirror_descriptors_lba57368: mirror_desc,
        p1_lba_start: p1_start,
        p1_sectors,
        p2_lba_start: p2_start,
        p2_sectors,
        p3_lba_start: p3_start,
        p3_sectors,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_procedural_meihdfs_synthesis_minimum_drive() {
        let sectors = 312_581_808u64; // ~160 GB drive
        let img = generate_meihdfs_layout(sectors, None).expect("Should generate layout");

        // Verify MBR Signature
        assert_eq!(&img.mbr_lba0[0..8], &PANASONIC_HDD_MBR_SIGNATURE);
        // Verify Mirror MBR is 100% bit-identical
        assert_eq!(&img.mbr_lba0[..], &img.mirror_mbr_lba57344[..]);
        // Verify Mirror Descriptors are 100% bit-identical
        assert_eq!(
            &img.descriptors_lba24[..],
            &img.mirror_descriptors_lba57368[..]
        );

        // Verify Partition 3 expands dynamically
        assert_eq!(img.p3_lba_start, 196608);
        assert_eq!(img.p3_sectors, sectors - 196608);

        // Verify first descriptor record is bitwise inverted extent header
        let raw_tag = img.descriptors_lba24[0];
        let clear_tag = raw_tag ^ 0xFF;
        assert_eq!(clear_tag, MEIHDFS_TAG_EXTENT_HEADER);
    }

    #[test]
    fn test_procedural_meihdfs_rejects_undersized_drive() {
        let sectors = 50_000u64; // Too small
        let res = generate_meihdfs_layout(sectors, None);
        assert!(res.is_err());
    }
}
