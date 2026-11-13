//! Cryptographic Forensics, Entropy Analysis & ECB Block Frequency Leak Detection
//!
//! Analyzes encrypted Panasonic container formats (`PANA_DVD.FRM`) and correlates
//! 64-bit Feistel cipher blocks with HDD partition structures.

pub mod codebook;
pub mod entropy;
pub mod feistel;
pub mod toc;
pub mod types;

pub use codebook::{correlate_hdd_partitions, print_codebook};
pub use entropy::{analyze_8byte_frequencies, analyze_frm_container};
pub use feistel::{
    adler32_panasonic_strided, derive_round_key_32b, feistel_decrypt_block, feistel_decrypt_buffer,
    feistel_encrypt_block, feistel_encrypt_buffer, feistel_f_round, reconstruct_master_key_7b,
};
pub use toc::{analyze_toc, decrypt_and_parse_frm_toc, dump_toc};
pub use types::*;
