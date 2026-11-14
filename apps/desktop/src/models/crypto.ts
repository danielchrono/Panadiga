/**
 * Panasonic DMR-EH55 Cryptographic Forensics & PANA_DVD.FRM Models
 * Matches src/crypto.rs & src/constants.rs
 */

export interface BlockFrequency {
  hex: string;
  count: number;
  total_bytes: number;
  interpretation: string;
}

export interface TocReport {
  entry_size_bytes: number;
  total_examined_records: number;
  common_subblock_1_hex: string;
  common_subblock_1_matches: number;
  common_subblock_2_hex: string;
  common_subblock_2_matches: number;
}

export interface PartitionCorrelation {
  partition_name: string;
  offset_hex: string;
  block_hex: string;
  interpretation: string;
  found_in_frm: boolean;
  frm_occurrences: number;
}

export interface CryptanalysisReport {
  frm_file: string;
  file_size_bytes: number;
  global_entropy: number;
  md5: string;
  sha256: string;
  top_blocks: BlockFrequency[];
  toc_analysis: TocReport;
  hdd_correlations: PartitionCorrelation[];
  is_ecb_leak_confirmed: boolean;
}

export interface TocEntry {
  index: number;
  offset_hex: string;
  target_name: string;
  target_architecture: string;
  processor_family_id: string;
  target_descriptor_id: string;
  raw_hex: string;
  subblocks: string[];
}

export interface FrmModuleInfo {
  module_index: number;
  name: string;
  target: string;
  start_offset_hex: string;
  end_offset_hex: string;
  payload_size_bytes: number;
  padding_size_bytes: number;
  hdd_correlation: string;
}

export interface DecryptedFrmTocEntry {
  index: number;
  entry_offset_hex: string;
  module_name: string;
  version_u16_hex: string;
  module_offset: number;
  module_size: number;
  hardware_family: string;
  target_flag: number;
  model_group: string;
  fvu_checksum_u32: number;
  list_sum_u16: number;
  calculated_list_sum_u16: number;
  list_sum_valid: boolean;
}

export interface FrmTocDumpReport {
  frm_file: string;
  file_size_bytes: number;
  toc_format: string;
  global_header_hex: string;
  toc_entries: TocEntry[];
  decrypted_toc: DecryptedFrmTocEntry[];
  modules: FrmModuleInfo[];
  toc_invariants_valid: boolean;
}

export interface CodebookEntry {
  rank: number;
  block_hex: string;
  known_plaintext_hex: string | null;
  interpretation: string;
  frm_occurrences: number;
  frm_total_bytes: number;
  hdd_occurrences: number;
  hdd_total_bytes: number;
  sample_locations: string[];
}

export interface CodebookReport {
  frm_file: string;
  hdd_file?: string | null;
  total_64bit_blocks_analyzed: number;
  unique_blocks: number;
  dictionary: CodebookEntry[];
  is_ecb_leak_confirmed: boolean;
  cross_medium_key_identity_proven: boolean;
}

export interface EntropyPoint {
  offset: number;
  offset_hex: string;
  entropy: number; // 0.00 to 8.00 bits/byte
  label?: string;
}

export interface FrmPackModuleInput {
  name: string;
  path: string;
}

export interface PackFrmRequest {
  mode: 'sd' | 'dvd' | 'digadiag';
  out_path: string;
  modules?: FrmPackModuleInput[];
  input_path?: string;
  dst_addr?: string;
  tag?: string;
  build_name?: string;
  hw_family?: string;
  model_group?: string;
  version_u16?: string;
  target_flag?: number;
}

export interface PackedFrmModuleSummary {
  index: number;
  name: string;
  module_offset: number;
  module_size: number;
  fvu_checksum_u32: number;
  list_sum_u16: number;
}

export interface FrmPackReport {
  mode: string;
  total_bytes: number;
  toc_region_size: number;
  modules: PackedFrmModuleSummary[];
}

export interface ExtrheadHeader {
  tag: string;
  flag: number;
  algo: number;
  dst_len: number;
  dst_addr: number;
  src_len: number;
  footer_offset: number;
  header_checksum: number;
  adler_step: number;
  build_name: string;
}

export interface ExtrheadUnpackReport {
  file_path: string;
  header: ExtrheadHeader;
  decompressed_bytes: number;
  calculated_checksum_hex: string;
  header_checksum_hex: string;
  checksum_valid: boolean;
  footer_present: boolean;
  written_output_path?: string | null;
}
