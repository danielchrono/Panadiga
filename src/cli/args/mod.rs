//! CLI Command and Argument Definitions
//!
//! Strongly typed Clap derivations for all Panadiga CLI commands,
//! modularized by functional domain with unified public re-exports.

pub mod crypto;
pub mod ghidra;
pub mod hdd;
pub mod nvram;
pub mod pico;
pub mod system;

pub use crypto::{FrmActions, FrmArgs, HeadersActions, HeadersArgs, ScanArgs};
pub use ghidra::{GhidraActions, GhidraArgs};
pub use hdd::{HddActions, HddArgs};
pub use nvram::{NvramActions, NvramArgs};
pub use pico::{GdbConfigArgs, PicoActions, PicoArgs};
pub use system::{ManualsArgs, MemoryMapArgs, RagActions, RagArgs, VerifyArtifactArgs};

use clap::{Parser, Subcommand};

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
