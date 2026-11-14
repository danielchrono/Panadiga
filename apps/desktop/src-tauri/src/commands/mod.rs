//! Panasonic DMR-EH55 DIGA Suite - Tauri IPC Commands Subsystem
//!
//! Modularized by Single Responsibility Principle (SRP) and Separation of Concerns (SoC):
//! - `dto`: Data Transfer Objects matching React/TypeScript frontend models and path helpers
//! - `storage`: Spansion S29JL064H NVRAM and HDD/MEIHDFS inspection, decoding, and extraction commands
//! - `crypto_manuals`: PANA_DVD.FRM 64-bit ECB cryptanalysis, TOC, codebook, and service manuals commands
//! - `workspace`: System overview (`get_system_overview`) and binary hex dump (`read_hex_dump`) commands
//! - `hardware`: Pico USB CDC, Cold Break-In, In-Circuit Erase, GDB RSP Bridge, External Tools, and RAG commands

pub mod crypto_manuals;
pub mod dto;
pub mod frm_pack;
pub mod hardware;
pub mod hardware_diag;
pub mod storage;
pub mod workspace;

pub use crypto_manuals::*;
pub use dto::*;
pub use frm_pack::*;
pub use hardware::*;
pub use hardware_diag::*;
pub use storage::*;
pub use workspace::*;
