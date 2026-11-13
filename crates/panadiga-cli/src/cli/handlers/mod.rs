//! CLI Command Handlers and Dispatch Engine
//!
//! Modularized by functional domain with unified `dispatch` and handler re-exports.

pub mod crypto;
pub mod ghidra;
pub mod hdd;
pub mod nvram;
pub mod pico;
pub mod system;

pub use crypto::{handle_frm, handle_headers, handle_scan};
pub use ghidra::handle_ghidra;
pub use hdd::handle_hdd;
pub use nvram::handle_nvram;
pub use pico::{handle_gdb_config, handle_pico};
pub use system::{
    handle_manuals, handle_mcp, handle_memory_map, handle_rag, handle_verify_artifact,
};

use super::args::Commands;
use anyhow::Result;
use std::path::PathBuf;
use std::process::Command;

/// Locates and launches the sibling `panadiga-desktop` native Tauri GUI executable across OSes.
pub fn launch_desktop_executable() -> Result<()> {
    let bin_name = if cfg!(windows) {
        "panadiga-desktop.exe"
    } else {
        "panadiga-desktop"
    };

    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Ok(exe) = std::env::current_exe()
        && let Some(dir) = exe.parent()
    {
        candidates.push(dir.join(bin_name));
    }
    let root = panadiga_core::constants::resolve_workspace_root();
    candidates.push(root.join("target").join("release").join(bin_name));
    candidates.push(root.join("target").join("debug").join(bin_name));

    for cand in candidates {
        if cand.is_file() {
            Command::new(&cand).spawn()?;
            return Ok(());
        }
    }

    anyhow::bail!(
        "Native desktop executable '{}' not found alongside CLI. Build with `npm run build` or use `-i` for TUI.",
        bin_name
    )
}

/// Dispatches a parsed CLI command to its specialized handler.
pub fn dispatch(command: Commands) -> Result<()> {
    match command {
        Commands::Desktop | Commands::Gui => launch_desktop_executable(),
        Commands::Interactive => crate::tui::run_interactive_pipeline(),
        Commands::Nvram(args) => handle_nvram(args),
        Commands::Hdd(args) => handle_hdd(args),
        Commands::Frm(args) => handle_frm(args),
        Commands::Headers(args) => handle_headers(args),
        Commands::Scan(args) => handle_scan(args),
        Commands::Manuals(args) => handle_manuals(args),
        Commands::Pico(args) => handle_pico(args),
        Commands::Ghidra(args) => handle_ghidra(args),
        Commands::Rag(args) => handle_rag(args),
        Commands::MemoryMap(args) => handle_memory_map(args),
        Commands::VerifyArtifact(args) => handle_verify_artifact(args),
        Commands::GdbConfig(args) => handle_gdb_config(args),
        Commands::GdbBridge { port, baud, listen } => {
            panadiga_core::pico::run_gdb_bridge(&port, baud, &listen)
        }
        Commands::Mcp(args) => handle_mcp(args),
    }
}
