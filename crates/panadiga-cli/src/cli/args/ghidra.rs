//! CLI argument definitions for Ghidra script generation.

use clap::{Args, Subcommand};
use std::path::PathBuf;

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
        #[arg(short, long, default_value = "tools/ghidra/ghidra_am34_memory_map.py")]
        output: PathBuf,
    },

    /// Export AM34 vector, peripheral, and NVRAM symbols Python script for Ghidra
    Symbols {
        /// Target Python script file path
        #[arg(
            short,
            long,
            default_value = "tools/ghidra/ghidra_importar_simbolos_panadiga.py"
        )]
        output: PathBuf,
    },

    /// Export all Ghidra automation scripts into a target directory
    ExportAll {
        /// Destination directory for generated Ghidra scripts
        #[arg(short, long, default_value = "tools/ghidra")]
        dir: PathBuf,
    },
}
