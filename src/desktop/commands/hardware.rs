//! Panasonic DMR-EH55 DIGA Suite - Pico Hardware, GDB Bridge, External Tools & RAG Commands
//!
//! Single Responsibility: Tauri IPC command dispatchers for RP2040/RP2350 USB CDC
//! communication, Cold Break-In sequencing, In-Circuit NVRAM erasure, GDB RSP
//! bridge lifecycle, external tooling invocation, and Zero-Error RAG queries.

use std::process::{Command, Stdio};

use super::dto::{
    Am34RegisterState, DigaModelProfileDto, FirstContactBackupResult, GdbBridgeStatus,
    GdbPacketTelemetryEvent, InCircuitNvramEraseResult, PicoDeviceStatusDto, PicoStatusResponse,
    get_diga_model_catalog_dto,
};

/// Returns the internal Panasonic DIGA Model Database (2MB/4MB/8MB/16MB NOR Flash profiles).
#[tauri::command]
pub fn get_diga_model_catalog() -> Result<Vec<DigaModelProfileDto>, String> {
    Ok(get_diga_model_catalog_dto())
}

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
    let port = port_name.unwrap_or_else(|| "".to_string());
    super::super::pico::pico_get_device_status_dto(port, None)
}

/// Frontend `pico_set_breakin_mode` command.
#[tauri::command]
pub fn pico_set_breakin_mode(
    port_name: Option<String>,
    mode: u8,
) -> Result<PicoDeviceStatusDto, String> {
    let port = port_name.unwrap_or_else(|| "".to_string());
    let _ = super::super::pico::pico_execute_cold_break_in(port.clone(), mode)?;
    super::super::pico::pico_get_device_status_dto(port, Some(mode))
}

/// Frontend `pico_inject_ram_loader` command (Handshake + Backup + INJECT_LOADER CONFIRM + ERASE_NVRAM CONFIRM).
#[tauri::command]
pub fn pico_inject_ram_loader(
    port_name: Option<String>,
) -> Result<InCircuitNvramEraseResult, String> {
    let port = port_name.unwrap_or_else(|| "".to_string());
    super::super::pico::pico_execute_in_circuit_erase_full(port)
}

/// Non-destructive 1st-Contact Read-Only Backup & Auto-Model Probe ("What's Next" protocol).
#[tauri::command]
pub fn pico_first_contact_backup(
    port_name: Option<String>,
) -> Result<FirstContactBackupResult, String> {
    let port = port_name.unwrap_or_else(|| "".to_string());
    super::super::pico::pico_execute_first_contact_backup(port)
}

/// Frontend `pico_send_serial_command` command for interactive USB CDC terminal.
#[tauri::command]
pub fn pico_send_serial_command(
    port_name: Option<String>,
    command: String,
) -> Result<String, String> {
    let port = port_name.unwrap_or_else(|| "".to_string());
    super::super::pico::pico_send_raw_serial_command(port, command)
}

/// Discovers any currently mounted Raspberry Pi BOOTSEL drives (RPI-RP2 / RP2350).
#[tauri::command]
pub fn pico_detect_bootsel_drives() -> Result<Vec<String>, String> {
    let drives = crate::pico::flasher::detect_bootsel_drives();
    Ok(drives
        .into_iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect())
}

/// Triggers 1200-baud touch reset on a running Pico to put it into BOOTSEL mode.
#[tauri::command]
pub fn pico_trigger_bootsel_reboot(port_name: String) -> Result<String, String> {
    crate::pico::flasher::trigger_1200_baud_bootsel(&port_name).map_err(|e| e.to_string())?;
    Ok(format!(
        "Port {} triggered 1200-baud reset into BOOTSEL mode",
        port_name
    ))
}

/// Flashes the embedded DWire32A firmware (RP2040 or RP2350) onto a detected BOOTSEL drive.
#[tauri::command]
pub fn pico_flash_firmware(
    target_family: Option<String>,
) -> Result<crate::pico::flasher::PicoFlashReport, String> {
    crate::pico::flasher::flash_embedded_firmware(target_family.as_deref())
        .map_err(|e| e.to_string())
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

/// Resolves the Panadiga workspace root (the directory that contains `tools/`),
/// independent of the process working directory (e.g. when launched from `dist/`).
fn resolve_workspace_root() -> std::path::PathBuf {
    let tools_dir = super::dto::resolve_candidate(&["tools"], "tools");
    let tools_abs = std::path::absolute(&tools_dir).unwrap_or_else(|_| tools_dir.into());
    tools_abs
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| std::path::PathBuf::from("."))
}

/// Spawns a fully detached child process.
///
/// Windows GUI processes have no valid standard handles; inheriting them makes
/// `CreateProcess` fail with `os error 50` ("Não há suporte para o pedido"),
/// so all three stdio streams are explicitly redirected to NUL.
fn spawn_detached(
    program: &str,
    args: &[&str],
    cwd: &std::path::Path,
) -> Result<(), std::io::Error> {
    let mut cmd = Command::new(program);
    cmd.args(args)
        .current_dir(cwd)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    cmd.spawn().map(|_| ())
}

/// Opens a `.cmd` helper from `tools/` in its own console window.
fn launch_tools_script(root: &std::path::Path, script: &str) -> Result<(), std::io::Error> {
    let script_path = root.join("tools").join(script);
    if !script_path.is_file() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("script não encontrado: {}", script_path.display()),
        ));
    }
    let script_str = script_path.to_string_lossy().to_string();
    spawn_detached("cmd", &["/c", "start", "", &script_str], root)
}

/// Launches external engineering tools (GDB Multiarch, OpenOCD, Ghidra scripts, HxD, PowerShell).
#[tauri::command]
pub fn launch_external_tool_command(tool: String) -> Result<String, String> {
    let root = resolve_workspace_root();
    match tool.to_lowercase().as_str() {
        "gdb" => {
            launch_tools_script(&root, "conectar_gdb_multiarch.cmd")
                .map_err(|e| format!("Falha ao iniciar GDB Multiarch: {}", e))?;
            Ok("Sessão interativa do GDB iniciada em novo terminal.".to_string())
        }
        "openocd" => {
            launch_tools_script(&root, "executar_openocd.cmd")
                .map_err(|e| format!("Falha ao iniciar OpenOCD: {}", e))?;
            Ok("Servidor OpenOCD iniciado em novo terminal.".to_string())
        }
        "ghidra_scripts" | "ghidra" => {
            launch_tools_script(&root, "iniciar_ghidra.cmd")
                .map_err(|e| format!("Falha ao iniciar Ghidra: {}", e))?;
            Ok("Inicializador do Ghidra disparado.".to_string())
        }
        "hxd" => {
            launch_tools_script(&root, "Abrir_HxD.cmd")
                .map_err(|e| format!("Falha ao abrir HxD: {}", e))?;
            Ok("HxD Hex Editor iniciado com o dump de memória.".to_string())
        }
        "terminal" => {
            let location = format!("Set-Location -LiteralPath '{}'", root.display());
            spawn_detached(
                "cmd",
                &[
                    "/c",
                    "start",
                    "",
                    "powershell",
                    "-NoExit",
                    "-Command",
                    &location,
                ],
                &root,
            )
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workspace_root_contains_tools_dir() {
        assert!(resolve_workspace_root().join("tools").is_dir());
    }

    #[test]
    fn unknown_external_tool_is_rejected() {
        let err = launch_external_tool_command("nope".to_string()).unwrap_err();
        assert!(err.contains("não suportada"));
    }
}
