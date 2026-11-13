//! Panasonic DIGA Core Library (`panadiga-core`)
//!
//! Pure cross-platform domain models, hardware SSOT, Matsushita AM34/MN103S constants,
//! Spansion NOR Flash / NVRAM surgery, MEIHFS HDD forensics, PANA_DVD.FRM cryptanalysis,
//! PDF manuals indexing, Ghidra Jython code generation, and Panadiga P7501 Async UART Pico host protocol.

#[macro_use]
pub mod logger;

pub mod constants;
pub mod crypto;
pub mod ghidra;
pub mod hdd;
pub mod headers;
pub mod manuals;
pub mod nvram;
pub mod pico;
pub mod scan;
pub mod ssot;

pub use logger::LogLevel;
