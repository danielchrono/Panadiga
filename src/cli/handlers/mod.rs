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
pub use system::{handle_manuals, handle_memory_map, handle_rag, handle_verify_artifact};

use super::args::Commands;
use anyhow::Result;

/// Dispatches a parsed CLI command to its specialized handler.
pub fn dispatch(command: Commands) -> Result<()> {
    match command {
        Commands::Desktop | Commands::Gui => {
            crate::desktop::run_desktop_app();
            Ok(())
        }
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
    }
}
