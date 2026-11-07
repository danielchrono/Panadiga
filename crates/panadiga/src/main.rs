//! Panasonic DIGA Engineering CLI (`panadiga`)
//!
//! Command-line interface providing high-performance zero-copy analysis,
//! NVRAM flash sanitization, cryptographic forensics, and service manual indexing.

use std::path::PathBuf;

use anyhow::Result;
use clap::{Args, Parser, Subcommand};

use panadiga::crypto::{analyze_frm_container, dump_toc, print_codebook};
use panadiga::hdd::{decode_meihdfs_descriptors, extract_hdd_partitions, inspect_hdd_image};
use panadiga::headers::{dump_file_hex, inspect_firmware_headers};
use panadiga::manuals::search_manuals;
use panadiga::nvram::clean_flash_nvram;
use panadiga::pico::{list_pico_devices, ping_pico, run_gdb_bridge};
use panadiga::scan::{run_deep_scan, scan_all_default};

#[derive(Parser, Debug)]
#[command(
    name = "panadiga",
    author = "Antigravity Systems Engineering Team",
    version = "0.1.0",
    about = "Panasonic DMR-EH55 & DIGA Engineering & Forensic Suite",
    long_about = "High-performance, strongly typed systems engineering suite for Panasonic DIGA recorders.\nReplaces legacy Python scripts with zero-copy memory-mapped operations, rigorous\nboundary verification, cryptographic forensics, and technical documentation lookup."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
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
}

// ============================================================================
// NVRAM Subcommand Definitions
// ============================================================================

#[derive(Args, Debug)]
struct NvramArgs {
    #[command(subcommand)]
    action: NvramActions,
}

#[derive(Subcommand, Debug)]
enum NvramActions {
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
struct HddArgs {
    #[command(subcommand)]
    action: HddActions,
}

#[derive(Subcommand, Debug)]
enum HddActions {
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
        #[arg(short, long, default_value = "extracted_partitions", value_name = "DIR")]
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
struct FrmArgs {
    #[command(subcommand)]
    action: FrmActions,
}

#[derive(Subcommand, Debug)]
enum FrmActions {
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
struct HeadersArgs {
    #[command(subcommand)]
    action: Option<HeadersActions>,

    /// Direct inspection of critical system firmware headers (HDD 32MB, 64MB and FRM)
    #[arg(short, long, default_value_t = 128)]
    length: usize,
}

#[derive(Subcommand, Debug)]
enum HeadersActions {
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
struct ScanArgs {
    /// Target binary file to scan (defaults to scanning both HDD dump and FRM)
    #[arg(value_name = "FILE")]
    file: Option<PathBuf>,

    /// Minimum length of extracted ASCII strings
    #[arg(short, long, default_value_t = 6)]
    min_len: usize,

    /// Case-insensitive search keywords (can be specified multiple times)
    #[arg(short, long = "keyword")]
    keywords: Vec<String>,

    /// Maximum number of matching strings to display (0 for unlimited)
    #[arg(short, long, default_value_t = 50)]
    limit: usize,

    /// Exports structured findings to JSON file
    #[arg(short, long)]
    json: Option<PathBuf>,
}

// ============================================================================
// Manuals Search Subcommand Definitions
// ============================================================================

#[derive(Args, Debug)]
struct ManualsArgs {
    /// Path to directory containing PDF service manuals and schematics
    #[arg(short, long, default_value = "manuals_and_schematics")]
    dir: PathBuf,

    /// Technical terms to search for
    #[arg(short, long = "term")]
    terms: Vec<String>,

    /// Optional substring filter for PDF filename (e.g. 'dmr-eh55p9')
    #[arg(short, long)]
    filter: Option<String>,
}

// ============================================================================
// Pico Hardware Controller & GDB RSP Bridge Subcommand Definitions
// ============================================================================

#[derive(Args, Debug)]
struct PicoArgs {
    #[command(subcommand)]
    action: PicoActions,
}

#[derive(Subcommand, Debug)]
enum PicoActions {
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
// Entrypoint
// ============================================================================

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Nvram(nvram_args) => match nvram_args.action {
            NvramActions::Clean { input, output } => {
                clean_flash_nvram(&input, output.as_deref())?;
            }
        },
        Commands::Hdd(hdd_args) => match hdd_args.action {
            HddActions::Inspect { image, density, json } => {
                let default_img = PathBuf::from("firmware_and_images/panasonic DMR EH-55 Firmware hdd_100.bin");
                let img_path = image.unwrap_or(default_img);
                inspect_hdd_image(&img_path, density, json.as_deref())?;
            }
            HddActions::Extract { image, outdir, json } => {
                let default_img = PathBuf::from("firmware_and_images/panasonic DMR EH-55 Firmware hdd_100.bin");
                let img_path = image.unwrap_or(default_img);
                extract_hdd_partitions(&img_path, &outdir, json.as_deref())?;
            }
            HddActions::DecodeDescriptors { image, json, journal } => {
                let default_img = PathBuf::from("firmware_and_images/panasonic DMR EH-55 Firmware hdd_100.bin");
                let img_path = image.unwrap_or(default_img);
                decode_meihdfs_descriptors(&img_path, journal, json.as_deref())?;
            }
        },
        Commands::Frm(frm_args) => match frm_args.action {
            FrmActions::Analyze { file, hdd, top, json } => {
                let default_frm = PathBuf::from("firmware_and_images/PANA_DVD.FRM");
                let frm_path = file.unwrap_or(default_frm);
                analyze_frm_container(&frm_path, hdd.as_deref(), top, json.as_deref())?;
            }
            FrmActions::DumpToc { file, json } => {
                let default_frm = PathBuf::from("firmware_and_images/PANA_DVD.FRM");
                let frm_path = file.unwrap_or(default_frm);
                dump_toc(&frm_path, json.as_deref())?;
            }
            FrmActions::Codebook { file, hdd, json } => {
                let default_frm = PathBuf::from("firmware_and_images/PANA_DVD.FRM");
                let frm_path = file.unwrap_or(default_frm);
                print_codebook(&frm_path, hdd.as_deref(), json.as_deref())?;
            }
        },
        Commands::Headers(headers_args) => match headers_args.action {
            Some(HeadersActions::Inspect { hdd, frm, length }) => {
                inspect_firmware_headers(hdd.as_deref(), frm.as_deref(), length)?;
            }
            Some(HeadersActions::Dump { file, offset, length }) => {
                dump_file_hex(&file, offset, length)?;
            }
            None => {
                inspect_firmware_headers(None, None, headers_args.length)?;
            }
        },
        Commands::Scan(scan_args) => {
            let custom_kws = if scan_args.keywords.is_empty() {
                None
            } else {
                Some(scan_args.keywords)
            };

            match scan_args.file {
                Some(f) => {
                    run_deep_scan(&f, scan_args.min_len, custom_kws, scan_args.limit, scan_args.json.as_deref())?;
                }
                None => {
                    scan_all_default(scan_args.min_len, scan_args.limit)?;
                }
            }
        },
        Commands::Manuals(manuals_args) => {
            let custom_terms = if manuals_args.terms.is_empty() {
                None
            } else {
                Some(manuals_args.terms)
            };
            search_manuals(&manuals_args.dir, custom_terms, manuals_args.filter.as_deref())?;
        }
        Commands::Pico(pico_args) => match pico_args.action {
            PicoActions::List => {
                let devices = list_pico_devices()?;
                println!("====================================================================");
                println!("   PANADIGA HARDWARE DISCOVERY: SERIAL COM PORTS & PICO CONTROLLERS");
                println!("====================================================================");
                if devices.is_empty() {
                    println!("No USB serial devices detected on system.");
                } else {
                    for dev in &devices {
                        println!("Port: {:<10} | {:<32} | VID:{:04X} PID:{:04X} | {}",
                            dev.port_name,
                            dev.architecture.to_string(),
                            dev.vid,
                            dev.pid,
                            dev.description
                        );
                    }
                }
                println!("--------------------------------------------------------------------");
            }
            PicoActions::Ping { port, baud } => {
                let report = ping_pico(&port, baud)?;
                println!("{}", report);
            }
            PicoActions::GdbBridge { port, baud, listen } => {
                run_gdb_bridge(&port, baud, &listen)?;
            }
        },
    }

    Ok(())
}
