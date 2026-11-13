//! Configuration constants and hyperparameters for the Panadiga Zero-Error RAG Engine.

use panadiga_core::constants::*;

/// Absolute maximum permissible voltage on digital PCB test points and debug bus (VEP79138).
pub const MAX_SAFE_VOLTAGE_VOLTS: f64 = 3.35;

/// Physical flash offset boundaries for SA0..SA133 IPL protection
pub const FORBIDDEN_FLASH_WRITE_START: usize = 0x0000_0000;
pub const FORBIDDEN_FLASH_WRITE_END: usize = SPANSION_BOOTLOADER_SIZE_BYTES - 1; // 0x007E_FFFF

/// Physical flash offset boundaries for safe NVRAM microsectors SA134..SA141
pub const SAFE_NVRAM_BASE_OFFSET: usize = SPANSION_NVRAM_BASE_OFFSET; // 0x007F_0000
pub const SAFE_NVRAM_END_OFFSET: usize = SPANSION_NVRAM_END_OFFSET; // 0x0080_0000

/// AM34 Architectural Bus boundaries for SA0..SA133 IPL protection
pub const FORBIDDEN_FLASH_BUS_START: u32 = AM34_FLASH_BASE; // 0x8000_0000
pub const FORBIDDEN_FLASH_BUS_END: u32 =
    AM34_FLASH_BASE + (SPANSION_BOOTLOADER_SIZE_BYTES as u32) - 1; // 0x807E_FFFF

/// AM34 Architectural Bus boundaries for safe NVRAM microsectors SA134..SA141
pub const SAFE_NVRAM_BUS_START: u32 = AM34_NVRAM_BASE; // 0x807F_0000
pub const SAFE_NVRAM_BUS_END: u32 = AM34_NVRAM_BASE + (SPANSION_NVRAM_SIZE_BYTES as u32) - 1; // 0x807F_FFFF

/// BM25 Okapi Parameters
pub const BM25_K1: f64 = 1.5;
pub const BM25_B: f64 = 0.75;

/// Reciprocal Rank Fusion constant
pub const RRF_K: usize = 60;

/// Minimum relevance score for a chunk to be accepted (CRAG filter)
pub const MIN_RELEVANCE_SCORE: f64 = 0.08;

/// Default number of chunks to retrieve
pub const DEFAULT_TOP_K: usize = 5;

/// Maximum number of chunks synthesized into final prompt
pub const MAX_CHUNKS_PER_PROMPT: usize = 8;

/// Epistemic abstention similarity threshold
pub const ABSTENTION_SIMILARITY_THRESHOLD: f64 = 0.12;

/// Minimum groundedness ratio required to release an answer without warnings
pub const MIN_GROUNDEDNESS_RATIO: f64 = 0.95;
