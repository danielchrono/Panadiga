/**
 * Panasonic DMR-EH55 HDD Storage & MEIHDFS Forensic Data Models
 * Matches src/hdd.rs & src/constants.rs
 */

export interface MbrTableReport {
  lba0_offset: number;
  lba0_hex_signature: string;
  format_version: number;
  magic_ascii: string;
  disk_volume_guid: string;
  has_standard_mbr_55aa: boolean;
  zeros_after_header_clean: boolean;
  lba0_md5: string;
  lba0_sha256: string;
  mirror_lba: number;
  mirror_offset: number;
  mirror_is_bit_identical: boolean;
  mirror_md5: string;
  mirror_sha256: string;
}

export interface AllocationJournalReport {
  descriptors_offset: number;
  descriptors_size: number;
  descriptors_mirror_identical: boolean;
  descriptors_sha256: string;
  journal_catalog_offset: number;
  journal_total_blocks: number;
  journal_max_index: number;
  journal_block_trailers: string[];
  state_marker_offset: number;
  state_marker_hex: string;
}

export interface PartitionReport {
  partition_id: number;
  name: string;
  offset: number;
  offset_hex: string;
  lba_start: number;
  exact_payload_size: number;
  sector_aligned_size: number;
  sectors_count: number;
  signature_hex: string;
  entropy: number;
  exact_md5: string;
  exact_sha256: string;
  aligned_md5?: string;
  aligned_sha256?: string;
  header_hex_64: string;
}

export interface ChunkDensityReport {
  mb_index: number;
  offset: number;
  non_zero_bytes: number;
  percentage: number;
}

export interface HddAnalysisReport {
  image_path: string;
  file_size_bytes: number;
  file_size_mb: number;
  total_sectors: number;
  global_md5: string;
  global_sha256: string;
  mbr_tables: MbrTableReport;
  allocation_and_journal: AllocationJournalReport;
  partitions: PartitionReport[];
  density_map: ChunkDensityReport[];
}

export interface MeihdfsAllocationRecord {
  index: number;
  offset_in_table: number;
  raw_hex: string;
  inverted_hex: string;
  raw_tag: number;
  decoded_tag: number;
  tag_name: string;
  extent_id?: number;
  sector_count?: number;
  lba_offset?: number;
  flags?: number;
  tail_hex: string;
  description: string;
}

export interface MeihdfsExtentSegment {
  record_index: number;
  subtag: number;
  sector_count: number;
  lba_offset_hint: number;
  flags: number;
  tail_hex: string;
}

export interface MeihdfsExtentSummary {
  extent_id: number;
  extent_id_hex: string;
  header_record_index?: number;
  total_sectors: number;
  total_bytes: number;
  segments_count: number;
  segments: MeihdfsExtentSegment[];
}

export interface JournalBlockDecoded {
  block_index: number;
  start_lba: number;
  start_offset_hex: string;
  is_active_header: boolean;
  header_hex_16: string;
  transaction_seq: number;
  pool_size: number;
  checksum_crc: string;
  trailer_offset_hex: string;
  active_payload_sectors: number;
}

export interface MeihdfsAllocationReport {
  source_path: string;
  is_full_hdd_image: boolean;
  is_mirror_verified: boolean;
  descriptors_sha256: string;
  total_records: number;
  tag_distribution: Record<string, number>;
  records: MeihdfsAllocationRecord[];
  extents: MeihdfsExtentSummary[];
  journal_blocks: JournalBlockDecoded[];
}

export interface ExtractedPartitionItem {
  id: number;
  name: string;
  filename: string;
  offsetHex: string;
  sizeBytes: number;
  sha256: string;
  md5: string;
  entropy: number;
}

export interface PartitionExtractionManifest {
  timestamp: string;
  sourceImage: string;
  outputDirectory: string;
  manifestSha256Sums: string;
  extractedPartitions: ExtractedPartitionItem[];
}
