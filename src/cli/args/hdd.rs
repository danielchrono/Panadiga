//! CLI argument definitions for HDD MEIHDFS inspection and physical drive writes.

use clap::{Args, Subcommand};
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct HddArgs {
    #[command(subcommand)]
    pub action: HddActions,
}

#[derive(Subcommand, Debug)]
pub enum HddActions {
    /// Performs zero-copy structural inspection of MBR, mirror table, allocation descriptors, and partitions
    Inspect {
        /// Path to the HDD dump image (defaults to firmware_and_images/panasonic DMR EH-55 Firmware hdd_100.bin)
        #[arg(value_name = "IMAGE_PATH")]
        image: Option<PathBuf>,

        /// Displays visual density distribution bar per 1 MB block
        #[arg(short, long)]
        density: bool,

        /// Exports structured JSON report to specified file
        #[arg(short, long, value_name = "JSON_FILE")]
        json: Option<PathBuf>,
    },

    /// Surgically extracts firmware partitions and filesystem descriptors with SHA256SUMS manifest
    Extract {
        /// Path to the HDD dump image (defaults to firmware_and_images/panasonic DMR EH-55 Firmware hdd_100.bin)
        #[arg(value_name = "IMAGE_PATH")]
        image: Option<PathBuf>,

        /// Output directory for extracted partition binaries
        #[arg(
            short,
            long,
            default_value = "extracted_partitions",
            value_name = "DIR"
        )]
        outdir: PathBuf,

        /// Exports structured JSON report to specified file
        #[arg(short, long, value_name = "JSON_FILE")]
        json: Option<PathBuf>,
    },

    /// Decodes the bit-inverted (One's Complement XOR 0xFF) MEIHDFS allocation descriptors and extents
    DecodeDescriptors {
        /// Path to the HDD dump image or extracted allocation descriptors file (defaults to firmware_and_images/panasonic DMR EH-55 Firmware hdd_100.bin)
        #[arg(value_name = "IMAGE_PATH")]
        image: Option<PathBuf>,

        /// Exports structured JSON report to specified file
        #[arg(short, long, value_name = "JSON_FILE")]
        json: Option<PathBuf>,

        /// Include journal blocks analysis if inspecting a full HDD image
        #[arg(long)]
        journal: bool,
    },

    /// Lists connected physical drives and identifies system OS drive lockouts
    ListDrives,

    /// Procedurally formats a non-system physical drive or disk image with MEIHDFS structures (mkfs.meihdfs)
    FormatMeihdfs {
        /// Target device path (e.g. \\.\PhysicalDrive1 or /dev/sdb)
        #[arg(short, long, value_name = "TARGET_DRIVE")]
        target: String,

        /// Total drive capacity in Gigabytes (e.g. 160, 250, 500, 1000)
        #[arg(short, long, default_value_t = 160, value_name = "SIZE_GB")]
        size_gb: u64,

        /// Optional path to PANA_DVD.FRM to inject Stage-2 AM34 OS kernel into Partition 1
        #[arg(long, value_name = "FRM_FILE")]
        inject_frm: Option<PathBuf>,

        /// Explicit safety confirmation token (e.g. CONFIRM-PhysicalDrive1)
        #[arg(long, value_name = "TOKEN")]
        confirm: String,
    },

    /// Streams a raw image (.bin or transparently decompressed .bin.gz) directly to a non-system physical drive
    WriteImage {
        /// Target device path (e.g. \\.\PhysicalDrive1 or /dev/sdb)
        #[arg(short, long, value_name = "TARGET_DRIVE")]
        target: String,

        /// Path to the source raw or gzipped disk image
        #[arg(short, long, value_name = "IMAGE_PATH")]
        image: PathBuf,

        /// Explicit safety confirmation token (e.g. CONFIRM-PhysicalDrive1)
        #[arg(long, value_name = "TOKEN")]
        confirm: String,
    },

    /// Restores LBA 0 and LBA 24..31 primary structures atomically from dual backup mirrors (LBA 57344 and 57368)
    RestoreMirrors {
        /// Target device path (e.g. \\.\PhysicalDrive1 or /dev/sdb)
        #[arg(short, long, value_name = "TARGET_DRIVE")]
        target: String,

        /// Explicit safety confirmation token (e.g. CONFIRM-PhysicalDrive1)
        #[arg(long, value_name = "TOKEN")]
        confirm: String,
    },
}
