//! CLI argument definitions for FRM cryptanalysis, headers inspection, and deep string scans.

use clap::{Args, Subcommand};
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct FrmArgs {
    #[command(subcommand)]
    pub action: FrmActions,
}

#[derive(Subcommand, Debug)]
pub enum FrmActions {
    /// Computes Shannon entropy, detects repeated 64-bit ECB leaks, and correlates with HDD partitions
    Analyze {
        /// Path to PANA_DVD.FRM update container (defaults to firmware_and_images/PANA_DVD.FRM)
        #[arg(value_name = "FRM_FILE")]
        file: Option<PathBuf>,

        /// Path to HDD image for cross-correlation verification
        #[arg(long, value_name = "HDD_IMAGE")]
        hdd: Option<PathBuf>,

        /// Number of top frequent 64-bit ciphertext blocks to display
        #[arg(short, long, default_value_t = 15, value_name = "COUNT")]
        top: usize,

        /// Exports structured JSON report to specified file
        #[arg(short, long, value_name = "JSON_FILE")]
        json: Option<PathBuf>,
    },

    /// Dumps and dissects the 48-byte TOC entries and 3 payload modules from PANA_DVD.FRM
    DumpToc {
        /// Path to PANA_DVD.FRM update container (defaults to firmware_and_images/PANA_DVD.FRM)
        #[arg(value_name = "FRM_FILE")]
        file: Option<PathBuf>,

        /// Exports structured JSON report to specified file
        #[arg(short, long, value_name = "JSON_FILE")]
        json: Option<PathBuf>,
    },

    /// Displays the 64-bit ECB Known-Plaintext Attack (KPA) codebook dictionary and HDD correlation
    Codebook {
        /// Path to PANA_DVD.FRM update container (defaults to firmware_and_images/PANA_DVD.FRM)
        #[arg(value_name = "FRM_FILE")]
        file: Option<PathBuf>,

        /// Path to HDD image for cross-referencing codebook entries
        #[arg(long, value_name = "HDD_IMAGE")]
        hdd: Option<PathBuf>,

        /// Exports structured JSON report to specified file
        #[arg(short, long, value_name = "JSON_FILE")]
        json: Option<PathBuf>,
    },

    /// Packs modules or raw diagnostic binaries into PANA_DVD.FRM, PANA__SD.FRM, or DIGADIAG.FRM
    Pack(FrmPackArgs),

    /// Unpacks and verifies a Panasonic EXTRHEAD/EXTRFOOT container (DIGADIAG, DRV, or MAIN)
    Unpack(FrmUnpackArgs),
}

#[derive(Args, Debug)]
pub struct FrmPackArgs {
    /// Target container layout: dvd (Feistel 48B TOC), sd (plaintext 64B TOC), or digadiag (128 KiB EXTRHEAD)
    #[arg(long, default_value = "sd", value_name = "MODE")]
    pub mode: String,

    /// Output .FRM path (e.g. PANA__SD.FRM or DIGADIAG.FRM)
    #[arg(short, long, value_name = "OUT_FILE")]
    pub out: PathBuf,

    /// Module entry for dvd/sd mode in NAME:PATH format (repeatable)
    #[arg(short, long = "module", value_name = "NAME:PATH")]
    pub modules: Vec<String>,

    /// Raw binary payload path for digadiag mode
    #[arg(short, long, value_name = "BIN_FILE")]
    pub input: Option<PathBuf>,

    /// MN103S RAM load/jump address for digadiag mode (hex or decimal)
    #[arg(long, default_value = "0x50400000", value_name = "ADDR")]
    pub dst_addr: String,

    /// 4-character EXTRHEAD tag for digadiag mode
    #[arg(long, default_value = "FVU ", value_name = "TAG")]
    pub tag: String,

    /// Build identifier written to EXTRHEAD for digadiag mode
    #[arg(long, default_value = "DIAG01", value_name = "BUILD")]
    pub build: String,

    /// Hardware family metadata written into dvd/sd TOC slots
    #[arg(long, default_value = "M6930", value_name = "HW")]
    pub hw: String,

    /// Model group metadata written into dvd/sd TOC slots
    #[arg(long, default_value = "NP-309", value_name = "MODEL")]
    pub model: String,

    /// Version word written into dvd/sd TOC slots (hex or decimal)
    #[arg(long, default_value = "0x0001", value_name = "VER")]
    pub version_u16: String,

    /// Target flag byte written into dvd/sd TOC slots
    #[arg(long, default_value_t = 0)]
    pub target_flag: u8,

    /// Exports structured JSON pack report to specified file
    #[arg(short, long, value_name = "JSON_FILE")]
    pub json: Option<PathBuf>,
}

#[derive(Args, Debug)]
pub struct FrmUnpackArgs {
    /// Path to packed EXTRHEAD binary file
    #[arg(value_name = "EXTRHEAD_FILE")]
    pub file: PathBuf,

    /// Output path for the decompressed binary payload
    #[arg(short, long, value_name = "OUT_FILE")]
    pub out: PathBuf,

    /// Partial decryption mode: auto, main, drv, or plain
    #[arg(long, default_value = "auto", value_name = "MODE")]
    pub mode: String,

    /// Exports EXTRHEAD metadata and checksum verification JSON to specified file
    #[arg(short, long, value_name = "JSON_FILE")]
    pub json: Option<PathBuf>,
}

#[derive(Args, Debug)]
pub struct HeadersArgs {
    #[command(subcommand)]
    pub action: Option<HeadersActions>,

    /// Direct inspection of critical system firmware headers (HDD 32MB, 64MB and FRM)
    #[arg(short, long, default_value_t = 128)]
    pub length: usize,
}

#[derive(Subcommand, Debug)]
pub enum HeadersActions {
    /// Inspects critical firmware partition headers (32MB, 64MB, FRM)
    Inspect {
        /// Path to HDD image
        #[arg(long)]
        hdd: Option<PathBuf>,

        /// Path to PANA_DVD.FRM
        #[arg(long)]
        frm: Option<PathBuf>,

        /// Number of bytes to display per header
        #[arg(short, long, default_value_t = 128)]
        length: usize,
    },

    /// Generic hex dump of any binary file at a specified offset and length
    Dump {
        /// Path to the binary file
        #[arg(value_name = "FILE")]
        file: PathBuf,

        /// Byte offset to start dumping from
        #[arg(short, long, default_value_t = 0)]
        offset: usize,

        /// Number of bytes to dump
        #[arg(short, long, default_value_t = 128)]
        length: usize,
    },
}

#[derive(Args, Debug)]
pub struct ScanArgs {
    /// Target binary file to scan (defaults to scanning both HDD dump and FRM)
    #[arg(value_name = "FILE")]
    pub file: Option<PathBuf>,

    /// Minimum length of extracted ASCII strings
    #[arg(short, long, default_value_t = 6)]
    pub min_len: usize,

    /// Case-insensitive search keywords (can be specified multiple times)
    #[arg(short, long = "keyword")]
    pub keywords: Vec<String>,

    /// Maximum number of matching strings to display (0 for unlimited)
    #[arg(short, long, default_value_t = 50)]
    pub limit: usize,

    /// Exports structured findings to JSON file
    #[arg(short, long)]
    pub json: Option<PathBuf>,
}
