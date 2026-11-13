//! Panasonic DMR-EH55 DIGA Suite - SSOT, Granular Diagnostics & Reactive Streaming Commands
//!
//! Single Responsibility: Exposes the Rust hardware/firmware SSOT DTO (`get_hardware_ssot_spec`),
//! async non-blocking Pico diagnostic execution (`pico_execute_diagnostic`) with real-time
//! Tauri event emission (`pico://serial-log`, `pico://diagnostic-result`, `pico://telemetry`),
//! and a single managed telemetry stream toggle (`set_pico_telemetry_stream`).

use serde::Serialize;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tauri::Emitter;

use super::dto::{HardwareSsotSpecDto, LiveNvramCaptureEventDto, build_hardware_ssot_spec};
use super::hardware::pico_get_status;
use crate::pico::diagnostics::{PicoDiagnosticResultDto, execute_diagnostic_on_port};

/// Real-time serial log entry pushed over Tauri event `"pico://serial-log"`.
#[derive(Debug, Clone, Serialize)]
pub struct ReactiveSerialEventDto {
    pub id: String,
    pub timestamp: String,
    pub source: String,
    pub level: String,
    pub message: String,
}

struct TelemetryStreamState {
    active: Arc<AtomicBool>,
    worker_started: Arc<AtomicBool>,
    port_name: Mutex<String>,
}

static STREAM_STATE: OnceLock<TelemetryStreamState> = OnceLock::new();

fn stream_state() -> &'static TelemetryStreamState {
    STREAM_STATE.get_or_init(|| TelemetryStreamState {
        active: Arc::new(AtomicBool::new(false)),
        worker_started: Arc::new(AtomicBool::new(false)),
        port_name: Mutex::new(String::new()),
    })
}

fn current_time_hhmmss() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let hh = (secs / 3600) % 24;
    let mm = (secs / 60) % 60;
    let ss = secs % 60;
    format!("{:02}:{:02}:{:02}", hh, mm, ss)
}

fn build_and_emit_live_nvram_capture(
    app: &tauri::AppHandle,
    source: &str,
    port_name: &str,
    base_bus_address: u32,
    bytes: &[u8],
) -> Result<LiveNvramCaptureEventDto, String> {
    let label = format!("{} ({})", source, port_name.trim());
    let report =
        panadiga_core::nvram::inspect_partial_nvram_capture(bytes, base_bus_address, &label)
            .map_err(|e| e.to_string())?;
    let event = LiveNvramCaptureEventDto {
        source: source.to_string(),
        port: port_name.trim().to_string(),
        base_bus_address: format!("0x{:08X}", base_bus_address),
        captured_bytes: bytes.len(),
        report,
    };
    let _ = app.emit("pico://nvram-live-capture", &event);
    Ok(event)
}

/// Captures real live NVRAM bytes in-circuit over Pico P7501 `DUMP` / GDB RSP in 8 KB microsector chunks,
/// audits sector activity (`SA134..SA141`) via `panadiga-core`, and emits `"pico://nvram-live-capture"`.
#[tauri::command]
pub async fn pico_capture_nvram_live(
    app: tauri::AppHandle,
    port_name: String,
    base_addr: Option<u32>,
    word_count: Option<u32>,
) -> Result<LiveNvramCaptureEventDto, String> {
    let start_addr = base_addr.unwrap_or(panadiga_core::constants::AM34_NVRAM_BASE);
    let total_words = word_count.unwrap_or(16_384).clamp(16, 32_768);
    let chunk_words_limit: u32 = 2_048; // 8 KB (1 Spansion microsector) per USB CDC DUMP transaction
    let mut accumulated: Vec<u8> = Vec::with_capacity((total_words as usize) * 4);
    let mut words_read: u32 = 0;

    while words_read < total_words {
        let cur_addr = start_addr.wrapping_add(words_read * 4);
        let chunk_words = (total_words - words_read).min(chunk_words_limit);
        let arg = format!("0x{:08X} {}", cur_addr, chunk_words);
        let port_clone = port_name.clone();
        let res = tauri::async_runtime::spawn_blocking(move || {
            execute_diagnostic_on_port(&port_clone, "DUMP", Some(&arg))
        })
        .await
        .map_err(|e| format!("Falha interna de execução assíncrona: {}", e))??;

        if !res.success {
            return Err(format!(
                "Falha ao capturar bloco NVRAM em 0x{:08X}: {}",
                cur_addr, res.summary
            ));
        }
        let mut chunk_bytes =
            panadiga_core::pico::cdc::parse_pico_dump_hex_to_bytes(&res.raw_output);
        if chunk_bytes.is_empty() {
            return Err(format!(
                "DUMP retornou 0 bytes legíveis em 0x{:08X} ({}).",
                cur_addr, res.summary
            ));
        }
        accumulated.append(&mut chunk_bytes);
        words_read += chunk_words;
    }

    build_and_emit_live_nvram_capture(
        &app,
        "pico-live-nvram-capture",
        &port_name,
        start_addr,
        &accumulated,
    )
}

/// Returns the complete Hardware & Firmware Single Source of Truth (SSOT) specification.
#[tauri::command]
pub fn get_hardware_ssot_spec() -> Result<HardwareSsotSpecDto, String> {
    Ok(build_hardware_ssot_spec())
}

/// Executes any granular Pico firmware command asynchronously without stalling the WebView UI,
/// pushing TX/RX logs, structured diagnostic output, and updated telemetry over Tauri events.
#[tauri::command]
pub async fn pico_execute_diagnostic(
    app: tauri::AppHandle,
    port_name: String,
    command_id: String,
    arg: Option<String>,
) -> Result<PicoDiagnosticResultDto, String> {
    let cmd_preview = match arg.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        Some(a) => format!("{} {}", command_id.trim().to_ascii_uppercase(), a),
        None => command_id.trim().to_ascii_uppercase(),
    };

    let _ = app.emit(
        "pico://serial-log",
        ReactiveSerialEventDto {
            id: format!(
                "tx-{}",
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis()
            ),
            timestamp: current_time_hhmmss(),
            source: "HOST".to_string(),
            level: "tx".to_string(),
            message: format!("> {}", cmd_preview),
        },
    );

    let port_clone = port_name.clone();
    let cmd_clone = command_id.clone();
    let arg_clone = arg.clone();

    let result = tauri::async_runtime::spawn_blocking(move || {
        execute_diagnostic_on_port(&port_clone, &cmd_clone, arg_clone.as_deref())
    })
    .await
    .map_err(|e| format!("Falha interna ao aguardar diagnóstico Pico: {}", e))??;

    let _ = app.emit("pico://diagnostic-result", &result);
    if result.command_id == "DUMP" && result.success {
        let bytes = panadiga_core::pico::cdc::parse_pico_dump_hex_to_bytes(&result.raw_output);
        if !bytes.is_empty() {
            let base_addr = result
                .dump_rows
                .first()
                .and_then(|row| {
                    u32::from_str_radix(
                        row.address
                            .trim_start_matches("0x")
                            .trim_start_matches("0X"),
                        16,
                    )
                    .ok()
                })
                .unwrap_or(panadiga_core::constants::AM34_NVRAM_BASE);
            let _ = build_and_emit_live_nvram_capture(
                &app,
                "pico-diagnostic-dump",
                &port_name,
                base_addr,
                &bytes,
            );
        }
    }
    let _ = app.emit(
        "pico://serial-log",
        ReactiveSerialEventDto {
            id: format!(
                "rx-{}",
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis()
            ),
            timestamp: current_time_hhmmss(),
            source: "PICO".to_string(),
            level: if result.success { "rx" } else { "error" }.to_string(),
            message: result.summary.clone(),
        },
    );

    if matches!(
        result.command_id.as_str(),
        "STATUS"
            | "VDD"
            | "PINCHECK"
            | "HANDSHAKE"
            | "RESET"
            | "AUTODETECT"
            | "FLASH_SIZE"
            | "ISOLATE"
            | "ENGAGE"
    ) && let Ok(mut status) = pico_get_status(Some(port_name))
    {
        if let Some(diag_vdd) = result.vdd_volts
            && status.vdd_volts <= 0.01
            && diag_vdd > 0.01
        {
            status.vdd_volts = diag_vdd;
            status.target_powered = diag_vdd >= 2.5 && diag_vdd < 3.45;
            if status.target_powered && status.core_state == "RESET" {
                status.core_state = "RUNNING".to_string();
            }
        }
        let _ = app.emit("pico://telemetry", &status);
    }

    Ok(result)
}

/// Enables or pauses the single background Pico telemetry push stream (`pico://telemetry`).
#[tauri::command]
pub fn set_pico_telemetry_stream(
    app: tauri::AppHandle,
    active: bool,
    port_name: Option<String>,
) -> Result<bool, String> {
    let state = stream_state();
    if let Some(port) = port_name.filter(|p| !p.trim().is_empty())
        && let Ok(mut guard) = state.port_name.lock()
    {
        *guard = port;
    }
    state.active.store(active, Ordering::Relaxed);

    if active
        && state
            .worker_started
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
    {
        let app_clone = app.clone();
        thread::spawn(move || {
            let st = stream_state();
            loop {
                thread::sleep(Duration::from_millis(2000));
                if !st.active.load(Ordering::Relaxed) {
                    continue;
                }
                let port = st.port_name.lock().map(|p| p.clone()).unwrap_or_default();
                if port.trim().is_empty() {
                    continue;
                }
                if port.starts_with("BLE") || port.starts_with("ANDROID-USB-OTG") {
                    continue;
                }
                if let Some(Ok(status)) = crate::pico::try_pico_get_device_status_dto(port) {
                    let _ = app_clone.emit("pico://telemetry", &status);
                }
            }
        });
    }

    Ok(active)
}
