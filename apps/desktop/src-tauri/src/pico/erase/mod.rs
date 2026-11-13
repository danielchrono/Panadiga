//! Panasonic DMR-EH55 DIGA Suite - In-Circuit NVRAM Erase & Live Secret Dumper
//!
//! Subsystem Module Decomposition (SRP):
//! - `parser`: Pure parsing of P7501 / memory dump streams and auto-detected NVRAM bases
//! - `backup`: First-contact read-only secrets backup, double-read SHA-256 verification
//! - `executor`: In-circuit RAM loader injection and Top-Boot 64KB EEMUL sector erase

pub mod backup;
pub mod executor;
pub mod parser;

pub use backup::pico_execute_first_contact_backup;
pub use executor::{pico_execute_in_circuit_erase, pico_execute_in_circuit_erase_full};
pub use parser::{parse_autodetected_nvram_base, parse_pico_dump_hex_to_bytes};
