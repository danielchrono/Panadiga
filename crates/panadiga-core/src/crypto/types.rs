use serde::{Deserialize, Serialize};

/// Top frequent 8-byte block record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockFrequency {
    pub hex: String,
    pub count: usize,
    pub total_bytes: usize,
    pub interpretation: String,
}

/// Analysis report for the Table of Contents (TOC) descriptors.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TocReport {
    pub entry_size_bytes: usize,
    pub total_examined_records: usize,
    pub common_subblock_1_hex: String,
    pub common_subblock_1_matches: usize,
    pub common_subblock_2_hex: String,
    pub common_subblock_2_matches: usize,
}

/// Correlation match between .FRM cipher blocks and HDD partitions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartitionCorrelation {
    pub partition_name: String,
    pub offset_hex: String,
    pub block_hex: String,
    pub interpretation: String,
    pub found_in_frm: bool,
    pub frm_occurrences: usize,
}

/// Comprehensive cryptanalysis report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptanalysisReport {
    pub frm_file: String,
    pub file_size_bytes: usize,
    pub global_entropy: f64,
    pub md5: String,
    pub sha256: String,
    pub top_blocks: Vec<BlockFrequency>,
    pub toc_analysis: TocReport,
    pub hdd_correlations: Vec<PartitionCorrelation>,
    pub is_ecb_leak_confirmed: bool,
}

/// Detailed Table of Contents (TOC) entry descriptor in PANA_DVD.FRM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TocEntry {
    pub index: usize,
    pub offset_hex: String,
    pub target_name: String,
    pub target_architecture: String,
    pub processor_family_id: String,
    pub target_descriptor_id: String,
    pub raw_hex: String,
    pub subblocks: Vec<String>,
}

/// Module layout information inside PANA_DVD.FRM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrmModuleInfo {
    pub module_index: usize,
    pub name: String,
    pub target: String,
    pub start_offset_hex: String,
    pub end_offset_hex: String,
    pub payload_size_bytes: usize,
    pub padding_size_bytes: usize,
    pub hdd_correlation: String,
}

/// Full Table of Contents dump report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrmTocDumpReport {
    pub frm_file: String,
    pub file_size_bytes: usize,
    pub toc_format: String,
    pub global_header_hex: String,
    pub toc_entries: Vec<TocEntry>,
    pub decrypted_toc: Vec<DecryptedFrmTocEntry>,
    pub modules: Vec<FrmModuleInfo>,
    pub toc_invariants_valid: bool,
}

/// Codebook entry mapping a 64-bit ciphertext block to known plaintext / semantics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodebookEntry {
    pub rank: usize,
    pub block_hex: String,
    pub known_plaintext_hex: Option<String>,
    pub interpretation: String,
    pub frm_occurrences: usize,
    pub frm_total_bytes: usize,
    pub hdd_occurrences: usize,
    pub hdd_total_bytes: usize,
    pub sample_locations: Vec<String>,
}

/// Codebook report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodebookReport {
    pub frm_file: String,
    pub hdd_file: Option<String>,
    pub total_64bit_blocks_analyzed: usize,
    pub unique_blocks: usize,
    pub dictionary: Vec<CodebookEntry>,
    pub is_ecb_leak_confirmed: bool,
    pub cross_medium_key_identity_proven: bool,
}

/// Decrypted 48-byte (`0x30`) `"Firm list"` TOC entry inside `PANA_DVD.FRM`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DecryptedFrmTocEntry {
    pub index: usize,
    pub entry_offset_hex: String,
    pub module_name: String,
    pub version_u16_hex: String,
    pub module_offset: u32,
    pub module_size: u32,
    pub hardware_family: String,
    pub target_flag: u8,
    pub model_group: String,
    pub fvu_checksum_u32: u32,
    pub list_sum_u16: u16,
    pub calculated_list_sum_u16: u16,
    pub list_sum_valid: bool,
}
