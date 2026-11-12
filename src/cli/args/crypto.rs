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
