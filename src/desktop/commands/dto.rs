//! Panasonic DMR-EH55 DIGA Suite - Desktop IPC Data Transfer Objects (DTOs)
//!
//! Single Responsibility: Defines strongly-typed serialization structures matching
//! the React/TypeScript Frontend models (`apps/desktop/src/models/`) and pure
//! candidate file path resolution helpers.

use serde::{Deserialize, Serialize};
use std::path::Path;

pub use super::super::pico::{
    Am34RegisterState, FirstContactBackupResult, GdbBridgeStatus, GdbPacketTelemetryEvent,
    InCircuitNvramEraseResult, PicoDeviceStatusDto, PicoStatusResponse,
};
pub use crate::crypto::{
    CodebookReport, CryptanalysisReport as FrmEntropyReport, FrmTocDumpReport,
};
pub use crate::hdd::{ExtractedPartitionReport, HddAnalysisReport, MeihdfsAllocationReport};
pub use crate::manuals::PdfSearchResult as ManualSearchResult;
pub use crate::nvram::NvramCleanReport;

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

/// Frontend-compatible extracted partition item for `extract_partitions`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedPartitionItemDto {
    pub id: usize,
    pub name: String,
    pub filename: String,
    #[serde(rename = "offsetHex")]
    pub offset_hex: String,
    #[serde(rename = "sizeBytes")]
    pub size_bytes: usize,
    pub sha256: String,
    pub md5: String,
    pub entropy: f64,
}

/// Frontend-compatible partition extraction manifest for `extract_partitions`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartitionExtractionManifestDto {
    pub timestamp: String,
    #[serde(rename = "sourceImage")]
    pub source_image: String,
    #[serde(rename = "outputDirectory")]
    pub output_directory: String,
    #[serde(rename = "manifestSha256Sums")]
    pub manifest_sha256_sums: String,
    #[serde(rename = "extractedPartitions")]
    pub extracted_partitions: Vec<ExtractedPartitionItemDto>,
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

/// Strongly-typed profile entry in the internal Panasonic DIGA Model Database (SSOT).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigaModelProfileDto {
    pub model_id: String,
    pub series: String,
    pub flash_size_mb: u32,
    pub flash_size_bytes: u64,
    pub nvram_base_hex: String,
    pub sector_geometry: String,
    pub soc_name: String,
}

const fn model_spec(
    model_id: &'static str,
    series: &'static str,
    flash_size_mb: u32,
    nvram_base_hex: &'static str,
    sector_geometry: &'static str,
    soc_name: &'static str,
) -> (
    &'static str,
    &'static str,
    u32,
    &'static str,
    &'static str,
    &'static str,
) {
    (
        model_id,
        series,
        flash_size_mb,
        nvram_base_hex,
        sector_geometry,
        soc_name,
    )
}

/// Complete internal database of Panasonic DIGA recorders across all 4 NOR Flash geometries (2MB/4MB/8MB/16MB).
pub fn get_diga_model_catalog_dto() -> Vec<DigaModelProfileDto> {
    let specs = [
        model_spec(
            "AUTO",
            "Auto-Detect (Passive Address Aliasing / Binary Signature)",
            8,
            "AUTO (0x801F0000 / 0x803F0000 / 0x807F0000 / 0x80FE0000)",
            "Auto-Detected Top-Boot / Uniform",
            "UniPhier AM34 / MN103S Auto",
        ),
        // 8 MB Top-Boot (2006+)
        model_spec(
            "DMR-EH55",
            "Série EH (HDD+DVD)",
            8,
            "0x807F0000",
            "Top-Boot 8x8KB (SA134..SA141)",
            "MN2DS0016",
        ),
        model_spec(
            "DMR-EH56",
            "Série EH (HDD+DVD)",
            8,
            "0x807F0000",
            "Top-Boot 8x8KB (SA134..SA141)",
            "MN2DS0016",
        ),
        model_spec(
            "DMR-EH65",
            "Série EH (HDD+DVD)",
            8,
            "0x807F0000",
            "Top-Boot 8x8KB (SA134..SA141)",
            "MN2DS0016",
        ),
        model_spec(
            "DMR-EH57",
            "Série EH (HDD+DVD)",
            8,
            "0x807F0000",
            "Top-Boot 8x8KB (SA134..SA141)",
            "MN2DS0018",
        ),
        model_spec(
            "DMR-EH58",
            "Série EH (HDD+DVD)",
            8,
            "0x807F0000",
            "Top-Boot 8x8KB (SA134..SA141)",
            "MN2DS0018",
        ),
        model_spec(
            "DMR-EH67",
            "Série EH (HDD+DVD)",
            8,
            "0x807F0000",
            "Top-Boot 8x8KB (SA134..SA141)",
            "MN2DS0018",
        ),
        model_spec(
            "DMR-EH68",
            "Série EH (HDD+DVD)",
            8,
            "0x807F0000",
            "Top-Boot 8x8KB (SA134..SA141)",
            "MN2DS0018",
        ),
        model_spec(
            "DMR-ES15",
            "Série ES (DVD/Combo)",
            8,
            "0x807F0000",
            "Top-Boot 8x8KB (SA134..SA141)",
            "MN2DS0016",
        ),
        model_spec(
            "DMR-ES25",
            "Série ES (DVD/Combo)",
            8,
            "0x807F0000",
            "Top-Boot 8x8KB (SA134..SA141)",
            "MN2DS0016",
        ),
        model_spec(
            "DMR-ES35V",
            "Série ES (DVD+VHS)",
            8,
            "0x807F0000",
            "Top-Boot 8x8KB (SA134..SA141)",
            "MN2DS0016",
        ),
        model_spec(
            "DMR-EZ25",
            "Série EZ (DVB-T DVD)",
            8,
            "0x807F0000",
            "Top-Boot 8x8KB (SA134..SA141)",
            "MN2DS0016",
        ),
        // 4 MB Top-Boot (2005)
        model_spec(
            "DMR-ES10",
            "Série ES (DVD-RAM)",
            4,
            "0x803F0000",
            "Top-Boot 8x8KB (SA63..SA70)",
            "MN2DS0009",
        ),
        model_spec(
            "DMR-ES20",
            "Série ES (DVD-RAM)",
            4,
            "0x803F0000",
            "Top-Boot 8x8KB (SA63..SA70)",
            "MN2DS0009",
        ),
        model_spec(
            "DMR-ES30V",
            "Série ES (DVD+VHS)",
            4,
            "0x803F0000",
            "Top-Boot 8x8KB (SA63..SA70)",
            "MN2DS0009",
        ),
        model_spec(
            "DMR-EH50",
            "Série EH (HDD+DVD)",
            4,
            "0x803F0000",
            "Top-Boot 8x8KB (SA63..SA70)",
            "MN2DS0009",
        ),
        model_spec(
            "DMR-EH60",
            "Série EH (HDD+DVD)",
            4,
            "0x803F0000",
            "Top-Boot 8x8KB (SA63..SA70)",
            "MN2DS0009",
        ),
        model_spec(
            "DMR-E100H",
            "Série ExxH (HDD+DVD)",
            4,
            "0x803F0000",
            "Top-Boot 8x8KB (SA63..SA70)",
            "MN2DS0004",
        ),
        model_spec(
            "DMR-E500H",
            "Série ExxH (HDD+DVD)",
            4,
            "0x803F0000",
            "Top-Boot 8x8KB (SA63..SA70)",
            "MN2DS0004",
        ),
        // 2 MB Top-Boot (2003/2004)
        model_spec(
            "DMR-E50",
            "Série E (DVD-RAM)",
            2,
            "0x801F0000",
            "Top-Boot 8x8KB (SA31..SA38)",
            "MN2DS0004",
        ),
        model_spec(
            "DMR-E55",
            "Série E (DVD-RAM)",
            2,
            "0x801F0000",
            "Top-Boot 8x8KB (SA31..SA38)",
            "MN2DS0004",
        ),
        model_spec(
            "DMR-E65",
            "Série E (DVD-RAM)",
            2,
            "0x801F0000",
            "Top-Boot 8x8KB (SA31..SA38)",
            "MN2DS0004",
        ),
        model_spec(
            "DMR-E80H",
            "Série ExxH (HDD+DVD)",
            2,
            "0x801F0000",
            "Top-Boot 8x8KB (SA31..SA38)",
            "MN2DS0004",
        ),
        model_spec(
            "DMR-E85H",
            "Série ExxH (HDD+DVD)",
            2,
            "0x801F0000",
            "Top-Boot 8x8KB (SA31..SA38)",
            "MN2DS0004",
        ),
        model_spec(
            "DMR-E95H",
            "Série ExxH (HDD+DVD)",
            2,
            "0x801F0000",
            "Top-Boot 8x8KB (SA31..SA38)",
            "MN2DS0004",
        ),
        // 16 MB Uniform / Top-Boot 128KB (DVB-T / HDMI)
        model_spec(
            "DMR-EX75",
            "Série EX (HDD+DVB-T)",
            16,
            "0x80FE0000",
            "Uniform 128KB (SA126..SA127)",
            "MN2DS0016",
        ),
        model_spec(
            "DMR-EX85",
            "Série EX (HDD+DVB-T)",
            16,
            "0x80FE0000",
            "Uniform 128KB (SA126..SA127)",
            "MN2DS0016",
        ),
        model_spec(
            "DMR-EX77",
            "Série EX (HDD+DVB-T)",
            16,
            "0x80FE0000",
            "Uniform 128KB (SA126..SA127)",
            "MN2DS0018",
        ),
        model_spec(
            "DMR-EX87",
            "Série EX (HDD+DVB-T)",
            16,
            "0x80FE0000",
            "Uniform 128KB (SA126..SA127)",
            "MN2DS0018",
        ),
        model_spec(
            "DMR-EX95V",
            "Série EX (HDD+VHS+DVB-T)",
            16,
            "0x80FE0000",
            "Uniform 128KB (SA126..SA127)",
            "MN2DS0016",
        ),
        model_spec(
            "DMR-EZ27",
            "Série EZ (DVD+DVB-T)",
            16,
            "0x80FE0000",
            "Uniform 128KB (SA126..SA127)",
            "MN2DS0018",
        ),
        model_spec(
            "DMR-EZ45V",
            "Série EZ (DVD+VHS+DVB-T)",
            16,
            "0x80FE0000",
            "Uniform 128KB (SA126..SA127)",
            "MN2DS0016",
        ),
        model_spec(
            "DMR-EZ48V",
            "Série EZ (DVD+VHS+DVB-T)",
            16,
            "0x80FE0000",
            "Uniform 128KB (SA126..SA127)",
            "MN2DS0018",
        ),
    ];

    specs
        .into_iter()
        .map(
            |(model_id, series, flash_size_mb, nvram_base_hex, sector_geometry, soc_name)| {
                DigaModelProfileDto {
                    model_id: model_id.to_string(),
                    series: series.to_string(),
                    flash_size_mb,
                    flash_size_bytes: (flash_size_mb as u64) * 1024 * 1024,
                    nvram_base_hex: nvram_base_hex.to_string(),
                    sector_geometry: sector_geometry.to_string(),
                    soc_name: soc_name.to_string(),
                }
            },
        )
        .collect()
}

/// Resolves the first existing file path from a candidate list, or returns `fallback`.
/// Probes current working dir, parent dirs (`../`, `../../`), and relative to the running executable.
pub fn resolve_candidate(candidates: &[&str], fallback: &str) -> String {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()));

    for &c in candidates {
        let p = Path::new(c);
        if p.exists() {
            return c.to_string();
        }

        // Try one and two levels up (e.g. running from dist/ or apps/desktop/)
        let p1 = Path::new("..").join(c);
        if p1.exists() {
            return p1.to_string_lossy().to_string();
        }
        let p2 = Path::new("../..").join(c);
        if p2.exists() {
            return p2.to_string_lossy().to_string();
        }

        // Try relative to the directory containing panadiga.exe
        if let Some(ref ed) = exe_dir {
            let exe_rel = ed.join(c);
            if exe_rel.exists() {
                return exe_rel.to_string_lossy().to_string();
            }
            if let Some(parent) = ed.parent() {
                let parent_rel = parent.join(c);
                if parent_rel.exists() {
                    return parent_rel.to_string_lossy().to_string();
                }
            }
        }
    }
    fallback.to_string()
}

/// Pure check for valid Panasonic DIGA Parallel NOR Flash geometries (2 MB, 4 MB, 8 MB, 16 MB).
/// Explicitly excludes 100 MB+ HDD dumps and optical .FRM containers.
pub const fn is_valid_nor_flash_size(size_bytes: u64) -> bool {
    matches!(
        size_bytes,
        0x0020_0000 | 0x0040_0000 | 0x0080_0000 | 0x0100_0000
    )
}

/// Pure signature detector identifying the Panasonic DIGA recorder model from a binary slice or geometry.
/// Covers all Panasonic DIGA generations: E-Series (E55..E500H), ES-Series (ES10..ES35V),
/// EH-Series (EH50..EH80V), EX-Series (EX75..EX99V), and EZ-Series (EZ25..EZ49V).
pub fn detect_diga_model_from_slice(data: &[u8]) -> String {
    let known_models = [
        // Série DMR-EH (HDD + DVD Analog/Hybrid)
        "DMR-EH80V",
        "DMR-EH75V",
        "DMR-EH68",
        "DMR-EH67",
        "DMR-EH65",
        "DMR-EH60",
        "DMR-EH59",
        "DMR-EH58",
        "DMR-EH57",
        "DMR-EH56",
        "DMR-EH55",
        "DMR-EH54",
        "DMR-EH50",
        // Série DMR-ES (DVD-RAM / VHS Combo without HDD)
        "DMR-ES35V",
        "DMR-ES30V",
        "DMR-ES25",
        "DMR-ES20",
        "DMR-ES15",
        "DMR-ES10",
        // Série DMR-EX (HDD + DVD + DVB-T Digital Tuner)
        "DMR-EX99V",
        "DMR-EX98V",
        "DMR-EX95V",
        "DMR-EX89",
        "DMR-EX88",
        "DMR-EX87",
        "DMR-EX85",
        "DMR-EX79",
        "DMR-EX78",
        "DMR-EX77",
        "DMR-EX768",
        "DMR-EX75",
        // Série DMR-EZ (DVD + Digital Tuner / VHS Combo)
        "DMR-EZ49V",
        "DMR-EZ48V",
        "DMR-EZ47V",
        "DMR-EZ45V",
        "DMR-EZ37V",
        "DMR-EZ28",
        "DMR-EZ27",
        "DMR-EZ25",
        // Série DMR-E / DMR-ExxH (1ª e 2ª Geração HDD / DVD-RAM)
        "DMR-E500H",
        "DMR-E100H",
        "DMR-E95H",
        "DMR-E85H",
        "DMR-E80H",
        "DMR-E75V",
        "DMR-E65",
        "DMR-E60",
        "DMR-E55",
        "DMR-E53",
        "DMR-E50",
    ];
    for model in known_models {
        if data
            .windows(model.len())
            .any(|w| w.eq_ignore_ascii_case(model.as_bytes()))
        {
            return model.to_string();
        }
    }
    match data.len() {
        0x0020_0000 => {
            "Panasonic DIGA (2MB NOR Flash - Geração 2003/2004: DMR-E50/E55/E65 / DMR-E80H/E85H/E95H)".to_string()
        }
        0x0040_0000 => {
            "Panasonic DIGA (4MB NOR Flash - Geração 2005: DMR-ES10/ES20/ES30V / DMR-EH50/EH60 / DMR-E100H/E500H)".to_string()
        }
        0x0080_0000 => {
            "Panasonic DIGA (8MB NOR Flash - Geração 2006+: DMR-ES15/ES25/ES35V / DMR-EH55/EH56/EH65 / DMR-EZ25)".to_string()
        }
        0x0100_0000 => {
            "Panasonic DIGA (16MB NOR Flash - Geração DVB-T/HDMI: DMR-EX75/EX85/EX77/EX87 / DMR-EZ27/EZ45V/EZ48V)".to_string()
        }
        _ => "Panasonic DIGA (Auto-Detected Target)".to_string(),
    }
}

/// Automatically scans workspace directories for any valid NOR Flash dump (2MB, 4MB, 8MB, or 16MB)
/// across any Panasonic DIGA model, never falling back to a 100MB HDD image.
pub fn resolve_default_flash_path(input_path: Option<String>) -> String {
    if let Some(custom) = input_path {
        let trimmed = custom.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }

    let mut candidate_dirs = vec![
        std::path::PathBuf::from("firmware_and_images"),
        std::path::PathBuf::from("../firmware_and_images"),
        std::path::PathBuf::from("../../firmware_and_images"),
        std::path::PathBuf::from("."),
        std::path::PathBuf::from(".."),
    ];

    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            candidate_dirs.push(exe_dir.join("firmware_and_images"));
            if let Some(parent) = exe_dir.parent() {
                candidate_dirs.push(parent.join("firmware_and_images"));
            }
        }
    }

    for dir in candidate_dirs {
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    let ext_ok = path
                        .extension()
                        .and_then(|e| e.to_str())
                        .map(|e| matches!(e.to_ascii_lowercase().as_str(), "bin" | "rom" | "dump"))
                        .unwrap_or(false);
                    if ext_ok {
                        if let Ok(meta) = entry.metadata() {
                            if is_valid_nor_flash_size(meta.len()) {
                                return path.to_string_lossy().to_string();
                            }
                        }
                    }
                }
            }
        }
    }

    resolve_candidate(
        &[
            "firmware_and_images/panasonic_dmr-eh55_flash_dump.bin",
            "firmware_and_images/panasonic_dmr-eh55_flash_cleaned.bin",
            "panasonic_dmr-eh55_flash_dump.bin",
            "firmware_and_images/panasonic_flash_dump.bin",
        ],
        "firmware_and_images/panasonic_dmr-eh55_flash_dump.bin",
    )
}
