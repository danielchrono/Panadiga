//! CLI handler for Ghidra script generation from the Rust SSOT.

use super::super::args::{GhidraActions, GhidraArgs};
use anyhow::Result;

pub fn handle_ghidra(args: GhidraArgs) -> Result<()> {
    match args.action {
        GhidraActions::Map { output } => {
            let script = crate::ghidra::generate_ghidra_memory_map();
            std::fs::write(&output, script)?;
            log_info!("[OK] Ghidra Memory Map script generated at: {:?}", output);
        }
        GhidraActions::Symbols { output } => {
            let script = crate::ghidra::generate_ghidra_symbols();
            std::fs::write(&output, script)?;
            log_info!("[OK] Ghidra Symbols script generated at: {:?}", output);
        }
        GhidraActions::ExportAll { dir } => {
            crate::ghidra::export_all_ghidra_scripts(&dir)?;
            log_info!(
                "[OK] All Ghidra scripts generated into directory: {:?}",
                dir
            );
        }
    }
    Ok(())
}
