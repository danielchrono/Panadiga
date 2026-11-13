//! CLI handler for Ghidra script generation from the Rust SSOT.

use super::super::args::{GhidraActions, GhidraArgs};
use anyhow::Result;
use panadiga_core::log_info;

fn write_ghidra_script(output: &std::path::Path, label: &str, script: &str) -> Result<()> {
    if let Some(parent) = output.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(output, script)?;
    log_info!("[OK] {} script generated at: {:?}", label, output);
    Ok(())
}

pub fn handle_ghidra(args: GhidraArgs) -> Result<()> {
    match args.action {
        GhidraActions::Map { output } => {
            let script = panadiga_core::ghidra::generate_ghidra_memory_map();
            write_ghidra_script(&output, "Ghidra Memory Map", &script)?;
        }
        GhidraActions::Symbols { output } => {
            let script = panadiga_core::ghidra::generate_ghidra_symbols();
            write_ghidra_script(&output, "Ghidra Symbols", &script)?;
        }
        GhidraActions::ExportAll { dir } => {
            panadiga_core::ghidra::export_all_ghidra_scripts(&dir)?;
            log_info!(
                "[OK] All Ghidra scripts generated into directory: {:?}",
                dir
            );
        }
    }
    Ok(())
}
