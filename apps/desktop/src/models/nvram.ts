/**
 * Panasonic DMR-EH55 NVRAM & Spansion Flash Data Models
 * Matches src/nvram.rs & src/constants.rs
 */

export interface MicroSectorAudit {
  sector_name: string;
  offset: number;
  active_bytes: number;
  total_bytes: number;
  header_hex: string;
}

export interface NvramCleanReport {
  input_file: string;
  output_file: string;
  backup_file: string;
  original_md5: string;
  original_sha256: string;
  clean_md5: string;
  clean_sha256: string;
  total_flash_bytes: number;
  nvram_offset_start: number;
  nvram_offset_end: number;
  nvram_non_ff_bytes: number;
  nvram_non_zero_bytes: number;
  microsectors: MicroSectorAudit[];
  bootloader_immutable: boolean;
}

export type SectorType = 'bootloader' | 'os_firmware' | 'normal_sector' | 'nvram_microsector';

export interface FlashSectorInfo {
  index: number;
  name: string;
  start_offset: number;
  end_offset: number;
  size_bytes: number;
  sector_type: SectorType;
  is_corrupted_area: boolean;
  status: 'clean' | 'corrupted' | 'factory' | 'immutable';
  active_bytes?: number;
  notes: string;
}

export interface LiveNvramCaptureEvent {
  source: string;
  port: string;
  base_bus_address: string;
  captured_bytes: number;
  captured_at: string;
  report: NvramCleanReport;
}
