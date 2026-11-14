//! Bidirectional Live USB-CDC <-> P7501 SC0 Bridge Session Manager.
//!
//! Spawns a dedicated serial reader/writer worker while the Pico executes
//! `BRIDGE [baud] [NORMAL|JIG[:ms]|NOPULL]`, streaming raw bytes/ASCII/hex live to
//! the desktop UI (`pico://bridge-data`) and cleanly terminating via `'~~~'`.

use serde::Serialize;
use std::io::{Read, Write};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter};

use super::gdb_bridge::is_gdb_bridge_active;
use super::serial::lock_serial_io;
use super::types::{extract_port_name, reject_mock_port};

static SC0_BRIDGE_ACTIVE: AtomicBool = AtomicBool::new(false);
static BRIDGE_SESSION: Mutex<Option<ActiveSc0Bridge>> = Mutex::new(None);
static BRIDGE_STATUS: Mutex<Option<Sc0BridgeStatusDto>> = Mutex::new(None);

#[derive(Debug, Clone, Serialize)]
pub struct Sc0BridgeStatusDto {
    pub running: bool,
    pub port_name: String,
    pub baud_rate: u32,
    pub boot_mode: String,
    pub bytes_rx: u64,
    pub bytes_tx: u64,
    pub last_event: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Sc0BridgeDataEvent {
    pub stream: String,
    pub text: String,
    pub hex: String,
    pub bytes: Vec<u8>,
    pub timestamp: String,
}

enum BridgeWorkerMsg {
    Data(Vec<u8>),
    StopAndExit,
}

struct ActiveSc0Bridge {
    tx: Sender<BridgeWorkerMsg>,
    join: Option<JoinHandle<()>>,
}

pub fn is_sc0_bridge_active() -> bool {
    SC0_BRIDGE_ACTIVE.load(Ordering::SeqCst)
}

fn default_bridge_status() -> Sc0BridgeStatusDto {
    Sc0BridgeStatusDto {
        running: false,
        port_name: String::new(),
        baud_rate: 115_200,
        boot_mode: String::new(),
        bytes_rx: 0,
        bytes_tx: 0,
        last_event: "Ponte SC0 inativa".to_string(),
    }
}

fn format_timestamp_hms() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let h = (secs / 3600) % 24;
    let m = (secs / 60) % 60;
    let s = secs % 60;
    format!("{h:02}:{m:02}:{s:02}")
}

fn bytes_to_hex_spaced(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| format!("{b:02X}"))
        .collect::<Vec<_>>()
        .join(" ")
}

fn set_bridge_status(status: Sc0BridgeStatusDto) {
    let mut guard = BRIDGE_STATUS.lock().unwrap_or_else(|p| p.into_inner());
    *guard = Some(status);
}

fn update_bridge_status<F: FnOnce(&mut Sc0BridgeStatusDto)>(f: F) -> Sc0BridgeStatusDto {
    let mut guard = BRIDGE_STATUS.lock().unwrap_or_else(|p| p.into_inner());
    let entry = guard.get_or_insert_with(default_bridge_status);
    f(entry);
    entry.clone()
}

pub fn get_sc0_bridge_status_dto() -> Sc0BridgeStatusDto {
    let mut guard = BRIDGE_STATUS.lock().unwrap_or_else(|p| p.into_inner());
    let mut dto = guard.get_or_insert_with(default_bridge_status).clone();
    dto.running = is_sc0_bridge_active();
    dto
}

/// Pure encoder converting UI Bridge inputs (`text`, `line`, `hex`, `rsp`) into raw wire bytes.
pub fn encode_bridge_input(input_mode: &str, payload: &str) -> Result<Vec<u8>, String> {
    let mode = input_mode.trim().to_ascii_lowercase();
    match mode.as_str() {
        "text" | "raw" => Ok(payload.as_bytes().to_vec()),
        "line" | "cmd" => {
            let trimmed = payload.trim_end_matches(['\r', '\n']);
            let mut out = trimmed.as_bytes().to_vec();
            out.push(b'\r');
            Ok(out)
        }
        "hex" => decode_hex_payload(payload),
        "rsp" => {
            let trimmed = payload.trim();
            if trimmed.is_empty() {
                return Err("Payload RSP vazio.".to_string());
            }
            let csum: u8 = trimmed.bytes().fold(0u8, |acc, b| acc.wrapping_add(b));
            Ok(format!("${}#{:02x}", trimmed, csum).into_bytes())
        }
        other => Err(format!(
            "Modo de entrada SC0 Bridge desconhecido: '{}'",
            other
        )),
    }
}

fn decode_hex_payload(raw_hex: &str) -> Result<Vec<u8>, String> {
    let cleaned: String = raw_hex
        .split_whitespace()
        .map(|tok| tok.trim_start_matches("0x").trim_start_matches("0X"))
        .collect::<Vec<_>>()
        .join("")
        .chars()
        .filter(|c| !matches!(c, ':' | '-' | ',' | '_'))
        .collect();

    if cleaned.is_empty() || cleaned.len() % 2 != 0 {
        return Err(format!(
            "Hexadecimal inválido (comprimento ímpar ou vazio): '{}'",
            raw_hex
        ));
    }
    let mut bytes = Vec::with_capacity(cleaned.len() / 2);
    for idx in (0..cleaned.len()).step_by(2) {
        let byte = u8::from_str_radix(&cleaned[idx..idx + 2], 16)
            .map_err(|_| format!("Dígito hexadecimal inválido em '{}'", raw_hex))?;
        bytes.push(byte);
    }
    Ok(bytes)
}

fn emit_bridge_event(app: &AppHandle, stream: &str, bytes: &[u8], text_override: Option<&str>) {
    let text = text_override
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| String::from_utf8_lossy(bytes).into_owned());
    let payload = Sc0BridgeDataEvent {
        stream: stream.to_string(),
        text,
        hex: bytes_to_hex_spaced(bytes),
        bytes: bytes.to_vec(),
        timestamp: format_timestamp_hms(),
    };
    let _ = app.emit("pico://bridge-data", payload);
}

fn run_bridge_worker(
    app: AppHandle,
    mut port: Box<dyn serialport::SerialPort>,
    rx_chan: Receiver<BridgeWorkerMsg>,
) {
    let mut buf = [0u8; 512];
    let mut stop_requested = false;

    loop {
        while let Ok(msg) = rx_chan.try_recv() {
            match msg {
                BridgeWorkerMsg::Data(bytes) => {
                    if let Err(err) = port.write_all(&bytes) {
                        update_bridge_status(|st| {
                            st.last_event = format!("Erro TX Bridge: {}", err);
                        });
                        stop_requested = true;
                        break;
                    }
                    let _ = port.flush();
                    let sent_len = bytes.len() as u64;
                    update_bridge_status(|st| {
                        st.bytes_tx += sent_len;
                        st.last_event = format!("TX {} bytes para SC0", sent_len);
                    });
                    emit_bridge_event(&app, "tx", &bytes, None);
                    if bytes == b"~~~" {
                        stop_requested = true;
                    }
                }
                BridgeWorkerMsg::StopAndExit => {
                    let _ = port.write_all(b"~~~");
                    let _ = port.flush();
                    emit_bridge_event(&app, "tx", b"~~~", Some("~~~ (exit BRIDGE)"));
                    stop_requested = true;
                }
            }
        }

        match port.read(&mut buf) {
            Ok(n) if n > 0 => {
                let slice = &buf[..n];
                let text = String::from_utf8_lossy(slice);
                update_bridge_status(|st| {
                    st.bytes_rx += n as u64;
                });
                emit_bridge_event(&app, "rx", slice, None);
                if text.contains("BRIDGE: OK") || text.contains("BRIDGE: ERROR") {
                    break;
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {
                if stop_requested {
                    break;
                }
            }
            _ => break,
        }
    }

    drop(port);
    SC0_BRIDGE_ACTIVE.store(false, Ordering::SeqCst);
    update_bridge_status(|st| {
        st.running = false;
        st.last_event = "Ponte SC0 encerrada cleanly (~~~)".to_string();
    });
    emit_bridge_event(
        &app,
        "sys",
        b"",
        Some("Ponte SC0 finalizada e porta serial liberada."),
    );
}

pub fn start_sc0_bridge_session(
    app: AppHandle,
    port_name: String,
    baud_rate: Option<u32>,
    boot_mode: Option<String>,
) -> Result<Sc0BridgeStatusDto, String> {
    let clean_port = extract_port_name(&port_name);
    reject_mock_port(&clean_port)?;
    if is_gdb_bridge_active() {
        return Err("Porta ocupada pela Ponte GDB RSP ativa.".to_string());
    }
    if is_sc0_bridge_active() {
        let _ = stop_sc0_bridge_session();
    }

    let baud = baud_rate.unwrap_or(115_200).clamp(1200, 460_800);
    let clean_boot = boot_mode.unwrap_or_default().trim().to_string();
    let cmd = if clean_boot.is_empty() || clean_boot.eq_ignore_ascii_case("PASSIVE") {
        format!("BRIDGE {}\r\n", baud)
    } else {
        format!("BRIDGE {} {}\r\n", baud, clean_boot)
    };

    let _io_guard = lock_serial_io();
    let mut port =
        panadiga_core::pico::cdc::open_cdc(&clean_port, 115_200, Duration::from_millis(25))
            .map_err(|e| format!("Falha ao abrir porta {} para SC0 Bridge: {}", clean_port, e))?;

    port.write_all(cmd.as_bytes())
        .map_err(|e| format!("Falha ao iniciar BRIDGE na porta {}: {}", clean_port, e))?;
    let _ = port.flush();

    let (tx, rx) = mpsc::channel();
    SC0_BRIDGE_ACTIVE.store(true, Ordering::SeqCst);

    let initial_status = Sc0BridgeStatusDto {
        running: true,
        port_name: clean_port.clone(),
        baud_rate: baud,
        boot_mode: if clean_boot.is_empty() {
            "PASSIVE".to_string()
        } else {
            clean_boot
        },
        bytes_rx: 0,
        bytes_tx: 0,
        last_event: format!(
            "BRIDGE {} 8N1 ativa em {} (~~~ para sair)",
            baud, clean_port
        ),
    };
    set_bridge_status(initial_status.clone());
    emit_bridge_event(&app, "sys", b"", Some(&initial_status.last_event));

    let handle = thread::spawn(move || run_bridge_worker(app, port, rx));
    let mut session_guard = BRIDGE_SESSION.lock().unwrap_or_else(|p| p.into_inner());
    *session_guard = Some(ActiveSc0Bridge {
        tx,
        join: Some(handle),
    });

    Ok(initial_status)
}

pub fn send_sc0_bridge_input(input_mode: String, payload: String) -> Result<usize, String> {
    if !is_sc0_bridge_active() {
        return Err(
            "Nenhuma sessão SC0 Bridge ativa. Inicie a ponte antes de transmitir.".to_string(),
        );
    }
    let bytes = encode_bridge_input(&input_mode, &payload)?;
    let byte_len = bytes.len();
    let tx_opt = {
        let guard = BRIDGE_SESSION.lock().unwrap_or_else(|p| p.into_inner());
        guard.as_ref().map(|s| s.tx.clone())
    };
    let tx = tx_opt.ok_or_else(|| "Canal da sessão SC0 Bridge indisponível.".to_string())?;
    tx.send(BridgeWorkerMsg::Data(bytes))
        .map_err(|_| "Worker SC0 Bridge já encerrado.".to_string())?;
    Ok(byte_len)
}

pub fn stop_sc0_bridge_session() -> Result<Sc0BridgeStatusDto, String> {
    let active = {
        let mut guard = BRIDGE_SESSION.lock().unwrap_or_else(|p| p.into_inner());
        guard.take()
    };
    if let Some(mut session) = active {
        let _ = session.tx.send(BridgeWorkerMsg::StopAndExit);
        if let Some(handle) = session.join.take() {
            let _ = handle.join();
        }
    }
    SC0_BRIDGE_ACTIVE.store(false, Ordering::SeqCst);
    Ok(get_sc0_bridge_status_dto())
}

#[cfg(test)]
mod tests {
    use super::encode_bridge_input;

    #[test]
    fn encodes_line_hex_and_rsp_frames_cleanly() {
        assert_eq!(encode_bridge_input("line", "PASS").unwrap(), b"PASS\r");
        assert_eq!(
            encode_bridge_input("hex", "0x15 0E:04:02").unwrap(),
            vec![0x15, 0x0E, 0x04, 0x02]
        );
        assert_eq!(encode_bridge_input("rsp", "g").unwrap(), b"$g#67");
        assert_eq!(encode_bridge_input("rsp", "Z0").unwrap(), b"$Z0#8a");
        assert_eq!(encode_bridge_input("text", "~~~").unwrap(), b"~~~");
        assert!(encode_bridge_input("hex", "ABC").is_err());
    }
}
