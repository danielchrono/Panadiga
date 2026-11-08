//! Panasonic DMR-EH55 DIGA Suite - Pico & DWire32A Domain Types
//!
//! Strongly-typed domain models, hardware constants, and pure utility functions.

use serde::{Deserialize, Serialize};

/// Raspberry Pi USB Vendor ID
pub const RASPBERRY_PI_VID: u16 = 0x2E8A;
/// Standard Raspberry Pi Pico (RP2040) USB CDC Serial Product ID
pub const PICO_RP2040_CDC_PID: u16 = 0x000A;
/// Standard Raspberry Pi Pico 2 (RP2350) USB CDC Serial Product ID
pub const PICO_RP2350_CDC_PID: u16 = 0x000F;
/// Raspberry Pi BOOTSEL Mode Product ID
pub const PICO_BOOTSEL_PID: u16 = 0x0003;

/// Diagnostic connection and telemetry response from the Pico controller.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PicoStatusResponse {
    pub connected: bool,
    pub port_name: String,
    pub baud_rate: u32,
    pub vdd_voltage: f32,
    pub target_powered: bool,
    pub firmware_banner: String,
    pub is_mock: bool,
    pub message: String,
}

/// Current status of the GDB TCP-to-Serial bridge server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GdbBridgeStatus {
    pub running: bool,
    pub port_name: String,
    pub listen_addr: String,
    pub connected_clients: u32,
    pub last_event: String,
}

/// AM34 CPU register file state for UI inspection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Am34RegisterState {
    pub d: [u32; 4],
    pub a: [u32; 4],
    pub mdr: u32,
    pub psw: u32,
    pub pc: u32,
    pub sp: u32,
    pub is_halted: bool,
}

/// Pure helper function to normalize port name string, extracting "COMx" or detecting mock mode.
pub fn extract_port_name(input: &str) -> String {
    let trimmed = input.trim();
    if trimmed.to_uppercase().contains("MOCK") {
        return "MOCK".to_string();
    }
    trimmed
        .split_whitespace()
        .next()
        .unwrap_or(trimmed)
        .to_string()
}
