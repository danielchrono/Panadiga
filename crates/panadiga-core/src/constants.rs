//! Panasonic DMR-EH55 System Architecture Constants
//!
//! Strongly typed, exhaustive definition of all memory boundaries, flash microsectors,
//! disk LBA offsets, partition headers, and cryptographic ECB leak signatures.
//! ZERO magic numbers allowed anywhere in the codebase.

// ============================================================================
// 1. Spansion S29JL064H NOR Flash (8 MB / 64 Mbit, TSOP-48) Memory Layout
// ============================================================================

/// Total raw size of the Spansion S29JL064H NOR Flash (8 Megabytes / 64 Mbits).
pub const SPANSION_FLASH_SIZE_BYTES: usize = 8 * 1024 * 1024; // 8,388,608 bytes

/// Boundary separating the OS / Bootloader payload from the top NVRAM sector.
/// The Bootloader and system software reside in 0x0000_0000 .. 0x007E_FFFF (8,323,072 bytes).
pub const SPANSION_BOOTLOADER_SIZE_BYTES: usize = 0x007F_0000; // 8,323,072 bytes

/// Base memory offset where the top NVRAM/EEPROM emulation sector begins in physical flash.
pub const SPANSION_NVRAM_BASE_OFFSET: usize = 0x007F_0000;

/// Total size of the NVRAM parameter storage area (top 64 Kilobytes).
pub const SPANSION_NVRAM_SIZE_BYTES: usize = 64 * 1024; // 65,536 bytes

/// Total size of the NVRAM parameter storage area for 16 MB NOR Flash models (top 128 Kilobytes).
pub const FLASH_16MB_NVRAM_SIZE_BYTES: usize = 128 * 1024; // 131,072 bytes

/// Total count of 8 KB micro-sectors in the 16 MB NOR Flash top NVRAM region (16 microsectors).
pub const FLASH_16MB_MICROSECTOR_COUNT: usize = 16;

/// Base memory offset where the 128 KB NVRAM begins in a 16 MB NOR Flash image.
pub const FLASH_16MB_NVRAM_BASE_OFFSET: usize = 0x00FE_0000;

/// End memory offset of the NVRAM area (must equal `SPANSION_FLASH_SIZE_BYTES`).
pub const SPANSION_NVRAM_END_OFFSET: usize = SPANSION_NVRAM_BASE_OFFSET + SPANSION_NVRAM_SIZE_BYTES;

/// Size of each individual hardware micro-sector within the top 64 KB bank.
/// In the Spansion top-boot architecture (Model S29JL064H), the top 64 KB sector
/// is subdivided into 8 protected microsectors of 8 KB each.
pub const SPANSION_MICROSECTOR_SIZE: usize = 8 * 1024; // 8,192 bytes

/// Total count of 8 KB micro-sectors in the top 64 KB sector.
pub const SPANSION_MICROSECTOR_COUNT: usize = 8;

/// Base sector index according to Spansion S29JL064H datasheet nomenclature.
/// Sectors SA134 through SA141 correspond to offsets 0x007F_0000 through 0x007F_FFFF.
pub const SPANSION_MICROSECTOR_BASE_INDEX: usize = 134;

/// Erased state byte for NOR flash memory cells (all bits high).
pub const FLASH_ERASED_BYTE: u8 = 0xFF;

// ============================================================================
// 2. Hard Disk Drive (HDD) Geometry, MEIHDFS Filesystem & Partition Signatures
// ============================================================================

/// Standard ATA/IDE sector size in bytes (LBA addressing unit).
pub const LBA_SECTOR_SIZE_BYTES: usize = 512;

/// Panasonic proprietary MEIHDFS MBR signature prefix (8 bytes).
/// Bytes 0x00-0x03: 0x00000000 (Reserved)
/// Bytes 0x04-0x05: 0x0100 (Format Revision 1.0, little-endian)
/// Bytes 0x06-0x07: 0x53, 0x48 ("SH" in ASCII -> System Storage Header)
pub const PANASONIC_HDD_MBR_SIGNATURE: [u8; 8] = [0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x53, 0x48];

/// Full 16-byte header seen on DMR-EH55 LBA 0 (including Disk Volume GUID).
pub const PANASONIC_HDD_MBR_FULL_SIGNATURE: [u8; 16] = [
    0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x53, 0x48, 0x62, 0x8C, 0x6E, 0xF5, 0x1B, 0x2C, 0x36, 0x66,
];

/// Standard IBM PC MBR boot signature (0x55AA at offsets 510..512).
/// Panasonic MEIHDFS raw drives deliberately lack this signature to prevent PC OS corruption.
pub const STANDARD_MBR_BOOT_SIGNATURE: [u8; 2] = [0x55, 0xAA];

/// Proprietary Panasonic Partition Header Magic (8 bytes).
/// Marks the starting boundary of embedded firmware images and system containers.
pub const PANASONIC_HDD_PARTITION_MAGIC: [u8; 8] = [0x51, 0x87, 0x13, 0x8D, 0x5B, 0x0A, 0xFB, 0x22];

// ============================================================================
// 3. Disk Layout Offsets, Mirror Tables & Partition Dimensions
// ============================================================================

/// Offset of Allocation Descriptors table (LBA 24 to 31, 8 sectors = 4096 bytes).
pub const ALLOCATION_DESCRIPTORS_OFFSET: u64 = 0x0000_3000;
pub const ALLOCATION_DESCRIPTORS_SIZE: usize = 4096;

/// Mirror offset of Allocation Descriptors (LBA 57368 to 57375).
pub const ALLOCATION_DESCRIPTORS_MIRROR_OFFSET: u64 = 0x01C0_3000;

/// Offset of the Transactional Journal Catalog (LBA 16384 / 8.00 MB).
pub const LBA_JOURNAL_CATALOG_OFFSET: u64 = 0x0080_0000;

/// Start offset of the Journal region (0x0082_0000 to 0x0096_0000 = 1,310,720 bytes).
pub const JOURNAL_AREA_OFFSET: u64 = 0x0082_0000;
pub const JOURNAL_AREA_END_OFFSET: u64 = 0x0096_0000;
pub const JOURNAL_AREA_SIZE: usize = (JOURNAL_AREA_END_OFFSET - JOURNAL_AREA_OFFSET) as usize; // 1,310,720 bytes
pub const JOURNAL_BLOCK_SIZE: usize = 128 * 1024; // 131,072 bytes (256 sectors)
pub const JOURNAL_BLOCK_COUNT: usize = 10;

/// State transition marker / boundary sector at 24.00 MB (LBA 49152).
pub const LBA_STATE_TRANSITION_MARKER_OFFSET: u64 = 0x0180_0000;

/// Mirror MBR backup table offset (28.00 MB, LBA 57344).
/// Sector 57344 contains a bit-for-bit identical replica of Sector 0.
pub const LBA_MIRROR_MBR_OFFSET: u64 = 0x01C0_0000;
pub const LBA_MIRROR_MBR_SECTOR: u64 = 57344;

/// Partition 1: Main System Firmware, Codecs & AM34 OS Container (32.00 MB).
pub const LBA_PARTITION_1_OFFSET: u64 = 0x0200_0000;
pub const LBA_PARTITION_1_SECTOR: u64 = 65536;

/// Exact payload size of Partition 1 (6,173,264 bytes / ~5.887 MB).
pub const LBA_PARTITION_1_PAYLOAD_SIZE: usize = 0x005E_3250;

/// Sector-aligned size of Partition 1 (6,173,696 bytes / 12,058 sectors of 512 bytes).
pub const LBA_PARTITION_1_ALIGNED_SIZE: usize = 0x005E_3400;

/// Partition 2: Secondary Firmware, Microcode, Sub-system & OSD Fonts (64.00 MB).
pub const LBA_PARTITION_2_OFFSET: u64 = 0x0400_0000;
pub const LBA_PARTITION_2_SECTOR: u64 = 131072;

/// Exact payload size of Partition 2 (720,896 bytes / 704 KB / 1,408 sectors).
pub const LBA_PARTITION_2_PAYLOAD_SIZE: usize = 0x000B_0000;

/// Pre-padding size of Partition 2 before trailing 48 padding bytes (720,848 bytes).
pub const LBA_PARTITION_2_PRE_PADDING_SIZE: usize = 0x000A_FFD0;

// ============================================================================
// 4. Cryptographic Primitives & ECB Leak Pattern Signatures
// ============================================================================

/// Native block size of the symmetric cipher applied to Panasonic firmware (64 bits).
pub const CIPHER_BLOCK_SIZE: usize = 8;

/// ECB Ciphertext Leak Pattern: 8 zero-bytes (`00 00 00 00 00 00 00 00`).
/// Appears in padding regions of encrypted partitions and in PANA_DVD.FRM (196 occurrences).
pub const ECB_NULL_BLOCK_8B: [u8; 8] = [0xE5, 0xD1, 0xF3, 0x4A, 0x51, 0x8B, 0x26, 0x90];

/// ECB Ciphertext Leak Pattern: 8 virgin flash bytes (`FF FF FF FF FF FF FF FF`).
/// Appears 3,812 times in PANA_DVD.FRM (over 30 KB of unprogrammed flash padding).
pub const ECB_VIRGIN_FLASH_8B: [u8; 8] = [0x03, 0x50, 0x65, 0xC8, 0x7B, 0xB8, 0xE0, 0x90];

/// ECB Ciphertext Leak Pattern: Common partition header repeated structure.
/// Appears 1,280 times in PANA_DVD.FRM.
pub const ECB_COMMON_STRUCT_8B: [u8; 8] = [0x4D, 0x3C, 0x5D, 0xBB, 0x0C, 0x35, 0x77, 0x77];

/// Size of each Table of Contents (TOC) entry descriptor in PANA_DVD.FRM (48 bytes).
pub const PANA_FRM_TOC_ENTRY_SIZE: usize = 0x30; // 48 bytes

/// Repeated 8-byte sub-block at TOC offsets 0x0010, 0x0040, 0x0070.
pub const PANA_FRM_TOC_SUBBLOCK_1: [u8; 8] = [0xEF, 0xDC, 0x1C, 0xDA, 0x13, 0x88, 0x1B, 0xAB];

/// Repeated 8-byte sub-block at TOC offsets 0x0018 and 0x0078.
pub const PANA_FRM_TOC_SUBBLOCK_2: [u8; 8] = [0x74, 0xA7, 0x8A, 0xCD, 0x78, 0x9B, 0x84, 0xFF];

/// Global 16-byte container header for PANA_DVD.FRM.
pub const PANA_FRM_GLOBAL_HEADER: [u8; 16] = [
    0x49, 0x2F, 0x05, 0x60, 0xCE, 0xDE, 0x50, 0x8C, 0xE3, 0xC0, 0x8A, 0xFD, 0x49, 0xB6, 0x4A, 0x9A,
];

/// Global container header size in PANA_DVD.FRM (16 bytes = 0x10).
pub const PANA_FRM_GLOBAL_HEADER_SIZE: usize = 0x10;

/// Total number of Table of Contents (TOC) entries in PANA_DVD.FRM.
pub const PANA_FRM_TOC_ENTRY_COUNT: usize = 3;

/// Total size of the Table of Contents (3 entries * 48 bytes = 144 bytes / 0x90).
pub const PANA_FRM_TOC_TOTAL_SIZE: usize = PANA_FRM_TOC_ENTRY_COUNT * PANA_FRM_TOC_ENTRY_SIZE;

/// Target descriptor for Optical Drive Servo MN102/DSP at TOC Entry 1 (offset 0x0048).
pub const PANA_FRM_TOC_DRIVE_TARGET_8B: [u8; 8] = [0x3B, 0x7E, 0x7E, 0xDB, 0xB6, 0xC5, 0x7A, 0x53];

/// Target descriptor for AM34 / MN103x RISC at TOC Entries 0 and 2 (offsets 0x0018, 0x0078).
pub const PANA_FRM_TOC_AM34_TARGET_8B: [u8; 8] = [0x74, 0xA7, 0x8A, 0xCD, 0x78, 0x9B, 0x84, 0xFF];

/// Offset where the encrypted payload begins in PANA_DVD.FRM (0x00A0).
pub const PANA_FRM_PAYLOAD_OFFSET: usize = 0x00A0;

/// Feistel cipher key part 1 from `BOOT.BIN` `.data` (`0x50480278`, file offset `0x39AA8`).
pub const PANA_FEISTEL_KEY_PART1: [u8; 7] = [0x72, 0xEB, 0xCA, 0x91, 0x30, 0x45, 0x68];

/// Feistel cipher key part 2 from `BOOT.BIN` `.data` (`0x5048027F`, file offset `0x39AAF`).
pub const PANA_FEISTEL_KEY_PART2: [u8; 8] = [0xAD, 0x99, 0xDF, 0x56, 0x72, 0x3A, 0xB8, 0x10];

/// Reconstructed 7-byte master key (`ab[i] = p2[i] ^ p1[i]`, with `p2[7]` XORed into indices 1 and 4).
pub const PANA_FEISTEL_MASTER_KEY_7B: [u8; 7] = [0xDF, 0x62, 0x15, 0xC7, 0x52, 0x7F, 0xD0];

/// 4-byte IV used by `FUN_44011d42` to expand the 7-byte master key into the 32-byte round key.
pub const PANA_FEISTEL_KEYSCHED_IV: [u8; 4] = [0x35, 0x7D, 0xFB, 0x9A];

/// Pre-expanded 32-byte Feistel round key (`FUN_44011d42`) for EH55/EH58 `.FRM` and HDD partitions.
pub const PANA_FEISTEL_ROUND_KEY_32B: [u8; 32] = [
    0xA3, 0xE7, 0x9C, 0x01, 0xB2, 0x5A, 0x54, 0x74, 0x59, 0x1D, 0xB2, 0xEF, 0x67, 0x26, 0x6D, 0xFF,
    0x63, 0x03, 0x87, 0x07, 0xB6, 0x56, 0x94, 0x0F, 0x02, 0x6D, 0xA5, 0x61, 0x25, 0x51, 0xF3, 0x15,
];

/// Official 256-byte substitution table from `BOOT.BIN` (`0x44035D84`) / `PROG061_BOOT` (`0x50435C0A`).
pub const PANA_FEISTEL_SBOX_256: [u8; 256] = [
    0x00, 0x01, 0xBB, 0x9C, 0x3F, 0x5E, 0xC7, 0x3E, 0x36, 0xA4, 0x92, 0x93, 0x38, 0x9B, 0x8D, 0x1A,
    0x3C, 0x84, 0xF7, 0x2A, 0xCB, 0x3D, 0x70, 0xAD, 0x30, 0xA2, 0xC1, 0x95, 0x03, 0x39, 0xA5, 0x0E,
    0xE6, 0xFB, 0x09, 0x1E, 0xB9, 0xC4, 0x24, 0x57, 0x5F, 0xC3, 0x5D, 0xFA, 0x60, 0x56, 0xFD, 0xED,
    0x81, 0xF3, 0x4A, 0xFF, 0x85, 0xDB, 0xCD, 0xA0, 0xD0, 0x9D, 0x8B, 0x26, 0x4C, 0x50, 0x0C, 0x1D,
    0x7D, 0x77, 0xDE, 0x7A, 0x0A, 0x0B, 0x73, 0xE1, 0x54, 0x76, 0xE8, 0xC2, 0x28, 0x05, 0x98, 0x99,
    0x29, 0x4B, 0xEE, 0x87, 0x42, 0x59, 0x65, 0xB8, 0x1F, 0xDF, 0x23, 0x58, 0x63, 0x69, 0x1C, 0x79,
    0x64, 0xC5, 0x6F, 0x13, 0x3B, 0xCE, 0x08, 0xA7, 0xB2, 0x12, 0x20, 0x7F, 0xE2, 0x89, 0x21, 0x2B,
    0x41, 0x49, 0x7C, 0x82, 0xBE, 0xF2, 0x43, 0x8E, 0x86, 0x02, 0x9E, 0xD8, 0x67, 0x8F, 0xA3, 0xE7,
    0xBC, 0xF8, 0x66, 0x22, 0x4D, 0xE5, 0xBA, 0xD2, 0xDA, 0xEA, 0x61, 0x25, 0xB0, 0xC0, 0x7B, 0xA6,
    0x48, 0x96, 0xDD, 0xDC, 0x71, 0x90, 0x55, 0x5B, 0x4F, 0x4E, 0x6D, 0xD1, 0x11, 0x34, 0xAA, 0x1B,
    0xF4, 0x17, 0x80, 0xB7, 0xCC, 0x8C, 0xD9, 0x37, 0x8A, 0x2C, 0x94, 0x16, 0x72, 0x40, 0xEF, 0x9A,
    0xC8, 0x47, 0xF6, 0xFC, 0x2E, 0xB3, 0x2F, 0xBD, 0xCF, 0xA9, 0x15, 0x10, 0x18, 0xEC, 0x6A, 0x6E,
    0xC9, 0x91, 0x53, 0x78, 0xA8, 0x3A, 0xAF, 0x0D, 0xE0, 0x97, 0x32, 0x51, 0xB1, 0xF5, 0x27, 0x2D,
    0x35, 0x88, 0x14, 0xE3, 0xFE, 0x06, 0xD7, 0xD6, 0xAB, 0xA1, 0xD5, 0xE4, 0x45, 0x44, 0x9F, 0x0F,
    0x5A, 0x83, 0xEB, 0xCA, 0x07, 0x04, 0xB4, 0xB6, 0x52, 0xAE, 0xD4, 0x33, 0x31, 0x75, 0xD3, 0x6C,
    0x62, 0xBF, 0x6B, 0xF9, 0xAC, 0x46, 0xF0, 0x5C, 0x19, 0x7E, 0x68, 0xB5, 0xF1, 0x74, 0xC6, 0xE9,
];

/// Size of the `"Firm list"` TOC header region at the beginning of `PANA_DVD.FRM` (`0x1800` / 6,144 B).
pub const PANA_FRM_TOC_REGION_SIZE: usize = 0x1800;

/// Exact offset and size of the `DRV S211` module inside decrypted `PANA_DVD.FRM`.
pub const PANA_FRM_DRV_S211_OFFSET: usize = 0x0000_1800;
pub const PANA_FRM_DRV_S211_SIZE: usize = 0x000B_0000;

/// Exact offset and size of the `PROG061 ` bootloader module inside decrypted `PANA_DVD.FRM`.
pub const PANA_FRM_PROG061_OFFSET: usize = 0x000B_1800;
pub const PANA_FRM_PROG061_SIZE: usize = 0x0004_0000;

/// Exact offset and size of the `MAIN3060NP` system firmware module inside decrypted `PANA_DVD.FRM`.
pub const PANA_FRM_MAIN3060_OFFSET: usize = 0x000F_1800;
pub const PANA_FRM_MAIN3060_SIZE: usize = 0x0063_D4E0;

/// Encrypted prefix/suffix span applied to `MAIN` `EXTRHEAD`/`EXTRFOOT` containers (`0x19000` / 102,400 B).
pub const PANA_EXTRHEAD_MAIN_CRYPT_SPAN: usize = 0x0001_9000;

/// Encrypted prefix span applied to `DRV` `EXTRHEAD` containers (`0x2800` / 10,240 B).
pub const PANA_EXTRHEAD_DRV_CRYPT_SPAN: usize = 0x0000_2800;

/// Default sampling stride for Panasonic's strided Adler-32 routine (`FUN_4401e5a3`).
pub const PANA_EXTRHEAD_DEFAULT_ADLER_STEP: usize = 16;

// ============================================================================
// 5. MEIHDFS Low-Level Allocation Descriptors & Record Tags (XOR 0xFF Decoded)
// ============================================================================

/// Size of each structured allocation record in bytes (16 bytes).
pub const MEIHDFS_RECORD_SIZE: usize = 16;

/// Total number of 16-byte records in the 4096-byte Allocation Descriptors table (256 records).
pub const MEIHDFS_DESCRIPTOR_RECORDS_COUNT: usize = 256;

/// MEIHDFS Allocation Record Tag 0x5D: Extent Descriptor Header / Node Header.
/// Raw byte on disk: 0xA2 (0xA2 ^ 0xFF = 0x5D).
pub const MEIHDFS_TAG_EXTENT_HEADER: u8 = 0x5D;

/// MEIHDFS Allocation Record Tag 0x55: Extent Segment Pointer / Allocation Map Entry.
/// Raw byte on disk: 0xAA (0xAA ^ 0xFF = 0x55).
pub const MEIHDFS_TAG_EXTENT_POINTER: u8 = 0x55;

/// MEIHDFS Allocation Record Tag 0x57: Extent Continuation Pointer / Indirect Block.
/// Raw byte on disk: 0xA8 (0xA8 ^ 0xFF = 0x57).
pub const MEIHDFS_TAG_EXTENT_CONTINUATION: u8 = 0x57;

/// MEIHDFS Allocation Record Tag 0x5F: Extent Pool Boundary / Volume Limiter.
/// Raw byte on disk: 0xA0 (0xA0 ^ 0xFF = 0x5F).
pub const MEIHDFS_TAG_EXTENT_BOUNDARY: u8 = 0x5F;

/// MEIHDFS Allocation Record Tag 0xE0: Virtual Node Table Entry / Inode Attributes.
/// Raw byte on disk: 0x1F (0x1F ^ 0xFF = 0xE0).
pub const MEIHDFS_TAG_VNODE_ENTRY: u8 = 0xE0;

/// MEIHDFS Allocation Record Tag 0xE1: Virtual Inode Directory Link / Hierarchical Index.
/// Raw byte on disk: 0x1E (0x1E ^ 0xFF = 0xE1).
pub const MEIHDFS_TAG_VINODE_LINK: u8 = 0xE1;

/// MEIHDFS Allocation Record Tag 0xFF: Unallocated / Free record slot.
/// Raw byte on disk: 0x00 (0x00 ^ 0xFF = 0xFF).
pub const MEIHDFS_TAG_FREE: u8 = 0xFF;

/// Relative offset within each 128 KB Journal block where the transaction commit trailer begins (0x12C0C).
pub const JOURNAL_TRANSACTION_TRAILER_OFFSET: usize = 0x12C0C;

/// Relative offset of the 32-bit transaction sequence ID in each 128 KB Journal block (0x12C14).
pub const JOURNAL_TRANSACTION_SEQ_OFFSET: usize = 0x12C14;

/// Module 1 (Sub-system / OSD) start offset in PANA_DVD.FRM (0x00A0).
pub const PANA_FRM_MOD1_START_OFFSET: usize = 0x00A0;
/// Module 1 active payload end offset (0x0AE480).
pub const PANA_FRM_MOD1_PAYLOAD_END: usize = 0x0AE480;
/// Module 1 virgin flash padding end offset (0x0AEFD0).
pub const PANA_FRM_MOD1_VIRGIN_PAD_END: usize = 0x0AEFD0;
/// Module 1 fixed partition structure end offset (0x0B17D0).
pub const PANA_FRM_MOD1_STRUCT_END: usize = 0x0B17D0;
/// Module 1 boundary offset (0x0B1800).
pub const PANA_FRM_MOD1_BOUNDARY: usize = 0x0B1800;

/// Module 2 (Drive Servo DSP) start offset in PANA_DVD.FRM (0x0B1800).
pub const PANA_FRM_MOD2_START_OFFSET: usize = 0x0B1800;
/// Module 2 active payload end offset (0x0EAC98).
pub const PANA_FRM_MOD2_PAYLOAD_END: usize = 0x0EAC98;
/// Module 2 virgin flash padding end offset (0x0F17F8).
pub const PANA_FRM_MOD2_VIRGIN_PAD_END: usize = 0x0F17F8;
/// Module 2 boundary offset (0x0F1818).
pub const PANA_FRM_MOD2_BOUNDARY: usize = 0x0F1818;

/// Module 3 (Main CPU OS AM34 / DIGA OS) start offset in PANA_DVD.FRM (0x0F1818).
pub const PANA_FRM_MOD3_START_OFFSET: usize = 0x0F1818;

// ============================================================================
// 6. Backward Compatibility Aliases for MEIHDFS Allocation Tags
// ============================================================================

/// Total count of 16-byte records in the 4096-byte allocation table (LBAs 24..31).
pub const MEIHDFS_RECORD_COUNT: usize = ALLOCATION_DESCRIPTORS_SIZE / MEIHDFS_RECORD_SIZE; // 256 records

pub const MEIHDFS_TAG_EXTENT_5D: u8 = MEIHDFS_TAG_EXTENT_HEADER;
pub const MEIHDFS_TAG_SYNC_55: u8 = MEIHDFS_TAG_EXTENT_POINTER;
pub const MEIHDFS_TAG_POINTER_57: u8 = MEIHDFS_TAG_EXTENT_CONTINUATION;
pub const MEIHDFS_TAG_DESCRIPTOR_5F: u8 = MEIHDFS_TAG_EXTENT_BOUNDARY;
pub const MEIHDFS_TAG_DIR_ENTRY_E0: u8 = MEIHDFS_TAG_VNODE_ENTRY;
pub const MEIHDFS_TAG_VIRTUAL_NODE_E1: u8 = MEIHDFS_TAG_VINODE_LINK;

// ============================================================================
// 7. AM34 / UniPhier Hardware Bus Regions & Peripheral Addresses (Ghidra SSOT)
// ============================================================================

/// AM34 Internal Work SRAM Base Physical Address (0x0000_0000).
pub const AM34_INTERNAL_SRAM_BASE: u32 = 0x0000_0000;
/// AM34 Internal Work SRAM Size (256 Kilobytes).
pub const AM34_INTERNAL_SRAM_SIZE: u32 = 0x0004_0000;
pub const AM34_SRAM_BASE: u32 = AM34_INTERNAL_SRAM_BASE;
pub const AM34_SRAM_SIZE: u32 = AM34_INTERNAL_SRAM_SIZE;

/// RAM Loader Injection Physical Address (0x0000_4000).
pub const AM34_SRAM_LOADER_BASE: u32 = 0x0000_4000;
/// RAM Loader Memory Size (4 Kilobytes).
pub const AM34_SRAM_LOADER_SIZE: u32 = 0x0000_1000;

/// Core MMIO Peripheral Bus Space (0x1000_0000).
pub const AM34_MMIO_BASE: u32 = 0x1000_0000;
/// Core MMIO Space Size (64 Kilobytes).
pub const AM34_MMIO_SIZE: u32 = 0x0001_0000;

/// External System SDRAM Base Address (0x4000_0000).
pub const AM34_SDRAM_BASE: u32 = 0x4000_0000;
/// External System SDRAM Size (64 Megabytes).
pub const AM34_SDRAM_SIZE: u32 = 0x0400_0000;

/// Spansion NOR Flash Hardware Mirror Base Address (0x8000_0000).
pub const AM34_FLASH_BASE: u32 = 0x8000_0000;
pub const AM34_FLASH_BUS_BASE: u32 = AM34_FLASH_BASE;
/// Spansion NOR Flash Total Memory Aperture (8 Megabytes).
pub const AM34_FLASH_SIZE: u32 = 0x0080_0000;
/// Spansion NOR Flash Protected IPL / Kernel Size (SA0..SA133 = 8,128 Kilobytes).
pub const AM34_FLASH_IPL_SIZE: u32 = 0x007F_0000;

/// NVRAM Top-Boot Partition Base Physical Address (0x807F_0000).
pub const AM34_NVRAM_BASE: u32 = 0x807F_0000;
pub const AM34_NVRAM_BUS_BASE: u32 = AM34_NVRAM_BASE;
/// NVRAM Parameter Storage Area Size (64 Kilobytes).
pub const AM34_NVRAM_SIZE: u32 = 0x0001_0000;

/// Panasonic UniPhier ASIC MMIO Base Physical Address (0xC000_0000).
pub const AM34_ASIC_MMIO_BASE: u32 = 0xC000_0000;
/// UniPhier System MMIO Space Size (64 Kilobytes).
pub const AM34_ASIC_MMIO_SIZE: u32 = 0x0001_0000;

/// PATA / IDE Controller Registers Base Physical Address (0xD400_0000).
pub const AM34_PATA_MMIO_BASE: u32 = 0xD400_0000;
/// PATA / IDE Controller Space Size (4 Kilobytes).
pub const AM34_PATA_MMIO_SIZE: u32 = 0x0000_1000;

// ============================================================================
// 8. Default System Paths, Network Settings & Hardware Parameters
// ============================================================================

/// Default relative path to the sample Panasonic HDD 100MB dump.
pub const DEFAULT_HDD_IMAGE_PATH: &str =
    "firmware_and_images/panasonic DMR EH-55 Firmware hdd_100.bin";

/// Default relative path to the official Panasonic optical update container.
pub const DEFAULT_FRM_CONTAINER_PATH: &str = "firmware_and_images/PANA_DVD.FRM";

/// Default directory containing Panasonic DMR-EH55 PDF service manuals and schematics.
pub const DEFAULT_MANUALS_DIR: &str = "manuals_and_schematics";

/// Default output directory for exported Ghidra scripts.
pub const DEFAULT_TOOLS_DIR: &str = "tools";

/// Default directory for partition extraction.
pub const DEFAULT_HDD_EXTRACT_DIR: &str = "hdd_extracted";

/// Default serial port for host communication with the Pico controller.
#[cfg(target_os = "windows")]
pub const DEFAULT_SERIAL_PORT: &str = "COM5";
#[cfg(target_os = "macos")]
pub const DEFAULT_SERIAL_PORT: &str = "/dev/cu.usbmodem101";
#[cfg(not(any(target_os = "windows", target_os = "macos")))]
pub const DEFAULT_SERIAL_PORT: &str = "/dev/ttyACM0";

/// Resolves the workspace root across CLI, Cargo workspace subcrate tests, Desktop, or Android.
/// Honors `PANADIGA_WORKSPACE_ROOT` first, then probes CWD, ancestors, and executable ancestors.
pub fn resolve_workspace_root() -> std::path::PathBuf {
    if let Ok(env_root) = std::env::var("PANADIGA_WORKSPACE_ROOT") {
        let trimmed = env_root.trim();
        if !trimmed.is_empty() {
            return std::path::PathBuf::from(trimmed);
        }
    }
    let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    for ancestor in cwd.ancestors() {
        if ancestor.join("Cargo.toml").is_file()
            && (ancestor.join("docs").is_dir() || ancestor.join("crates").is_dir())
        {
            return ancestor.to_path_buf();
        }
    }
    if let Ok(exe) = std::env::current_exe() {
        for ancestor in exe.ancestors() {
            if ancestor.join("docs").is_dir() || ancestor.join("manuals_and_schematics").is_dir() {
                return ancestor.to_path_buf();
            }
        }
    }
    cwd
}

/// Resolves a workspace-relative subdirectory (e.g. `"docs"` or `"manuals_and_schematics"`).
pub fn find_workspace_subdir(rel: &str) -> Option<std::path::PathBuf> {
    let root = resolve_workspace_root().join(rel);
    if root.is_dir() {
        return Some(root);
    }
    for prefix in [".", "..", "../..", "../../.."] {
        let cand = std::path::Path::new(prefix).join(rel);
        if cand.is_dir() {
            return Some(cand);
        }
    }
    None
}

/// Default baud rate for Pico USB CDC communication.
pub const DEFAULT_BAUD_RATE: u32 = 115_200;

/// Default TCP listen host address for the GDB RSP bridge.
pub const DEFAULT_GDB_LISTEN_HOST: &str = "127.0.0.1";

/// Default TCP listen port for the GDB RSP bridge.
pub const DEFAULT_GDB_LISTEN_PORT: u16 = 3333;

/// Default TCP listen address and port for the GDB RSP bridge.
pub const DEFAULT_GDB_LISTEN_SOCKET: &str = "127.0.0.1:3333";

/// Candidate search paths for the raw Panasonic HDD backup image across development directories.
pub const CANDIDATE_HDD_RAW_BACKUP_PATHS: &[&str] = &[
    "firmware_and_images/panasonic_dmr-eh55_hdd_raw_backup.bin",
    "../../firmware_and_images/panasonic_dmr-eh55_hdd_raw_backup.bin",
    "../../../firmware_and_images/panasonic_dmr-eh55_hdd_raw_backup.bin",
];

/// Candidate directory search paths for Panasonic service manuals and schematics.
pub const CANDIDATE_MANUALS_DIRS: &[&str] = &[
    "manuals_and_schematics",
    "../../manuals_and_schematics",
    "../../../manuals_and_schematics",
];

// ============================================================================
// 9. Structured Memory Map Definitions (SSOT for CLI, TUI, and Ghidra)
// ============================================================================

/// Strongly-typed hardware memory region descriptor for Panasonic Matsushita AM34 / MN103S.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct MemoryRegionDefinition {
    pub name: &'static str,
    pub start: u32,
    pub end: u32,
    pub size_kb: usize,
    pub perms: &'static str,
}

impl MemoryRegionDefinition {
    /// Formats the region descriptor into standard JSON format for CLI and API consumers.
    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "name": self.name,
            "start": format!("0x{:08X}", self.start),
            "end": format!("0x{:08X}", self.end),
            "size_kb": self.size_kb,
            "perms": self.perms
        })
    }
}

/// Official Matsushita AM34 / MN103S memory map regions.
pub const AM34_SYSTEM_MEMORY_REGIONS: &[MemoryRegionDefinition] = &[
    MemoryRegionDefinition {
        name: "INTERNAL_SRAM",
        start: 0x0000_0000,
        end: 0x0003_FFFF,
        size_kb: 256,
        perms: "rwx",
    },
    MemoryRegionDefinition {
        name: "CORE_MMIO",
        start: 0x1000_0000,
        end: 0x1000_FFFF,
        size_kb: 64,
        perms: "rw-",
    },
    MemoryRegionDefinition {
        name: "MAIN_SDRAM",
        start: 0x4000_0000,
        end: 0x43FF_FFFF,
        size_kb: 65536,
        perms: "rwx",
    },
    MemoryRegionDefinition {
        name: "FLASH_NOR_IPL",
        start: 0x8000_0000,
        end: 0x807E_FFFF,
        size_kb: 8128,
        perms: "r-x",
    },
    MemoryRegionDefinition {
        name: "NVRAM_U99",
        start: 0x807F_0000,
        end: 0x807F_FFFF,
        size_kb: 64,
        perms: "rw-",
    },
    MemoryRegionDefinition {
        name: "SYSTEM_MMIO",
        start: 0xC000_0000,
        end: 0xC000_3FFF,
        size_kb: 16,
        perms: "rw-",
    },
    MemoryRegionDefinition {
        name: "AM34_OCD_MMIO",
        start: 0xC000_4000,
        end: 0xC000_40FF,
        size_kb: 1,
        perms: "rw-",
    },
    MemoryRegionDefinition {
        name: "ATA_IDE_MMIO",
        start: 0xD400_0000,
        end: 0xD400_0FFF,
        size_kb: 4,
        perms: "rw-",
    },
];

/// Returns all AM34 memory map regions as a JSON array value.
pub fn get_am34_memory_map_json() -> serde_json::Value {
    let list: Vec<serde_json::Value> = AM34_SYSTEM_MEMORY_REGIONS
        .iter()
        .map(|r| r.to_json_value())
        .collect();
    serde_json::Value::Array(list)
}

/// Formats the AM34 memory map architecture into a clean ASCII table.
pub fn format_memory_map_table() -> String {
    let mut out = String::new();
    out.push_str(
        "+------------------+---------------+---------------+-----------+-------------+\n",
    );
    out.push_str(
        "| Region           | Start Address | End Address   | Size (KB) | Permissions |\n",
    );
    out.push_str(
        "+------------------+---------------+---------------+-----------+-------------+\n",
    );
    for r in AM34_SYSTEM_MEMORY_REGIONS {
        out.push_str(&format!(
            "| {:<16} | 0x{:08X}    | 0x{:08X}    | {:>9} | {:<11} |\n",
            r.name, r.start, r.end, r.size_kb, r.perms
        ));
    }
    out.push_str(
        "+------------------+---------------+---------------+-----------+-------------+\n",
    );
    out
}
