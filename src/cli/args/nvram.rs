//! CLI argument definitions for NOR Flash and NVRAM management.

use clap::{Args, Subcommand};
use std::path::PathBuf;

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
