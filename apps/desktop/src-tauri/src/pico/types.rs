//! Panasonic DMR-EH55 DIGA Suite - Pico P7501 Async UART Domain Types
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

/// Telemetry event recorded by the introspective GDB RSP bridge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GdbPacketTelemetryEvent {
    pub timestamp_ms: u64,
    pub direction: String,
    pub command_type: String,
    pub raw_packet: String,
    pub checksum_valid: bool,
}

/// Complete Pico device status matching Frontend `PicoDeviceStatus` model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PicoDeviceStatusDto {
    pub connected: bool,
    pub port: String,
    #[serde(rename = "baudRate")]
    pub baud_rate: u32,
    pub vdd_volts: f32,
    pub target_powered: bool,
    pub breakin_mode: u8,
    pub core_state: String,
    pub bus_freq_hz: u32,
    pub firmware_version: String,
    pub chip_id: String,
    pub total_probes: u32,
    pub hits_detected: u32,
    pub open_bus_high: u32,
    pub open_bus_low: u32,
    pub ram_loader_injected: bool,
    pub last_keepalive: String,
}

/// Result of in-circuit RAM loader injection and NVRAM sector erasure matching Frontend `InCircuitNvramEraseResult`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InCircuitNvramEraseResult {
    pub success: bool,
    pub elapsed_ms: u64,
    pub voltage_before: f32,
    pub voltage_after: f32,
    pub ram_loader_addr: String,
    pub bytes_erased: u32,
    pub verification_sha256: String,
    pub log_lines: Vec<String>,
}

/// Result of the non-destructive "What's Next" 1st-Contact Read-Only Backup & Auto-Model Probe.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirstContactBackupResult {
    pub success: bool,
    pub detected_model: String,
    pub nvram_bytes: usize,
    pub nvram_sha256: String,
    pub double_read_verified: bool,
    pub sram_bytes: usize,
    pub ipl_bytes: usize,
    pub log_lines: Vec<String>,
}

/// Pure helper function to normalize port name string, extracting "COMx" or device path.
pub fn extract_port_name(input: &str) -> String {
    let trimmed = input.trim();
    trimmed
        .split_whitespace()
        .next()
        .unwrap_or(trimmed)
        .to_string()
}

/// Enforces production hardware-only policy by rejecting legacy `MOCK*` identifiers.
pub fn reject_mock_port(clean_port: &str) -> Result<(), String> {
    if clean_port.trim().to_ascii_uppercase().starts_with("MOCK") {
        return Err(
            "Simulador MOCK desativado em produção: conecte o Raspberry Pi Pico real via USB CDC / BLE NUS."
                .to_string(),
        );
    }
    Ok(())
}
