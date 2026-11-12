//! CLI argument definitions for manuals search, RAG queries, memory map, and artifact verification.

use clap::{Args, Subcommand};
use std::path::PathBuf;

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
