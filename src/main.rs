//! Panasonic DIGA Engineering CLI (`panadiga`)
//!
//! Command-line interface providing high-performance zero-copy analysis,
//! NVRAM flash sanitization, cryptographic forensics, and service manual indexing.

use std::path::PathBuf;

use anyhow::Result;
use clap::{Args, Parser, Subcommand};

use panadiga::constants::{DEFAULT_FRM_CONTAINER_PATH, DEFAULT_HDD_IMAGE_PATH};
use panadiga::crypto::{analyze_frm_container, dump_toc, print_codebook};
use panadiga::hdd::{decode_meihdfs_descriptors, extract_hdd_partitions, inspect_hdd_image};
use panadiga::headers::{dump_file_hex, inspect_firmware_headers};
use panadiga::manuals::search_manuals;
use panadiga::nvram::clean_flash_nvram;
use panadiga::pico::{list_pico_devices, ping_pico, run_gdb_bridge};
use panadiga::scan::{run_deep_scan, scan_all_default};
use panadiga::tui::run_interactive_pipeline;
use panadiga::{log_error, log_info, log_println};

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
    command: Option<Commands>,

    /// Launch interactive terminal user interface (TUI)
    #[arg(short, long)]
    interactive: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
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
// Ghidra Script Generator Subcommand Definitions
// ============================================================================

#[derive(Args, Debug)]
struct GhidraArgs {
    #[command(subcommand)]
    action: GhidraActions,
}

#[derive(Subcommand, Debug)]
enum GhidraActions {
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
struct RagArgs {
    #[command(subcommand)]
    action: RagActions,
}

#[derive(Subcommand, Debug)]
enum RagActions {
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
struct MemoryMapArgs {
    /// Target output file path (.json or table)
    #[arg(short, long, value_name = "PATH")]
    output: Option<PathBuf>,

    /// Output format: json, table, markdown
    #[arg(short, long, default_value = "json", value_name = "FORMAT")]
    format: String,
}

#[derive(Args, Debug)]
struct VerifyArtifactArgs {
    /// Path to firmware, NVRAM or HDD partition binary
    #[arg(value_name = "FILE")]
    file: PathBuf,

    /// Expected SHA-256 hash for strict assertion
    #[arg(short, long, value_name = "HEX")]
    sha256: Option<String>,
}

#[derive(Args, Debug)]
struct GdbConfigArgs {
    /// Serial port name (e.g. COM5, /dev/ttyACM0)
    #[arg(short, long, default_value = "COM5", value_name = "PORT")]
    port: String,

    /// TCP port for GDB remote protocol
    #[arg(short, long, default_value_t = 3333, value_name = "TCP_PORT")]
    tcp_port: u16,

    /// TCP listen host address
    #[arg(short = 'H', long, default_value = "127.0.0.1", value_name = "HOST")]
    host: String,
}

// ============================================================================
// Entrypoint
// ============================================================================

fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.interactive
        || cli.command.is_none()
        || matches!(cli.command, Some(Commands::Interactive))
    {
        return run_interactive_pipeline();
    }

    match cli.command.unwrap() {
        Commands::Interactive => run_interactive_pipeline()?,
        Commands::Nvram(nvram_args) => match nvram_args.action {
            NvramActions::Clean { input, output } => {
                clean_flash_nvram(&input, output.as_deref())?;
            }
        },
        Commands::Hdd(hdd_args) => match hdd_args.action {
            HddActions::Inspect {
                image,
                density,
                json,
            } => {
                let default_img = PathBuf::from(DEFAULT_HDD_IMAGE_PATH);
                let img_path = image.unwrap_or(default_img);
                inspect_hdd_image(&img_path, density, json.as_deref())?;
            }
            HddActions::Extract {
                image,
                outdir,
                json,
            } => {
                let default_img = PathBuf::from(DEFAULT_HDD_IMAGE_PATH);
                let img_path = image.unwrap_or(default_img);
                extract_hdd_partitions(&img_path, &outdir, json.as_deref())?;
            }
            HddActions::DecodeDescriptors {
                image,
                json,
                journal,
            } => {
                let default_img = PathBuf::from(DEFAULT_HDD_IMAGE_PATH);
                let img_path = image.unwrap_or(default_img);
                decode_meihdfs_descriptors(&img_path, journal, json.as_deref())?;
            }
        },
        Commands::Frm(frm_args) => match frm_args.action {
            FrmActions::Analyze {
                file,
                hdd,
                top,
                json,
            } => {
                let default_frm = PathBuf::from(DEFAULT_FRM_CONTAINER_PATH);
                let frm_path = file.unwrap_or(default_frm);
                analyze_frm_container(&frm_path, hdd.as_deref(), top, json.as_deref())?;
            }
            FrmActions::DumpToc { file, json } => {
                let default_frm = PathBuf::from(DEFAULT_FRM_CONTAINER_PATH);
                let frm_path = file.unwrap_or(default_frm);
                dump_toc(&frm_path, json.as_deref())?;
            }
            FrmActions::Codebook { file, hdd, json } => {
                let default_frm = PathBuf::from(DEFAULT_FRM_CONTAINER_PATH);
                let frm_path = file.unwrap_or(default_frm);
                print_codebook(&frm_path, hdd.as_deref(), json.as_deref())?;
            }
        },
        Commands::Headers(headers_args) => match headers_args.action {
            Some(HeadersActions::Inspect { hdd, frm, length }) => {
                inspect_firmware_headers(hdd.as_deref(), frm.as_deref(), length)?;
            }
            Some(HeadersActions::Dump {
                file,
                offset,
                length,
            }) => {
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
                    run_deep_scan(
                        &f,
                        scan_args.min_len,
                        custom_kws,
                        scan_args.limit,
                        scan_args.json.as_deref(),
                    )?;
                }
                None => {
                    scan_all_default(scan_args.min_len, scan_args.limit)?;
                }
            }
        }
        Commands::Manuals(manuals_args) => {
            let custom_terms = if manuals_args.terms.is_empty() {
                None
            } else {
                Some(manuals_args.terms)
            };
            search_manuals(
                &manuals_args.dir,
                custom_terms,
                manuals_args.filter.as_deref(),
            )?;
        }
        Commands::Pico(pico_args) => match pico_args.action {
            PicoActions::List => {
                let devices = list_pico_devices()?;
                log_println!(
                    "===================================================================="
                );
                log_println!("   PANADIGA HARDWARE DISCOVERY: SERIAL COM PORTS & PICO CONTROLLERS");
                log_println!(
                    "===================================================================="
                );
                if devices.is_empty() {
                    log_println!("No USB serial devices detected on system.");
                } else {
                    for dev in &devices {
                        log_println!(
                            "Port: {:<10} | {:<32} | VID:{:04X} PID:{:04X} | {}",
                            dev.port_name,
                            dev.architecture.to_string(),
                            dev.vid,
                            dev.pid,
                            dev.description
                        );
                    }
                }
                log_println!(
                    "--------------------------------------------------------------------"
                );
            }
            PicoActions::Ping { port, baud } => {
                let report = ping_pico(&port, baud)?;
                log_println!("{}", report);
            }
            PicoActions::GdbBridge { port, baud, listen } => {
                run_gdb_bridge(&port, baud, &listen)?;
            }
        },
        Commands::Ghidra(ghidra_args) => match ghidra_args.action {
            GhidraActions::Map { output } => {
                let script = panadiga::ghidra::generate_ghidra_memory_map();
                std::fs::write(&output, script)?;
                log_info!("[OK] Ghidra Memory Map script generated at: {:?}", output);
            }
            GhidraActions::Symbols { output } => {
                let script = panadiga::ghidra::generate_ghidra_symbols();
                std::fs::write(&output, script)?;
                log_info!("[OK] Ghidra Symbols script generated at: {:?}", output);
            }
            GhidraActions::ExportAll { dir } => {
                panadiga::ghidra::export_all_ghidra_scripts(&dir)?;
                log_info!(
                    "[OK] All Ghidra scripts generated into directory: {:?}",
                    dir
                );
            }
        },
        Commands::Rag(rag_args) => {
            match rag_args.action {
                RagActions::Query { query, json } => {
                    let ans = panadiga::rag::query(&query);
                    if json {
                        let dto = ans.to_query_result();
                        log_println!("{}", serde_json::to_string_pretty(&dto)?);
                    } else {
                        log_println!(
                            "===================================================================="
                        );
                        log_println!("         PANADIGA ZERO-ERROR RAG ENGINE (NATIVE RUST)");
                        log_println!(
                            "===================================================================="
                        );
                        log_println!("Query:              {}", query);
                        log_println!("Groundedness Score: {:.2}%", ans.groundedness_score * 100.0);
                        log_println!("Confidence:         {:.2}%", ans.confidence * 100.0);
                        if ans.is_abstained {
                            log_println!("Status:             ABSTAINED");
                            if let Some(reason) = &ans.abstention_reason {
                                log_println!("Reason:             {}", reason);
                            }
                        } else {
                            log_println!("Status:             ANSWERED");
                        }
                        if let Some(s) = &ans.safety_report {
                            log_println!(
                                "Safety Check:       {}",
                                if s.is_safe {
                                    "PASS (SAFE)"
                                } else {
                                    "FAIL (HAZARD DETECTED)"
                                }
                            );
                            for v in &s.violations {
                                log_println!("  [!] VIOLATION: {}", v);
                            }
                            for a in &s.critical_alerts {
                                log_println!("  [*] ALERT: {}", a);
                            }
                        }
                        if let Some(c) = &ans.critic_report {
                            log_println!(
                                "Critic Faithful:    {}",
                                if c.is_faithful { "YES" } else { "NO" }
                            );
                            if !c.ungrounded_entities.is_empty() {
                                log_println!("  [?] Ungrounded:   {:?}", c.ungrounded_entities);
                            }
                        }
                        log_println!(
                            "--------------------------------------------------------------------"
                        );
                        log_println!("ANSWER:\n{}", ans.answer_text);
                        if !ans.citations.is_empty() {
                            log_println!("--------------------------------------------------------------------");
                            log_println!("CITATIONS:");
                            for (idx, cit) in ans.citations.iter().enumerate() {
                                log_println!(
                                    "  [{}] {} | Section: {} | Lines: {}",
                                    idx + 1,
                                    cit.doc_title,
                                    cit.section_title,
                                    cit.line_range
                                );
                                log_println!("      Excerpt: \"{}\"", cit.excerpt);
                            }
                        }
                        log_println!(
                            "===================================================================="
                        );
                    }
                }
            }
        }
        Commands::MemoryMap(args) => {
            let regions = [
                serde_json::json!({
                    "name": "INTERNAL_SRAM",
                    "start": "0x00000000",
                    "end": "0x0003FFFF",
                    "size_kb": 256,
                    "perms": "rwx"
                }),
                serde_json::json!({
                    "name": "CORE_MMIO",
                    "start": "0x10000000",
                    "end": "0x1000FFFF",
                    "size_kb": 64,
                    "perms": "rw-"
                }),
                serde_json::json!({
                    "name": "MAIN_SDRAM",
                    "start": "0x40000000",
                    "end": "0x43FFFFFF",
                    "size_kb": 65536,
                    "perms": "rwx"
                }),
                serde_json::json!({
                    "name": "FLASH_NOR_IPL",
                    "start": "0x80000000",
                    "end": "0x807EFFFF",
                    "size_kb": 8128,
                    "perms": "r-x"
                }),
                serde_json::json!({
                    "name": "NVRAM_U99",
                    "start": "0x807F0000",
                    "end": "0x807FFFFF",
                    "size_kb": 64,
                    "perms": "rw-"
                }),
                serde_json::json!({
                    "name": "SYSTEM_MMIO",
                    "start": "0xC0000000",
                    "end": "0xC0003FFF",
                    "size_kb": 16,
                    "perms": "rw-"
                }),
                serde_json::json!({
                    "name": "DWIRE32A_OCD",
                    "start": "0xC0004000",
                    "end": "0xC00040FF",
                    "size_kb": 1,
                    "perms": "rw-"
                }),
                serde_json::json!({
                    "name": "ATA_IDE_MMIO",
                    "start": "0xD4000000",
                    "end": "0xD4000FFF",
                    "size_kb": 4,
                    "perms": "rw-"
                }),
            ];

            if args.format.to_lowercase() == "json" {
                let data = serde_json::to_string_pretty(&regions)?;
                if let Some(out_p) = args.output {
                    std::fs::write(&out_p, &data)?;
                    log_println!("Memory map written to {:?}", out_p);
                } else {
                    log_println!("{}", data);
                }
            } else {
                let mut out = String::new();
                out.push_str("+------------------+---------------+---------------+-----------+-------------+\n");
                out.push_str("| Region           | Start Address | End Address   | Size (KB) | Permissions |\n");
                out.push_str("+------------------+---------------+---------------+-----------+-------------+\n");
                for r in &regions {
                    out.push_str(&format!(
                        "| {:<16} | {:<13} | {:<13} | {:>9} | {:<11} |\n",
                        r["name"].as_str().unwrap(),
                        r["start"].as_str().unwrap(),
                        r["end"].as_str().unwrap(),
                        r["size_kb"].as_i64().unwrap(),
                        r["perms"].as_str().unwrap()
                    ));
                }
                out.push_str("+------------------+---------------+---------------+-----------+-------------+\n");
                if let Some(out_p) = args.output {
                    std::fs::write(&out_p, &out)?;
                    log_println!("Memory map written to {:?}", out_p);
                } else {
                    print!("{}", out);
                }
            }
        }
        Commands::VerifyArtifact(args) => {
            if !args.file.exists() {
                log_error!("ERROR: File not found: {:?}", args.file);
                std::process::exit(1);
            }
            let data = std::fs::read(&args.file)?;
            let size = data.len();
            use sha2::{Digest, Sha256};
            let mut hasher = Sha256::new();
            hasher.update(&data);
            let computed_sha256 = hex::encode(hasher.finalize());

            log_println!("====================================================================");
            log_println!("            CRYPTOGRAPHIC ARTIFACT VERIFICATION");
            log_println!("====================================================================");
            log_println!(
                "File:     {}",
                args.file.file_name().and_then(|n| n.to_str()).unwrap_or("")
            );
            log_println!("Size:     {} bytes ({:.2} KB)", size, size as f64 / 1024.0);
            log_println!("SHA-256:  {}", computed_sha256);
            log_println!("--------------------------------------------------------------------");

            if let Some(expected) = args.sha256 {
                if computed_sha256.eq_ignore_ascii_case(&expected) {
                    log_println!("✔ HASH CONFIRMED SUCCESSFULLY!");
                } else {
                    log_error!("✖ HASH MISMATCH! Expected: {}", expected);
                    std::process::exit(2);
                }
            }
        }
        Commands::GdbConfig(args) => {
            log_println!("====================================================================");
            log_println!("          DWire32A GDB RSP Bridge Configuration");
            log_println!("====================================================================");
            log_println!("• Serial Port:   {}", args.port);
            log_println!("• Listen Socket: {}:{}", args.host, args.tcp_port);
            log_println!();
            log_println!("GDB Command to Connect:");
            log_println!("  (gdb) target remote {}:{}", args.host, args.tcp_port);
            log_println!("  (gdb) set architecture am34");
            log_println!("  (gdb) info registers");
            log_println!("====================================================================");
        }
    }

    Ok(())
}
