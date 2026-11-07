use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};

/// MBR structure inspection report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MbrTableReport {
    pub lba0_offset: u64,
    pub lba0_hex_signature: String,
    pub format_version: u16,
    pub magic_ascii: String,
    pub disk_volume_guid: String,
    pub has_standard_mbr_55aa: bool,
    pub zeros_after_header_clean: bool,
    pub lba0_md5: String,
    pub lba0_sha256: String,
    pub mirror_lba: u64,
    pub mirror_offset: u64,
    pub mirror_is_bit_identical: bool,
    pub mirror_md5: String,
    pub mirror_sha256: String,
}

/// Metadata and transactional journal inspection report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationJournalReport {
    pub descriptors_offset: u64,
    pub descriptors_size: usize,
    pub descriptors_mirror_identical: bool,
    pub descriptors_sha256: String,
    pub journal_catalog_offset: u64,
    pub journal_total_blocks: u32,
    pub journal_max_index: u32,
    pub journal_block_trailers: Vec<String>,
    pub state_marker_offset: u64,
    pub state_marker_hex: String,
}

/// Embedded firmware partition descriptor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartitionReport {
    pub partition_id: u32,
    pub name: String,
    pub offset: u64,
    pub offset_hex: String,
    pub lba_start: u64,
    pub exact_payload_size: usize,
    pub sector_aligned_size: usize,
    pub sectors_count: usize,
    pub signature_hex: String,
    pub entropy: f64,
    pub exact_md5: String,
    pub exact_sha256: String,
    pub aligned_md5: Option<String>,
    pub aligned_sha256: Option<String>,
    pub header_hex_64: String,
}

/// Extracted partition/descriptor metadata report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedPartitionReport {
    pub name: String,
    pub filename: String,
    pub path: String,
    pub size_bytes: usize,
    pub sha256: String,
}

/// Chunk density metric for visual disk layout representation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkDensityReport {
    pub mb_index: usize,
    pub offset: u64,
    pub non_zero_bytes: usize,
    pub percentage: f64,
}

/// Global HDD image analysis report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HddAnalysisReport {
    pub image_path: String,
    pub file_size_bytes: u64,
    pub file_size_mb: f64,
    pub total_sectors: u64,
    pub global_md5: String,
    pub global_sha256: String,
    pub mbr_tables: MbrTableReport,
    pub allocation_and_journal: AllocationJournalReport,
    pub partitions: Vec<PartitionReport>,
    pub density_map: Vec<ChunkDensityReport>,
}

/// Single decoded 16-byte MEIHDFS allocation record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeihdfsAllocationRecord {
    pub index: usize,
    pub offset_in_table: usize,
    pub raw_hex: String,
    pub inverted_hex: String,
    pub raw_tag: u8,
    pub decoded_tag: u8,
    pub tag_name: String,
    pub extent_id: Option<u16>,
    pub sector_count: Option<u32>,
    pub lba_offset: Option<u32>,
    pub flags: Option<u16>,
    pub tail_hex: String,
    pub description: String,
}

/// Extent segment detail.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeihdfsExtentSegment {
    pub record_index: usize,
    pub subtag: u8,
    pub sector_count: u16,
    pub lba_offset_hint: u16,
    pub flags: u16,
    pub tail_hex: String,
}

/// Consolidated extent summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeihdfsExtentSummary {
    pub extent_id: u16,
    pub extent_id_hex: String,
    pub header_record_index: Option<usize>,
    pub total_sectors: u32,
    pub total_bytes: u64,
    pub segments_count: usize,
    pub segments: Vec<MeihdfsExtentSegment>,
}

/// Decoded Journal Block Report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalBlockDecoded {
    pub block_index: usize,
    pub start_lba: u64,
    pub start_offset_hex: String,
    pub is_active_header: bool,
    pub header_hex_16: String,
    pub transaction_seq: u32,
    pub pool_size: u32,
    pub checksum_crc: String,
    pub trailer_offset_hex: String,
    pub active_payload_sectors: usize,
}

/// Complete report of decoded allocation descriptors & journal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeihdfsAllocationReport {
    pub source_path: String,
    pub is_full_hdd_image: bool,
    pub is_mirror_verified: bool,
    pub descriptors_sha256: String,
    pub total_records: usize,
    pub tag_distribution: BTreeMap<String, usize>,
    pub records: Vec<MeihdfsAllocationRecord>,
    pub extents: Vec<MeihdfsExtentSummary>,
    pub journal_blocks: Vec<JournalBlockDecoded>,
}
