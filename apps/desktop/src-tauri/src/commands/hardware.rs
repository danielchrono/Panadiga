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

/// Triggers P7501 discrete boot/reset sequence on the RP2040/RP2350 hardware probe.
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
    let drives = panadiga_core::pico::flasher::detect_bootsel_drives();
    Ok(drives
        .into_iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect())
}

/// Triggers 1200-baud touch reset on a running Pico to put it into BOOTSEL mode.
#[tauri::command]
pub fn pico_trigger_bootsel_reboot(port_name: String) -> Result<String, String> {
    panadiga_core::pico::flasher::trigger_1200_baud_bootsel(&port_name)
        .map_err(|e| e.to_string())?;
    Ok(format!(
        "Port {} triggered 1200-baud reset into BOOTSEL mode",
        port_name
    ))
}

/// Flashes the embedded P7501 Async UART firmware (RP2040 or RP2350) onto a detected BOOTSEL drive.
#[tauri::command]
pub fn pico_flash_firmware(
    target_family: Option<String>,
) -> Result<panadiga_core::pico::flasher::PicoFlashReport, String> {
    panadiga_core::pico::flasher::flash_embedded_firmware(target_family.as_deref())
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

/// Resolves the Panadiga workspace root via `panadiga-core` multiplatform discovery.
#[allow(dead_code)]
fn resolve_workspace_root() -> std::path::PathBuf {
    panadiga_core::constants::resolve_workspace_root()
}

/// Spawns a fully detached child process cleanly across Desktop OSes.
#[allow(dead_code)]
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

#[cfg(target_os = "windows")]
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

#[cfg(target_os = "linux")]
fn spawn_first_terminal(
    root: &std::path::Path,
    shell_cmd: Option<&str>,
) -> Result<(), std::io::Error> {
    for (term, exec_flag) in [
        ("x-terminal-emulator", "-e"),
        ("gnome-terminal", "--"),
        ("konsole", "-e"),
        ("xfce4-terminal", "-x"),
        ("xterm", "-e"),
    ] {
        let result = match shell_cmd {
            Some(c) => spawn_detached(term, &[exec_flag, "sh", "-lc", c], root),
            None => spawn_detached(term, &[], root),
        };
        if result.is_ok() {
            return Ok(());
        }
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "Nenhum emulador de terminal Linux encontrado no PATH.",
    ))
}

/// Launches external engineering tools across Windows, macOS, Linux, and returns clean guidance on Android.
#[tauri::command]
pub fn launch_external_tool_command(tool: String) -> Result<String, String> {
    #[cfg(target_os = "android")]
    {
        let _ = tool;
        Err(
            "Ferramentas externas de bancada (GDB CLI, Ghidra, Terminal) são recursos exclusivos do host Desktop; utilize os painéis nativos In-App no Android."
                .to_string(),
        )
    }

    #[cfg(target_os = "windows")]
    {
        let root = resolve_workspace_root();
        match tool.to_lowercase().as_str() {
            "gdb" => {
                launch_tools_script(&root, "conectar_gdb_multiarch.cmd")
                    .map_err(|e| format!("Falha ao iniciar GDB Multiarch: {}", e))?;
                Ok("Sessão interativa do GDB iniciada em novo terminal.".to_string())
            }
            "ghidra_scripts" | "ghidra" => {
                launch_tools_script(&root, "iniciar_ghidra.cmd")
                    .map_err(|e| format!("Falha ao iniciar Ghidra: {}", e))?;
                Ok("Inicializador do Ghidra disparado.".to_string())
            }
            "hxd" => {
                launch_tools_script(&root, "Abrir_HxD.cmd")
                    .map_err(|e| format!("Falha ao abrir editor hexadecimal: {}", e))?;
                Ok("Editor hexadecimal iniciado no workspace.".to_string())
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
                "Ferramenta externa '{}' não suportada. Opções: gdb, ghidra, hxd, terminal",
                other
            )),
        }
    }

    #[cfg(target_os = "macos")]
    {
        let root = resolve_workspace_root();
        let root_str = root.to_string_lossy().to_string();
        match tool.to_lowercase().as_str() {
            "gdb" => {
                spawn_detached(
                    "osascript",
                    &[
                        "-e",
                        &format!(
                            "tell application \"Terminal\" to do script \"cd '{}' && gdb-multiarch -ex 'target remote 127.0.0.1:3333'\"",
                            root_str
                        ),
                    ],
                    &root,
                )
                .map_err(|e| format!("Falha ao iniciar Terminal com GDB: {}", e))?;
                Ok("Terminal macOS iniciado conectando em 127.0.0.1:3333.".to_string())
            }
            "ghidra_scripts" | "ghidra" => {
                let ghidra_dir = root.join("tools").join("ghidra");
                spawn_detached("open", &[&ghidra_dir.to_string_lossy()], &root)
                    .map_err(|e| format!("Falha ao abrir scripts do Ghidra: {}", e))?;
                Ok("Diretório de scripts Ghidra aberto no Finder.".to_string())
            }
            "hxd" => {
                let images_dir = root.join("firmware_and_images");
                spawn_detached("open", &[&images_dir.to_string_lossy()], &root)
                    .map_err(|e| format!("Falha ao abrir pasta de imagens: {}", e))?;
                Ok("Diretório de imagens binárias aberto no Finder.".to_string())
            }
            "terminal" => {
                spawn_detached("open", &["-a", "Terminal", &root_str], &root)
                    .map_err(|e| format!("Falha ao abrir Terminal macOS: {}", e))?;
                Ok("Terminal macOS iniciado no workspace.".to_string())
            }
            other => Err(format!(
                "Ferramenta externa '{}' não suportada. Opções: gdb, ghidra, hxd, terminal",
                other
            )),
        }
    }

    #[cfg(all(
        not(target_os = "windows"),
        not(target_os = "macos"),
        not(target_os = "android")
    ))]
    {
        let root = resolve_workspace_root();
        match tool.to_lowercase().as_str() {
            "gdb" => {
                spawn_first_terminal(
                    &root,
                    Some("gdb-multiarch -ex 'target remote 127.0.0.1:3333'; exec bash"),
                )
                .map_err(|e| format!("Falha ao iniciar GDB no terminal Linux: {}", e))?;
                Ok("Sessão GDB Multiarch iniciada no terminal.".to_string())
            }
            "ghidra_scripts" | "ghidra" => {
                let ghidra_res = spawn_detached("ghidraRun", &[], &root).or_else(|_| {
                    spawn_detached(
                        "xdg-open",
                        &[&root.join("tools").join("ghidra").to_string_lossy()],
                        &root,
                    )
                });
                ghidra_res.map_err(|e| format!("Falha ao iniciar Ghidra no Linux: {}", e))?;
                Ok("Ghidra / diretório de scripts aberto.".to_string())
            }
            "hxd" => {
                spawn_detached(
                    "xdg-open",
                    &[&root.join("firmware_and_images").to_string_lossy()],
                    &root,
                )
                .map_err(|e| format!("Falha ao abrir diretório de imagens: {}", e))?;
                Ok("Pasta de dumps abertas via xdg-open (use o Inspetor Hex nativo).".to_string())
            }
            "terminal" => {
                spawn_first_terminal(&root, None)
                    .map_err(|e| format!("Falha ao abrir terminal Linux: {}", e))?;
                Ok("Terminal Linux iniciado no workspace.".to_string())
            }
            other => Err(format!(
                "Ferramenta externa '{}' não suportada. Opções: gdb, ghidra, hxd, terminal",
                other
            )),
        }
    }
}

/// Queries the deterministic 6-layer Zero-Error RAG Engine natively in-process.
#[tauri::command]
pub fn query_rag_engine(
    query: String,
    _top_k: Option<usize>,
) -> Result<panadiga_rag::models::RagQueryResultDto, String> {
    let answer = panadiga_rag::query(&query).to_query_result();
    Ok(answer)
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
