//! Tauri IPC Commands for Interactive Bidirectional SC0 Bridge (`BRIDGE`).

use crate::pico::{
    Sc0BridgeStatusDto, get_sc0_bridge_status_dto, send_sc0_bridge_input, start_sc0_bridge_session,
    stop_sc0_bridge_session,
};

#[tauri::command]
pub async fn pico_start_sc0_bridge(
    app: tauri::AppHandle,
    port_name: String,
    baud_rate: Option<u32>,
    boot_mode: Option<String>,
) -> Result<Sc0BridgeStatusDto, String> {
    tauri::async_runtime::spawn_blocking(move || {
        start_sc0_bridge_session(app, port_name, baud_rate, boot_mode)
    })
    .await
    .map_err(|e| format!("Falha ao iniciar SC0 Bridge: {}", e))?
}

#[tauri::command]
pub async fn pico_send_bridge_input(input_mode: String, payload: String) -> Result<usize, String> {
    tauri::async_runtime::spawn_blocking(move || send_sc0_bridge_input(input_mode, payload))
        .await
        .map_err(|e| format!("Falha ao transmitir na SC0 Bridge: {}", e))?
}

#[tauri::command]
pub async fn pico_stop_sc0_bridge() -> Result<Sc0BridgeStatusDto, String> {
    tauri::async_runtime::spawn_blocking(stop_sc0_bridge_session)
        .await
        .map_err(|e| format!("Falha ao encerrar SC0 Bridge: {}", e))?
}

#[tauri::command]
pub async fn pico_get_bridge_status() -> Result<Sc0BridgeStatusDto, String> {
    Ok(get_sc0_bridge_status_dto())
}
