//! Panasonic DMR-EH55 DIGA Suite - Tauri IPC Commands
//!
//! Strongly-typed command dispatchers linking the frontend to native `panadiga`
//! forensics engines and RP2040 hardware controllers.

use std::path::Path;

pub use panadiga::crypto::{
    CodebookReport, CryptanalysisReport as FrmEntropyReport, FrmTocDumpReport,
};
pub use panadiga::hdd::{ExtractedPartitionReport, HddAnalysisReport, MeihdfsAllocationReport};
pub use panadiga::manuals::PdfSearchResult as ManualSearchResult;
pub use panadiga::nvram::NvramCleanReport;
pub use crate::pico_serial::{Am34RegisterState, GdbBridgeStatus, PicoStatusResponse};

/// Surgically cleans and audits the Spansion S29JL064H flash dump NVRAM emulation partitions.
#[tauri::command]
pub fn clean_nvram_command(
    input_path: String,
    output_path: Option<String>,
) -> Result<NvramCleanReport, String> {
    let in_p = Path::new(&input_path);
    let out_p = output_path.as_deref().map(Path::new);
    panadiga::nvram::clean_flash_nvram(in_p, out_p).map_err(|e| e.to_string())
}

/// Inspects HDD image layout, MBR tables, partition metadata, and allocation density.
#[tauri::command]
pub fn inspect_hdd_command(
    image_path: String,
    show_density: bool,
) -> Result<HddAnalysisReport, String> {
    panadiga::hdd::inspect_hdd_image(Path::new(&image_path), show_density, None)
        .map_err(|e| e.to_string())
}

/// Decodes bit-inverted MEIHDFS filesystem allocation descriptors and journal blocks.
#[tauri::command]
pub fn decode_meihdfs_command(
    image_path: String,
    include_journal: bool,
) -> Result<MeihdfsAllocationReport, String> {
    panadiga::hdd::decode_meihdfs_descriptors(Path::new(&image_path), include_journal, None)
        .map_err(|e| e.to_string())
}

/// Surgically extracts firmware partitions, MBR structures, and cryptographic manifest.
#[tauri::command]
pub fn extract_hdd_partitions_command(
    image_path: String,
    output_dir: Option<String>,
) -> Result<Vec<ExtractedPartitionReport>, String> {
    let out_dir = output_dir.unwrap_or_else(|| "extracted_partitions".to_string());
    panadiga::hdd::extract_hdd_partitions(Path::new(&image_path), Path::new(&out_dir), None)
        .map_err(|e| e.to_string())
}

/// Performs Shannon entropy calculation, cipher frequency analysis, and ECB leak detection.
#[tauri::command]
pub fn analyze_frm_command(frm_path: String) -> Result<FrmEntropyReport, String> {
    let hdd_candidates = [
        Path::new("firmware_and_images/panasonic_dmr-eh55_hdd_raw_backup.bin"),
        Path::new("../../firmware_and_images/panasonic_dmr-eh55_hdd_raw_backup.bin"),
        Path::new("../../../firmware_and_images/panasonic_dmr-eh55_hdd_raw_backup.bin"),
    ];
    let hdd_path = hdd_candidates.iter().find(|p| p.exists()).copied();

    panadiga::crypto::analyze_frm_container(Path::new(&frm_path), hdd_path, 25, None)
        .map_err(|e| e.to_string())
}

/// Dumps and validates the Table of Contents (TOC) descriptors in PANA_DVD.FRM.
#[tauri::command]
pub fn dump_frm_toc_command(frm_path: String) -> Result<FrmTocDumpReport, String> {
    panadiga::crypto::dump_toc(Path::new(&frm_path), None).map_err(|e| e.to_string())
}

/// Computes the 64-bit ECB Known-Plaintext Attack (KPA) codebook dictionary.
#[tauri::command]
pub fn get_frm_codebook_command(
    frm_path: String,
    hdd_path: Option<String>,
) -> Result<CodebookReport, String> {
    let hdd_p = hdd_path.as_deref().map(Path::new);
    panadiga::crypto::print_codebook(Path::new(&frm_path), hdd_p, None)
        .map_err(|e| e.to_string())
}

/// Searches official Panasonic DMR-EH55 service manuals and schematics for technical terms.
#[tauri::command]
pub fn search_manuals_command(term: String) -> Result<Vec<ManualSearchResult>, String> {
    let candidate_dirs = [
        Path::new("manuals_and_schematics"),
        Path::new("../../manuals_and_schematics"),
        Path::new("../../../manuals_and_schematics"),
    ];
    let manuals_dir = candidate_dirs
        .iter()
        .find(|p| p.exists())
        .ok_or_else(|| "Manuals directory 'manuals_and_schematics' not found".to_string())?;

    panadiga::manuals::search_manuals(manuals_dir, Some(vec![term]), None)
        .map_err(|e| e.to_string())
}

/// Enumerates connected serial ports and provides RP2040 simulation options.
#[tauri::command]
pub fn list_pico_serial_ports() -> Result<Vec<String>, String> {
    crate::pico_serial::list_pico_serial_ports()
}

/// Connects to a Pico serial COM port and reads device telemetry and VDD voltage.
#[tauri::command]
pub fn pico_connect_and_test(
    port_name: String,
    baud_rate: u32,
) -> Result<PicoStatusResponse, String> {
    crate::pico_serial::pico_connect_and_test(port_name, baud_rate)
}

/// Triggers Cold Break-In sequence on the RP2040 DWire32A interface.
#[tauri::command]
pub fn pico_execute_cold_break_in(port_name: String, mode: u8) -> Result<String, String> {
    crate::pico_serial::pico_execute_cold_break_in(port_name, mode)
}

/// Executes in-circuit surgical sector erase of NVRAM micro-sectors on Spansion S29JL064H.
#[tauri::command]
pub fn pico_execute_in_circuit_erase(port_name: String) -> Result<String, String> {
    crate::pico_serial::pico_execute_in_circuit_erase(port_name)
}

/// Switches the Pico firmware into GDB Remote Serial Protocol bridge mode.
#[tauri::command]
pub fn pico_switch_to_gdb_mode(port_name: String) -> Result<String, String> {
    crate::pico_serial::pico_switch_to_gdb_mode(port_name)
}

/// Starts the GDB TCP-to-Serial bridge server.
#[tauri::command]
pub fn start_gdb_bridge_command(
    port_name: String,
    listen_addr: String,
) -> Result<GdbBridgeStatus, String> {
    crate::pico_serial::start_gdb_bridge(port_name, listen_addr)
}

/// Stops the GDB TCP-to-Serial bridge server.
#[tauri::command]
pub fn stop_gdb_bridge_command() -> Result<GdbBridgeStatus, String> {
    crate::pico_serial::stop_gdb_bridge()
}

/// Retrieves the current status of the GDB bridge server.
#[tauri::command]
pub fn get_gdb_bridge_status_command() -> Result<GdbBridgeStatus, String> {
    crate::pico_serial::get_gdb_bridge_status()
}

/// Retrieves the current AM34 CPU register file state.
#[tauri::command]
pub fn get_am34_registers_command() -> Result<Am34RegisterState, String> {
    crate::pico_serial::get_am34_registers()
}
