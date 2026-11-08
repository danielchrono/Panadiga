//! CLI Command and Argument Definitions
//!
//! Strongly typed Clap derivations for all Panadiga CLI commands.

use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "panadiga",
    author = "Antigravity Systems Engineering Team",
    version = "0.1.0",
    about = "Panasonic DMR-EH55 & DIGA Engineering & Forensic Suite",
    long_about = "High-performance, strongly typed systems engineering suite for Panasonic DIGA recorders.\nReplaces legacy Python scripts with zero-copy memory-mapped operations, rigorous\nboundary verification, cryptographic forensics, and technical documentation lookup."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Launch interactive terminal user interface (TUI)
    #[arg(short, long)]
    pub interactive: bool,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Launch native desktop GUI (WebView2 / WebKitGTK / WKWebView)
    #[command(name = "desktop")]
    Desktop,

    /// Launch native desktop GUI (alias for desktop)
    #[command(name = "gui")]
    Gui,

    /// Interactive guided terminal pipeline (TUI)
    #[command(name = "interactive", alias = "tui")]
    Interactive,

    /// NVRAM and Spansion S29JL064H NOR Flash management
    Nvram(NvramArgs),

    /// Hard Disk Drive (HDD) MEIHDFS and partition forensics
    Hdd(HddArgs),

    /// Cryptanalysis and optical update container (.FRM) inspection
    Frm(FrmArgs),

    /// Firmware partition headers, boot sectors, and formatted hex dumps
    Headers(HeadersArgs),

    /// High-performance zero-copy string extraction and keyword scanner
    Scan(ScanArgs),

    /// Index and search service manuals and schematics (PDF)
    Manuals(ManualsArgs),

    /// Hardware host controller (RP2040 / RP2350) and GDB RSP bridge
    Pico(PicoArgs),

    /// Ghidra reverse engineering script generation directly from Rust SSOT
    Ghidra(GhidraArgs),

    /// Native 6-layer Zero-Error RAG Engine for reverse engineering queries
    Rag(RagArgs),

    /// Display or export official AM34 / UniPhier MN2DS0016 memory map
    #[command(name = "memory-map", alias = "memory_map")]
    MemoryMap(MemoryMapArgs),

    /// Cryptographic artifact SHA-256 verification and assertion
    #[command(name = "verify-artifact", alias = "verify")]
    VerifyArtifact(VerifyArtifactArgs),

    /// GDB RSP bridge network and serial connection parameters
    #[command(name = "gdb-config", alias = "gdb_config")]
    GdbConfig(GdbConfigArgs),
}

// ============================================================================
// NVRAM Subcommand Definitions
// ============================================================================

#[derive(Args, Debug)]
pub struct NvramArgs {
    #[command(subcommand)]
    pub action: NvramActions,
}

#[derive(Subcommand, Debug)]
pub enum NvramActions {
    /// Cleans corrupted NVRAM sectors (0x007F0000..0x007FFFFF), backs up corrupt state, and audits microsectors
    Clean {
        /// Path to raw 8 MB Spansion S29JL064H flash dump
        #[arg(value_name = "INPUT_DUMP")]
        input: PathBuf,

        /// Path for the sanitized clean firmware output (defaults to <INPUT>_CLEAN.bin)
        #[arg(value_name = "OUTPUT_CLEAN")]
        output: Option<PathBuf>,
    },
}

// ============================================================================
// HDD Subcommand Definitions
// ============================================================================

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
}

// ============================================================================
// FRM Cryptanalysis Subcommand Definitions
// ============================================================================

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

// ============================================================================
// Headers & Hex Dump Subcommand Definitions
// ============================================================================

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

// ============================================================================
// String Scanner Subcommand Definitions
// ============================================================================

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

// ============================================================================
// Manuals Search Subcommand Definitions
// ============================================================================

#[derive(Args, Debug)]
pub struct ManualsArgs {
    /// Path to directory containing PDF service manuals and schematics
    #[arg(short, long, default_value = "manuals_and_schematics")]
    pub dir: PathBuf,

    /// Technical terms to search for
    #[arg(short, long = "term")]
    pub terms: Vec<String>,

    /// Optional substring filter for PDF filename (e.g. 'dmr-eh55p9')
    #[arg(short, long)]
    pub filter: Option<String>,
}

// ============================================================================
// Pico Hardware Controller & GDB RSP Bridge Subcommand Definitions
// ============================================================================

#[derive(Args, Debug)]
pub struct PicoArgs {
    #[command(subcommand)]
    pub action: PicoActions,
}

#[derive(Subcommand, Debug)]
pub enum PicoActions {
    /// Lists all attached serial COM ports and identifies RP2040 vs RP2350 Pico 2 hardware
    List,

    /// Probes a Pico board, verifies communication, and reads target 3.3V VDD rail voltage
    Ping {
        /// Serial port identifier (e.g. COM3 or /dev/ttyACM0)
        #[arg(short, long, value_name = "PORT")]
        port: String,

        /// Baud rate (defaults to 115200)
        #[arg(short, long, default_value_t = 115200)]
        baud: u32,
    },

    /// Starts a TCP-to-Serial GDB Remote Serial Protocol (RSP) bridge server
    GdbBridge {
        /// Serial port identifier connected to Raspberry Pi Pico (e.g. COM3)
        #[arg(short, long, value_name = "PORT")]
        port: String,

        /// Serial baud rate (defaults to 115200)
        #[arg(short, long, default_value_t = 115200)]
        baud: u32,

        /// TCP listener address and port for incoming GDB connections
        #[arg(short, long, default_value = "127.0.0.1:3333")]
        listen: String,
    },
}

// ============================================================================
// Ghidra Script Generator Subcommand Definitions
// ============================================================================

#[derive(Args, Debug)]
pub struct GhidraArgs {
    #[command(subcommand)]
    pub action: GhidraActions,
}

#[derive(Subcommand, Debug)]
pub enum GhidraActions {
    /// Export AM34 memory map Python script for Ghidra
    Map {
        /// Target Python script file path
        #[arg(short, long, default_value = "tools/ghidra_am34_memory_map.py")]
        output: PathBuf,
    },

    /// Export AM34 vector, peripheral, and NVRAM symbols Python script for Ghidra
    Symbols {
        /// Target Python script file path
        #[arg(
            short,
            long,
            default_value = "tools/ghidra_importar_simbolos_panadiga.py"
        )]
        output: PathBuf,
    },

    /// Export all Ghidra automation scripts into a target directory
    ExportAll {
        /// Destination directory for generated Ghidra scripts
        #[arg(short, long, default_value = "tools")]
        dir: PathBuf,
    },
}

// ============================================================================
// RAG, Memory Map, Verification & GDB Config Subcommand Definitions
// ============================================================================

#[derive(Args, Debug)]
pub struct RagArgs {
    #[command(subcommand)]
    pub action: RagActions,
}

#[derive(Subcommand, Debug)]
pub enum RagActions {
    /// Query the deterministic 6-layer Zero-Error RAG Engine
    Query {
        /// Technical question or prompt
        #[arg(value_name = "QUERY")]
        query: String,

        /// Output full JSON result
        #[arg(short, long)]
        json: bool,
    },
}

#[derive(Args, Debug)]
pub struct MemoryMapArgs {
    /// Target output file path (.json or table)
    #[arg(short, long, value_name = "PATH")]
    pub output: Option<PathBuf>,

    /// Output format: json, table, markdown
    #[arg(short, long, default_value = "json", value_name = "FORMAT")]
    pub format: String,
}

#[derive(Args, Debug)]
pub struct VerifyArtifactArgs {
    /// Path to firmware, NVRAM or HDD partition binary
    #[arg(value_name = "FILE")]
    pub file: PathBuf,

    /// Expected SHA-256 hash for strict assertion
    #[arg(short, long, value_name = "HEX")]
    pub sha256: Option<String>,
}

#[derive(Args, Debug)]
pub struct GdbConfigArgs {
    /// Serial port name (e.g. COM5, /dev/ttyACM0)
    #[arg(short, long, default_value = "COM5", value_name = "PORT")]
    pub port: String,

    /// TCP port for GDB remote protocol
    #[arg(short, long, default_value_t = 3333, value_name = "TCP_PORT")]
    pub tcp_port: u16,

    /// TCP listen host address
    #[arg(short = 'H', long, default_value = "127.0.0.1", value_name = "HOST")]
    pub host: String,
}
