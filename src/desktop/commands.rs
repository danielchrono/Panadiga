//! Panasonic DMR-EH55 DIGA Suite - Tauri IPC Commands
//!
//! Strongly-typed command dispatchers linking the frontend to native `panadiga`
//! forensics engines, RP2040/RP2350 hardware controllers, and external debugging tools.

use std::path::Path;
use std::process::Command;

pub use super::pico::{
    Am34RegisterState, GdbBridgeStatus, GdbPacketTelemetryEvent, PicoStatusResponse,
};
pub use crate::crypto::{
    CodebookReport, CryptanalysisReport as FrmEntropyReport, FrmTocDumpReport,
};
pub use crate::hdd::{ExtractedPartitionReport, HddAnalysisReport, MeihdfsAllocationReport};
pub use crate::manuals::PdfSearchResult as ManualSearchResult;
pub use crate::nvram::NvramCleanReport;

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
                .args(["/c", "start", "tools\\conectar_gdb_multiarch.bat"])
                .spawn()
                .map_err(|e| format!("Falha ao iniciar GDB Multiarch: {}", e))?;
            Ok("Sessão interativa do GDB iniciada em novo terminal.".to_string())
        }
        "openocd" => {
            Command::new("cmd")
                .args(["/c", "start", "tools\\executar_openocd.bat"])
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
                .args(["/c", "start", "tools\\Abrir_HxD.bat"])
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
