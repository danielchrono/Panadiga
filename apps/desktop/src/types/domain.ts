/**
 * @file domain.ts
 * @brief Strict domain models, type definitions, and physical constants
 *        for the Panasonic DMR-EH55 DIGA Engineering Suite.
 *
 * Adheres strictly to Single Source of Truth (SSOT), Zero Magic Numbers,
 * and Functional Programming (FP) error result models.
 */

/* =========================================================================
 * 1. Physical Hardware Constants (SSOT)
 * ========================================================================= */

/** Total physical capacity of Spansion S29JL064H NOR Flash (8 MB / 64 Mbit) */
export const SPANSION_FLASH_SIZE_BYTES = 8 * 1024 * 1024; // 8,388,608 bytes

/** Base memory offset of the top NVRAM emulation sector (0x007F_0000) */
export const SPANSION_NVRAM_BASE_OFFSET = 0x007f0000;

/** Total size of the NVRAM parameter storage partition (64 KB) */
export const SPANSION_NVRAM_SIZE_BYTES = 64 * 1024; // 65,536 bytes

/** Size of each individual microsector in top-boot partition (8 KB) */
export const SPANSION_MICROSECTOR_SIZE_BYTES = 8 * 1024; // 8,192 bytes

/** Total count of 8 KB microsectors (SA134 through SA141) */
export const SPANSION_MICROSECTOR_COUNT = 8;

/** First microsector numeric index in Spansion datasheet */
export const SPANSION_MICROSECTOR_BASE_INDEX = 134;

/** Standard ATA/IDE sector size in bytes */
export const LBA_SECTOR_SIZE_BYTES = 512;

/** Panasonic MEIHDFS Storage Header magic string */
export const MEIHDFS_MBR_SIGNATURE_ASCII = "SH";

/** Known Feistel 64-bit ECB Ciphertext for 8 consecutive 0x00 bytes */
export const ECB_CIPHERTEXT_NULL_BLOCK_HEX = "E5D1F34A518B2690";

/** Known Feistel 64-bit ECB Ciphertext for 8 consecutive 0xFF bytes (Virgin Flash) */
export const ECB_CIPHERTEXT_VIRGIN_FLASH_HEX = "035065C87BB8E090";

/** Nominal DWire32A clock speed in Hertz */
export const DWIRE_NOMINAL_CLOCK_HZ = 1_000_000;

/* =========================================================================
 * 2. Enumerations
 * ========================================================================= */

/** Navigation views available in the desktop application */
export enum DigaTab {
  NVRAM_SANITIZER = "NVRAM_SANITIZER",
  HDD_FORENSICS = "HDD_FORENSICS",
  FRM_CRYPTANALYSIS = "FRM_CRYPTANALYSIS",
  DWIRE_PICO_BRIDGE = "DWIRE_PICO_BRIDGE",
  DOCUMENTATION_SEARCH = "DOCUMENTATION_SEARCH",
}

/** Execution status of async engineering tasks */
export enum ExecutionStatus {
  IDLE = "IDLE",
  RUNNING = "RUNNING",
  SUCCESS = "SUCCESS",
  ERROR = "ERROR",
}

/** Cold Break-in synchronization modes for DWire32A OCD */
export enum DwireBreakInMode {
  MODE_0 = 0, // EXTRG Idle Low -> High, 32 clocks pre-reset
  MODE_1 = 1, // EXTRG Idle High -> Low, 32 clocks pre-reset
  MODE_2 = 2, // EXTRG Idle Low -> High, 32 clocks pre + post burst
  MODE_3 = 3, // EXTRG Idle High -> Low, 32 clocks pre + post burst
}

/* =========================================================================
 * 3. Functional Programming Result Type (Either pattern)
 * ========================================================================= */

export type Result<T, E = Error> =
  | { readonly success: true; readonly data: T }
  | { readonly success: false; readonly error: E };

export const ok = <T>(data: T): Result<T, never> => ({
  success: true,
  data,
});

export const err = <E>(error: E): Result<never, E> => ({
  success: false,
  error,
});

/* =========================================================================
 * 4. Domain Data Transfer Objects (DTO)
 * ========================================================================= */

/** Microsector audit record */
export interface MicroSectorAudit {
  readonly sector_name: string;
  readonly offset: number;
  readonly active_bytes: number;
  readonly total_bytes: number;
  readonly header_hex: string;
}

/** Report generated when cleaning Spansion NVRAM */
export interface NvramCleanReport {
  readonly input_file: string;
  readonly output_file: string;
  readonly backup_file: string;
  readonly original_md5: string;
  readonly original_sha256: string;
  readonly clean_md5: string;
  readonly clean_sha256: string;
  readonly total_flash_bytes: number;
  readonly nvram_offset_start: number;
  readonly nvram_offset_end: number;
  readonly nvram_non_ff_bytes: number;
  readonly nvram_non_zero_bytes: number;
  readonly microsectors: readonly MicroSectorAudit[];
  readonly bootloader_immutable: boolean;
}

/** Partition metadata in MEIHDFS drive */
export interface HddPartitionInfo {
  readonly partition_index: number;
  readonly start_lba: number;
  readonly byte_offset: number;
  readonly size_bytes: number;
  readonly header_magic: string;
  readonly exact_sha256: string;
}

/** HDD MBR and Partition Inspection Report */
export interface HddInspectionReport {
  readonly file_path: string;
  readonly file_size_bytes: number;
  readonly total_sectors: number;
  readonly mbr_tables: {
    readonly magic_ascii: string;
    readonly lba0_sha256: string;
    readonly mirror_lba: number;
    readonly mirror_is_bit_identical: boolean;
  };
  readonly partitions: readonly HddPartitionInfo[];
}

/** Top frequent 8-byte block in FRM cryptanalysis */
export interface FrmCipherBlock {
  readonly hex: string;
  readonly count: number;
  readonly known_plaintext?: string;
}

/** Cryptanalysis report for optical container PANA_DVD.FRM */
export interface FrmAnalysisReport {
  readonly file_path: string;
  readonly file_size_bytes: number;
  readonly global_entropy: number;
  readonly is_ecb_leak_confirmed: boolean;
  readonly virgin_flash_block_count: number;
  readonly null_block_count: number;
  readonly top_blocks: readonly FrmCipherBlock[];
}

/** Diagnostic probe hit from DWire32A scan */
export interface DwireProbeHit {
  readonly opcode_hex: string;
  readonly response_hex: string;
  readonly classification: string;
}

/** Diagnostic statistics gathered from DWire32A Pico scan */
export interface DwireScanStats {
  readonly vdd_volts: number;
  readonly is_powered: boolean;
  readonly total_probes: number;
  readonly open_bus_high: number;
  readonly open_bus_low: number;
  readonly hits_detected: number;
  readonly candidate_hits: readonly DwireProbeHit[];
}

/** Matched manual search entry */
export interface ManualSearchResult {
  readonly manual_filename: string;
  readonly page_number: number;
  readonly snippet: string;
}
