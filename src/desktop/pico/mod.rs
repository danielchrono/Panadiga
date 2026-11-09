//! Panasonic DMR-EH55 DIGA Suite - Pico & DWire32A Hardware Subsystem
//!
//! Modular architecture organized by Single Responsibility Principle (SRP) and Separation of Concerns (SoC):
//! - `types`: Domain models, enums, constants, and pure helpers
//! - `rsp_protocol`: Pure functional GDB RSP packet framing, checksums, and mock emulation
//! - `gdb_bridge`: Imperative TCP-to-Serial GDB RSP proxy server and shadow register controller
//! - `serial`: USB CDC serial communication, port enumeration, telemetry, and cold break-in
//! - `erase`: In-circuit NVRAM sector erasure and pre-erase live secret dumper

pub mod erase;
pub mod gdb_bridge;
pub mod rsp_protocol;
pub mod serial;
pub mod types;

pub use erase::*;
pub use gdb_bridge::*;
pub use rsp_protocol::*;
pub use serial::*;
pub use types::*;
