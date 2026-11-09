//! Panasonic DMR-EH55 DIGA Suite - Pico Hardware, GDB Bridge, External Tools & RAG Commands
//!
//! Single Responsibility: Tauri IPC command dispatchers for RP2040/RP2350 USB CDC
//! communication, Cold Break-In sequencing, In-Circuit NVRAM erasure, GDB RSP
//! bridge lifecycle, external tooling invocation, and Zero-Error RAG queries.

use std::process::Command;

use super::dto::{
    Am34RegisterState, FirstContactBackupResult, GdbBridgeStatus, GdbPacketTelemetryEvent,
    InCircuitNvramEraseResult, PicoDeviceStatusDto, PicoStatusResponse,
};

/// Enumerates connected serial ports and provides RP2040 simulation options.
#[tauri::command]
pub fn list_pico_serial_ports() -> Result<Vec<String>, String> {
    super::super::pico::list_pico_serial_ports()
}

/// Connects to a Pico serial COM port and reads device telemetry and VDD voltage.
#[tauri::command]
pub fn pico_connect_and_test(
    port_name: String,
    baud_rate: u32,
) -> Result<PicoStatusResponse, String> {
    super::super::pico::pico_connect_and_test(port_name, baud_rate)
}

/// Triggers Cold Break-In sequence on the RP2040 DWire32A interface.
#[tauri::command]
pub fn pico_execute_cold_break_in(port_name: String, mode: u8) -> Result<String, String> {
    super::super::pico::pico_execute_cold_break_in(port_name, mode)
}

/// Executes in-circuit surgical sector erase of NVRAM micro-sectors on Spansion S29JL064H.
#[tauri::command]
pub fn pico_execute_in_circuit_erase(port_name: String) -> Result<String, String> {
    super::super::pico::pico_execute_in_circuit_erase(port_name)
}

/// Switches the Pico firmware into GDB Remote Serial Protocol bridge mode.
#[tauri::command]
pub fn pico_switch_to_gdb_mode(port_name: String) -> Result<String, String> {
    super::super::pico::pico_switch_to_gdb_mode(port_name)
}

/// Frontend `pico_get_status` command.
#[tauri::command]
pub fn pico_get_status(port_name: Option<String>) -> Result<PicoDeviceStatusDto, String> {
    let port = port_name.unwrap_or_else(|| "MOCK_RP2040".to_string());
    super::super::pico::pico_get_device_status_dto(port, None)
}

/// Frontend `pico_set_breakin_mode` command.
#[tauri::command]
pub fn pico_set_breakin_mode(
    port_name: Option<String>,
    mode: u8,
) -> Result<PicoDeviceStatusDto, String> {
    let port = port_name.unwrap_or_else(|| "MOCK_RP2040".to_string());
    let _ = super::super::pico::pico_execute_cold_break_in(port.clone(), mode)?;
    super::super::pico::pico_get_device_status_dto(port, Some(mode))
}

/// Frontend `pico_inject_ram_loader` command (Handshake + Backup + INJECT_LOADER CONFIRM + ERASE_NVRAM CONFIRM).
#[tauri::command]
pub fn pico_inject_ram_loader(
    port_name: Option<String>,
) -> Result<InCircuitNvramEraseResult, String> {
    let port = port_name.unwrap_or_else(|| "MOCK_RP2040".to_string());
    super::super::pico::pico_execute_in_circuit_erase_full(port)
}

/// Non-destructive 1st-Contact Read-Only Backup & Auto-Model Probe ("What's Next" protocol).
#[tauri::command]
pub fn pico_first_contact_backup(
    port_name: Option<String>,
) -> Result<FirstContactBackupResult, String> {
    let port = port_name.unwrap_or_else(|| "MOCK_RP2040".to_string());
    super::super::pico::pico_execute_first_contact_backup(port)
}

/// Frontend `pico_send_serial_command` command for interactive USB CDC terminal.
#[tauri::command]
pub fn pico_send_serial_command(
    port_name: Option<String>,
    command: String,
) -> Result<String, String> {
    let port = port_name.unwrap_or_else(|| "MOCK_RP2040".to_string());
    super::super::pico::pico_send_raw_serial_command(port, command)
}

/// Starts the GDB TCP-to-Serial bridge server.
#[tauri::command]
pub fn start_gdb_bridge_command(
    port_name: String,
    listen_addr: String,
) -> Result<GdbBridgeStatus, String> {
    super::super::pico::start_gdb_bridge(port_name, listen_addr)
}

/// Stops the GDB TCP-to-Serial bridge server.
#[tauri::command]
pub fn stop_gdb_bridge_command() -> Result<GdbBridgeStatus, String> {
    super::super::pico::stop_gdb_bridge()
}

/// Retrieves the current status of the GDB bridge server.
#[tauri::command]
pub fn get_gdb_bridge_status_command() -> Result<GdbBridgeStatus, String> {
    super::super::pico::get_gdb_bridge_status()
}

/// Retrieves the current AM34 CPU register file state.
#[tauri::command]
pub fn get_am34_registers_command() -> Result<Am34RegisterState, String> {
    super::super::pico::get_am34_registers()
}

/// Retrieves recorded GDB RSP packet telemetry events from the introspective ring buffer.
#[tauri::command]
pub fn get_gdb_telemetry_command() -> Result<Vec<GdbPacketTelemetryEvent>, String> {
    Ok(super::super::pico::get_gdb_telemetry())
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
