//! Multi-Platform Host Controller (RP2040 / RP2350 / RP2 Family) Serial & GDB RSP Bridge Module
//!
//! Provides cross-platform hardware discovery, diagnostic telemetry probing, and a TCP-to-Serial
//! GDB Remote Serial Protocol (RSP) bridge connecting high-level debuggers
//! (mn10300-elf-gdb, gdb-multiarch, Ghidra, IDA Pro) to the Panasonic DWire32A OCD bus.
//! Supports Raspberry Pi Pico/Pico W, Pico 2 (ARM & RISC-V), Arduino Nano RP2040 Connect,
//! Seeed XIAO, Adafruit, SparkFun, and Waveshare RP2-family controllers across Windows, Linux, and macOS.

pub mod bridge;
pub mod diagnostics;
pub mod flasher;
pub mod platforms;

pub use bridge::run_gdb_bridge;
pub use diagnostics::ping_pico;
pub use flasher::*;
pub use platforms::*;
