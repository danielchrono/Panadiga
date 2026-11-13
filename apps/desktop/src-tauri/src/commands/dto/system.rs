//! Desktop DTOs for workspace binary inventory, system overview, and hex views.

use serde::{Deserialize, Serialize};

/// Frontend-compatible binary file status entry for `get_system_overview`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinaryFileStatusDto {
    pub id: String,
    pub name: String,
    pub path: String,
    pub exists: bool,
    pub size_bytes: Option<u64>,
    pub formatted_size: Option<String>,
    pub expected_role: String,
    pub status: String,
}

/// Frontend-compatible system overview DTO for `get_system_overview`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemOverviewDto {
    pub product_model: String,
    pub soc_processor: String,
    pub core_architecture: String,
    pub os_system: String,
    pub flash_rom: String,
    pub hdd_specs: String,
    pub optical_drive: String,
    pub debug_port: String,
    pub error_code: String,
    pub error_description: String,
    pub firmware_version: String,
    pub binaries: Vec<BinaryFileStatusDto>,
}

/// Frontend-compatible hex dump row for `read_hex_dump`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HexDumpRowDto {
    pub offset: usize,
    pub offset_hex: String,
    pub bytes: Vec<u8>,
    pub hex_part1: String,
    pub hex_part2: String,
    pub ascii: String,
}

/// Catalog entry for a real Panasonic service manual PDF under `manuals_and_schematics/`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceManualCatalogItemDto {
    pub file_name: String,
    pub relative_path: String,
    pub size_bytes: u64,
    pub formatted_size: String,
}

/// Live NVRAM sector capture event streamed from Pico (P7501 UART) / Android USB-OTG / BLE.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveNvramCaptureEventDto {
    pub source: String,
    pub port: String,
    pub base_bus_address: String,
    pub captured_bytes: usize,
    pub report: panadiga_core::nvram::NvramCleanReport,
}
