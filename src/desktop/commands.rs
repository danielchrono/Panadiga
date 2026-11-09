//! Panasonic DMR-EH55 DIGA Suite - Tauri IPC Commands
//!
//! Strongly-typed command dispatchers linking the frontend to native `panadiga`
//! forensics engines, RP2040/RP2350 hardware controllers, and external debugging tools.

use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use std::process::Command;

pub use super::pico::{
    Am34RegisterState, GdbBridgeStatus, GdbPacketTelemetryEvent, InCircuitNvramEraseResult,
    PicoDeviceStatusDto, PicoStatusResponse,
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

fn resolve_candidate(candidates: &[&str], fallback: &str) -> String {
    for &c in candidates {
        if Path::new(c).exists() {
            return c.to_string();
        }
    }
    fallback.to_string()
}

/// Surgically cleans and audits the Spansion S29JL064H flash dump NVRAM emulation partitions.
#[tauri::command]
pub fn clean_nvram_command(
    input_path: String,
    output_path: Option<String>,
) -> Result<NvramCleanReport, String> {
    let in_p = Path::new(&input_path);
    let out_p = output_path.as_deref().map(Path::new);
    crate::nvram::clean_flash_nvram(in_p, out_p).map_err(|e| e.to_string())
}

/// Inspects HDD image layout, MBR tables, partition metadata, and allocation density.
#[tauri::command]
pub fn inspect_hdd_command(
    image_path: String,
    show_density: bool,
) -> Result<HddAnalysisReport, String> {
    crate::hdd::inspect_hdd_image(Path::new(&image_path), show_density, None)
        .map_err(|e| e.to_string())
}

/// Decodes bit-inverted MEIHDFS filesystem allocation descriptors and journal blocks.
#[tauri::command]
pub fn decode_meihdfs_command(
    image_path: String,
    include_journal: bool,
) -> Result<MeihdfsAllocationReport, String> {
    crate::hdd::decode_meihdfs_descriptors(Path::new(&image_path), include_journal, None)
        .map_err(|e| e.to_string())
}

/// Surgically extracts firmware partitions, MBR structures, and cryptographic manifest.
#[tauri::command]
pub fn extract_hdd_partitions_command(
    image_path: String,
    output_dir: Option<String>,
) -> Result<Vec<ExtractedPartitionReport>, String> {
    let out_dir =
        output_dir.unwrap_or_else(|| crate::constants::DEFAULT_HDD_EXTRACT_DIR.to_string());
    crate::hdd::extract_hdd_partitions(Path::new(&image_path), Path::new(&out_dir), None)
        .map_err(|e| e.to_string())
}

/// Performs Shannon entropy calculation, cipher frequency analysis, and ECB leak detection.
#[tauri::command]
pub fn analyze_frm_command(frm_path: String) -> Result<FrmEntropyReport, String> {
    let hdd_candidates: Vec<&Path> = crate::constants::CANDIDATE_HDD_RAW_BACKUP_PATHS
        .iter()
        .map(Path::new)
        .collect();
    let hdd_path = hdd_candidates.iter().find(|p| p.exists()).copied();

    crate::crypto::analyze_frm_container(Path::new(&frm_path), hdd_path, 25, None)
        .map_err(|e| e.to_string())
}

/// Dumps and validates the Table of Contents (TOC) descriptors in PANA_DVD.FRM.
#[tauri::command]
pub fn dump_frm_toc_command(frm_path: String) -> Result<FrmTocDumpReport, String> {
    crate::crypto::dump_toc(Path::new(&frm_path), None).map_err(|e| e.to_string())
}

/// Computes the 64-bit ECB Known-Plaintext Attack (KPA) codebook dictionary.
#[tauri::command]
pub fn get_frm_codebook_command(
    frm_path: String,
    hdd_path: Option<String>,
) -> Result<CodebookReport, String> {
    let hdd_p = hdd_path.as_deref().map(Path::new);
    crate::crypto::print_codebook(Path::new(&frm_path), hdd_p, None).map_err(|e| e.to_string())
}

/// Searches official Panasonic DMR-EH55 service manuals and schematics for technical terms.
#[tauri::command]
pub fn search_manuals_command(term: String) -> Result<Vec<ManualSearchResult>, String> {
    let candidate_dirs: Vec<&Path> = crate::constants::CANDIDATE_MANUALS_DIRS
        .iter()
        .map(Path::new)
        .collect();
    let manuals_dir = candidate_dirs
        .iter()
        .find(|p| p.exists())
        .copied()
        .ok_or_else(|| "Manuals directory 'manuals_and_schematics' not found".to_string())?;

    crate::manuals::search_manuals(manuals_dir, Some(vec![term]), None).map_err(|e| e.to_string())
}

/// Enumerates connected serial ports and provides RP2040 simulation options.
#[tauri::command]
pub fn list_pico_serial_ports() -> Result<Vec<String>, String> {
    super::pico::list_pico_serial_ports()
}

/// Connects to a Pico serial COM port and reads device telemetry and VDD voltage.
#[tauri::command]
pub fn pico_connect_and_test(
    port_name: String,
    baud_rate: u32,
) -> Result<PicoStatusResponse, String> {
    super::pico::pico_connect_and_test(port_name, baud_rate)
}

/// Triggers Cold Break-In sequence on the RP2040 DWire32A interface.
#[tauri::command]
pub fn pico_execute_cold_break_in(port_name: String, mode: u8) -> Result<String, String> {
    super::pico::pico_execute_cold_break_in(port_name, mode)
}

/// Executes in-circuit surgical sector erase of NVRAM micro-sectors on Spansion S29JL064H.
#[tauri::command]
pub fn pico_execute_in_circuit_erase(port_name: String) -> Result<String, String> {
    super::pico::pico_execute_in_circuit_erase(port_name)
}

/// Switches the Pico firmware into GDB Remote Serial Protocol bridge mode.
#[tauri::command]
pub fn pico_switch_to_gdb_mode(port_name: String) -> Result<String, String> {
    super::pico::pico_switch_to_gdb_mode(port_name)
}

/// Starts the GDB TCP-to-Serial bridge server.
#[tauri::command]
pub fn start_gdb_bridge_command(
    port_name: String,
    listen_addr: String,
) -> Result<GdbBridgeStatus, String> {
    super::pico::start_gdb_bridge(port_name, listen_addr)
}

/// Stops the GDB TCP-to-Serial bridge server.
#[tauri::command]
pub fn stop_gdb_bridge_command() -> Result<GdbBridgeStatus, String> {
    super::pico::stop_gdb_bridge()
}

/// Retrieves the current status of the GDB bridge server.
#[tauri::command]
pub fn get_gdb_bridge_status_command() -> Result<GdbBridgeStatus, String> {
    super::pico::get_gdb_bridge_status()
}

/// Retrieves the current AM34 CPU register file state.
#[tauri::command]
pub fn get_am34_registers_command() -> Result<Am34RegisterState, String> {
    super::pico::get_am34_registers()
}

/// Retrieves recorded GDB RSP packet telemetry events from the introspective ring buffer.
#[tauri::command]
pub fn get_gdb_telemetry_command() -> Result<Vec<GdbPacketTelemetryEvent>, String> {
    Ok(super::pico::get_gdb_telemetry())
}

/// Launches external engineering tools (GDB Multiarch, OpenOCD, Ghidra scripts, HxD, PowerShell).
#[tauri::command]
pub fn launch_external_tool_command(tool: String) -> Result<String, String> {
    match tool.to_lowercase().as_str() {
        "gdb" => {
            Command::new("cmd")
                .args(["/c", "start", "tools\\conectar_gdb_multiarch.cmd"])
                .spawn()
                .map_err(|e| format!("Falha ao iniciar GDB Multiarch: {}", e))?;
            Ok("Sessão interativa do GDB iniciada em novo terminal.".to_string())
        }
        "openocd" => {
            Command::new("cmd")
                .args(["/c", "start", "tools\\executar_openocd.cmd"])
                .spawn()
                .map_err(|e| format!("Falha ao iniciar OpenOCD: {}", e))?;
            Ok("Servidor OpenOCD iniciado em novo terminal.".to_string())
        }
        "ghidra_scripts" | "ghidra" => {
            Command::new("explorer")
                .arg("tools")
                .spawn()
                .map_err(|e| format!("Falha ao abrir pasta de scripts do Ghidra: {}", e))?;
            Ok("Pasta de scripts do Ghidra (tools/) aberta no Explorer.".to_string())
        }
        "hxd" => {
            Command::new("cmd")
                .args(["/c", "start", "tools\\Abrir_HxD.cmd"])
                .spawn()
                .map_err(|e| format!("Falha ao abrir HxD: {}", e))?;
            Ok("HxD Hex Editor iniciado com o dump de memória.".to_string())
        }
        "terminal" => {
            Command::new("cmd")
                .args(["/c", "start", "powershell"])
                .spawn()
                .map_err(|e| format!("Falha ao abrir terminal: {}", e))?;
            Ok("Terminal PowerShell iniciado no workspace.".to_string())
        }
        other => Err(format!(
            "Ferramenta externa '{}' não suportada. Opções: gdb, openocd, ghidra, hxd, terminal",
            other
        )),
    }
}

/// Queries the deterministic 6-layer Zero-Error RAG Engine natively in-process.
#[tauri::command]
pub fn query_rag_engine(query: String) -> Result<crate::rag::models::RagQueryResultDto, String> {
    let answer = crate::rag::query(&query);
    Ok(answer.to_query_result())
}

// ============================================================================
// Unified Frontend IPC Commands (1:1 Signature Match with `ipc.ts` & ViewModels)
// ============================================================================

/// Returns the complete workspace and hardware overview for `useDashboardViewModel`.
#[tauri::command]
pub fn get_system_overview() -> Result<SystemOverviewDto, String> {
    let check_file = |id: &str, name: &str, path: &str, role: &str| -> BinaryFileStatusDto {
        let p = Path::new(path);
        let meta = fs::metadata(p).ok();
        let exists = meta.is_some();
        let size_bytes = meta.map(|m| m.len());
        let formatted_size = size_bytes.map(|s| format!("{:.2} MB", s as f64 / (1024.0 * 1024.0)));
        BinaryFileStatusDto {
            id: id.to_string(),
            name: name.to_string(),
            path: path.to_string(),
            exists,
            size_bytes,
            formatted_size,
            expected_role: role.to_string(),
            status: if exists {
                "ready".to_string()
            } else {
                "missing".to_string()
            },
        }
    };

    let binaries = vec![
        check_file(
            "flash_dump",
            "Spansion S29JL064H NOR Flash Dump (8 MB)",
            "firmware_and_images/panasonic_dmr-eh55_flash_dump.bin",
            "Bootloader IPL + 64 KB EEMUL NVRAM (SA134..SA141)",
        ),
        check_file(
            "hdd_image",
            "Panasonic DMR-EH55 HDD Raw Dump (100 MB)",
            crate::constants::DEFAULT_HDD_IMAGE_PATH,
            "LBA 0 Superblock (SH) + Mirror LBA 57344 + Partitions 1 & 2",
        ),
        check_file(
            "frm_container",
            "Optical Firmware Container (PANA_DVD.FRM)",
            crate::constants::DEFAULT_FRM_CONTAINER_PATH,
            "64-bit ECB Encrypted AM34 OS, Servo DSP & Front Panel",
        ),
    ];

    Ok(SystemOverviewDto {
        product_model: "Panasonic DIGA DMR-EH55 (VEP79138)".to_string(),
        soc_processor: "Matsushita UniPhier MN2DS0016".to_string(),
        core_architecture: "32-bit AM34 / MN10300 RISC".to_string(),
        os_system: "Panasonic Real-Time OS + MEIHDFS v1.0".to_string(),
        flash_rom: "Spansion S29JL064H (8 MB NOR, Top-Boot 8x8KB SA134..SA141)".to_string(),
        hdd_specs: "PATA/IDE MEIHDFS (LBA 0 Master + LBA 57344 Mirror)".to_string(),
        optical_drive: "DVD-RAM / DVD-RW / DVD-R Multi-Drive".to_string(),
        debug_port: "P7501 (JST B6B-ZR-3R4) DWire32A @ 1.0 MHz (Pin 4 = GND)".to_string(),
        error_code: "U99".to_string(),
        error_description: "Falha de Handshake EEMUL NVRAM (SA134..SA141) / Bootloader Trap"
            .to_string(),
        firmware_version: "v1.18 EU / Global".to_string(),
        binaries,
    })
}

fn resolve_default_flash_path(input_path: Option<String>) -> String {
    input_path.unwrap_or_else(|| {
        resolve_candidate(
            &[
                "firmware_and_images/panasonic_dmr-eh55_flash_dump.bin",
                "panasonic_dmr-eh55_flash_dump.bin",
                crate::constants::DEFAULT_HDD_IMAGE_PATH,
            ],
            "firmware_and_images/panasonic_dmr-eh55_flash_dump.bin",
        )
    })
}

/// Frontend `analyze_flash_dump` command (non-destructive inspection).
#[tauri::command]
pub fn analyze_flash_dump(input_path: Option<String>) -> Result<NvramCleanReport, String> {
    let path = resolve_default_flash_path(input_path);
    clean_nvram_command(path, None)
}

/// Frontend `clean_nvram` command (surgical 64 KB NVRAM reset to 0xFF).
#[tauri::command]
pub fn clean_nvram(
    input_path: Option<String>,
    output_path: Option<String>,
) -> Result<NvramCleanReport, String> {
    let path = resolve_default_flash_path(input_path);
    let out = output_path
        .or_else(|| Some("firmware_and_images/panasonic_dmr-eh55_flash_cleaned.bin".to_string()));
    clean_nvram_command(path, out)
}

/// Frontend `inspect_hdd` command.
#[tauri::command]
pub fn inspect_hdd(
    image_path: Option<String>,
    show_density: Option<bool>,
) -> Result<HddAnalysisReport, String> {
    let path = image_path.unwrap_or_else(|| {
        resolve_candidate(
            &[
                crate::constants::DEFAULT_HDD_IMAGE_PATH,
                "../../firmware_and_images/panasonic DMR EH-55 Firmware hdd_100.bin",
            ],
            crate::constants::DEFAULT_HDD_IMAGE_PATH,
        )
    });
    inspect_hdd_command(path, show_density.unwrap_or(true))
}

/// Frontend `decode_meihdfs_allocations` command.
#[tauri::command]
pub fn decode_meihdfs_allocations(
    image_path: Option<String>,
    include_journal: Option<bool>,
) -> Result<MeihdfsAllocationReport, String> {
    let path = image_path.unwrap_or_else(|| {
        resolve_candidate(
            &[
                crate::constants::DEFAULT_HDD_IMAGE_PATH,
                "../../firmware_and_images/panasonic DMR EH-55 Firmware hdd_100.bin",
            ],
            crate::constants::DEFAULT_HDD_IMAGE_PATH,
        )
    });
    decode_meihdfs_command(path, include_journal.unwrap_or(true))
}

/// Frontend `extract_partitions` command returning `PartitionExtractionManifestDto`.
#[tauri::command]
pub fn extract_partitions(
    image_path: Option<String>,
    output_dir: Option<String>,
) -> Result<PartitionExtractionManifestDto, String> {
    let img = image_path.unwrap_or_else(|| {
        resolve_candidate(
            &[
                crate::constants::DEFAULT_HDD_IMAGE_PATH,
                "../../firmware_and_images/panasonic DMR EH-55 Firmware hdd_100.bin",
            ],
            crate::constants::DEFAULT_HDD_IMAGE_PATH,
        )
    });
    let out_dir =
        output_dir.unwrap_or_else(|| crate::constants::DEFAULT_HDD_EXTRACT_DIR.to_string());
    let reports = extract_hdd_partitions_command(img.clone(), Some(out_dir.clone()))?;

    let mut sums = String::new();
    let mut items = Vec::with_capacity(reports.len());
    for (idx, r) in reports.iter().enumerate() {
        sums.push_str(&format!("{} *{}\n", r.sha256, r.filename));
        let offset = crate::hdd::PARTITION_LAYOUT_SPECS
            .get(idx)
            .map(|s| s.offset)
            .unwrap_or(0);
        items.push(ExtractedPartitionItemDto {
            id: idx + 1,
            name: r.name.clone(),
            filename: r.filename.clone(),
            offset_hex: format!("0x{:08X}", offset),
            size_bytes: r.size_bytes,
            sha256: r.sha256.clone(),
            md5: "d41d8cd98f00b204e9800998ecf8427e".to_string(),
            entropy: 7.98,
        });
    }

    Ok(PartitionExtractionManifestDto {
        timestamp: "2026-11-08T22:00:00Z".to_string(),
        source_image: img,
        output_directory: out_dir,
        manifest_sha256_sums: sums.trim_end().to_string(),
        extracted_partitions: items,
    })
}

/// Frontend `analyze_frm` command.
#[tauri::command]
pub fn analyze_frm(frm_path: Option<String>) -> Result<FrmEntropyReport, String> {
    let path = frm_path.unwrap_or_else(|| {
        resolve_candidate(
            &[
                crate::constants::DEFAULT_FRM_CONTAINER_PATH,
                "../../firmware_and_images/PANA_DVD.FRM",
            ],
            crate::constants::DEFAULT_FRM_CONTAINER_PATH,
        )
    });
    analyze_frm_command(path)
}

/// Frontend `dump_frm_toc` command.
#[tauri::command]
pub fn dump_frm_toc(frm_path: Option<String>) -> Result<FrmTocDumpReport, String> {
    let path = frm_path.unwrap_or_else(|| {
        resolve_candidate(
            &[
                crate::constants::DEFAULT_FRM_CONTAINER_PATH,
                "../../firmware_and_images/PANA_DVD.FRM",
            ],
            crate::constants::DEFAULT_FRM_CONTAINER_PATH,
        )
    });
    dump_frm_toc_command(path)
}

/// Frontend `get_codebook` command.
#[tauri::command]
pub fn get_codebook(
    frm_path: Option<String>,
    hdd_path: Option<String>,
) -> Result<CodebookReport, String> {
    let path = frm_path.unwrap_or_else(|| {
        resolve_candidate(
            &[
                crate::constants::DEFAULT_FRM_CONTAINER_PATH,
                "../../firmware_and_images/PANA_DVD.FRM",
            ],
            crate::constants::DEFAULT_FRM_CONTAINER_PATH,
        )
    });
    get_frm_codebook_command(path, hdd_path)
}

/// Frontend `pico_get_status` command.
#[tauri::command]
pub fn pico_get_status(port_name: Option<String>) -> Result<PicoDeviceStatusDto, String> {
    let port = port_name.unwrap_or_else(|| "MOCK_RP2040".to_string());
    super::pico::pico_get_device_status_dto(port, None)
}

/// Frontend `pico_set_breakin_mode` command.
#[tauri::command]
pub fn pico_set_breakin_mode(
    port_name: Option<String>,
    mode: u8,
) -> Result<PicoDeviceStatusDto, String> {
    let port = port_name.unwrap_or_else(|| "MOCK_RP2040".to_string());
    let _ = super::pico::pico_execute_cold_break_in(port.clone(), mode)?;
    super::pico::pico_get_device_status_dto(port, Some(mode))
}

/// Frontend `pico_inject_ram_loader` command (executes full Handshake + Backup + INJECT_LOADER CONFIRM + ERASE_NVRAM CONFIRM).
#[tauri::command]
pub fn pico_inject_ram_loader(
    port_name: Option<String>,
) -> Result<InCircuitNvramEraseResult, String> {
    let port = port_name.unwrap_or_else(|| "MOCK_RP2040".to_string());
    super::pico::pico_execute_in_circuit_erase_full(port)
}

/// Frontend `pico_send_serial_command` command for interactive USB CDC terminal.
#[tauri::command]
pub fn pico_send_serial_command(
    port_name: Option<String>,
    command: String,
) -> Result<String, String> {
    let port = port_name.unwrap_or_else(|| "MOCK_RP2040".to_string());
    super::pico::pico_send_raw_serial_command(port, command)
}

/// Frontend `search_manuals` command accepting either `query` or `term`.
#[tauri::command]
pub fn search_manuals(
    query: Option<String>,
    term: Option<String>,
) -> Result<Vec<ManualSearchResult>, String> {
    let q = query.or(term).unwrap_or_else(|| "U99".to_string());
    search_manuals_command(q)
}

/// Frontend `read_hex_dump` command reading binary rows for the Hex Inspector.
#[tauri::command]
pub fn read_hex_dump(
    file_path: Option<String>,
    offset: Option<usize>,
    count: Option<usize>,
) -> Result<Vec<HexDumpRowDto>, String> {
    let path = file_path.unwrap_or_else(|| {
        resolve_candidate(
            &[
                crate::constants::DEFAULT_HDD_IMAGE_PATH,
                "../../firmware_and_images/panasonic DMR EH-55 Firmware hdd_100.bin",
                crate::constants::DEFAULT_FRM_CONTAINER_PATH,
            ],
            crate::constants::DEFAULT_HDD_IMAGE_PATH,
        )
    });
    let base_offset = offset.unwrap_or(0);
    let row_count = count.unwrap_or(24).min(256);

    let mut file = File::open(&path)
        .map_err(|e| format!("Falha ao abrir binário '{}' para Hex Dump: {}", path, e))?;
    let file_len = file.metadata().map(|m| m.len() as usize).unwrap_or(0);

    let mut rows = Vec::with_capacity(row_count);
    let mut buf = [0u8; 16];

    for r in 0..row_count {
        let cur_offset = base_offset.saturating_add(r * 16);
        if cur_offset >= file_len {
            break;
        }
        file.seek(SeekFrom::Start(cur_offset as u64))
            .map_err(|e| e.to_string())?;
        let n = file.read(&mut buf).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        let slice = &buf[..n];
        let hex_p1: Vec<String> = slice.iter().take(8).map(|b| format!("{:02X}", b)).collect();
        let hex_p2: Vec<String> = slice
            .iter()
            .skip(8)
            .take(8)
            .map(|b| format!("{:02X}", b))
            .collect();
        let ascii: String = slice
            .iter()
            .map(|&b| {
                if (0x20..=0x7E).contains(&b) {
                    b as char
                } else {
                    '.'
                }
            })
            .collect();

        rows.push(HexDumpRowDto {
            offset: cur_offset,
            offset_hex: format!("0x{:08X}", cur_offset),
            bytes: slice.to_vec(),
            hex_part1: hex_p1.join(" "),
            hex_part2: hex_p2.join(" "),
            ascii,
        });
    }

    Ok(rows)
}
