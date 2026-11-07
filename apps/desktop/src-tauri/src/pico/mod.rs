//! Panasonic DMR-EH55 DIGA Suite - Pico & DWire32A Hardware Subsystem
//!
//! Modular architecture organized by single responsibility:
//! - `types`: Domain models, enums, constants, and pure helpers
//! - `serial`: USB CDC serial communication, break-in, and in-circuit erase
//! - `gdb_bridge`: TCP-to-Serial GDB Remote Serial Protocol (RSP) proxy server

pub mod gdb_bridge;
pub mod serial;
pub mod types;

pub use gdb_bridge::*;
pub use serial::*;
pub use types::*;
