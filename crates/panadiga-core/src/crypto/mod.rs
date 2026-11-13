//! Cryptographic Forensics, Entropy Analysis & ECB Block Frequency Leak Detection
//!
//! Analyzes encrypted Panasonic container formats (`PANA_DVD.FRM`) and correlates
//! 64-bit Feistel cipher blocks with HDD partition structures.

pub mod codebook;
pub mod entropy;
pub mod toc;
pub mod types;

pub use codebook::{correlate_hdd_partitions, print_codebook};
pub use entropy::{analyze_8byte_frequencies, analyze_frm_container};
pub use toc::{analyze_toc, dump_toc};
pub use types::*;
