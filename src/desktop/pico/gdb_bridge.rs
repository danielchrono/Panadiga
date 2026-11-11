//! Panasonic DMR-EH55 DIGA Suite - GDB Remote Serial Protocol (RSP) Bridge Controller
//!
//! Imperative shell managing the background TCP-to-Serial proxy thread,
//! client connection lifecycle, and AM34 shadow register file state.
//! Pure RSP framing and validation reside in `rsp_protocol.rs`.

use std::collections::VecDeque;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;
use std::time::Duration;

use super::rsp_protocol::{
    compute_mock_rsp_response_pure, decode_am34_registers_hex, verify_and_classify_rsp_packet,
};
use super::types::{
    Am34RegisterState, GdbBridgeStatus, GdbPacketTelemetryEvent, extract_port_name,
};

struct GdbBridgeController {
    running: Arc<AtomicBool>,
    client_connected: Arc<AtomicBool>,
    client_count: Arc<AtomicU32>,
    port_name: Mutex<String>,
    listen_addr: Mutex<String>,
    last_event: Mutex<String>,
    thread_handle: Mutex<Option<thread::JoinHandle<()>>>,
    registers: Mutex<Am34RegisterState>,
    telemetry_ring: Mutex<VecDeque<GdbPacketTelemetryEvent>>,
}

static GDB_CONTROLLER: OnceLock<GdbBridgeController> = OnceLock::new();

fn get_controller() -> &'static GdbBridgeController {
    GDB_CONTROLLER.get_or_init(|| GdbBridgeController {
        running: Arc::new(AtomicBool::new(false)),
        client_connected: Arc::new(AtomicBool::new(false)),
        client_count: Arc::new(AtomicU32::new(0)),
        port_name: Mutex::new(String::new()),
        listen_addr: Mutex::new("127.0.0.1:3333".to_string()),
        last_event: Mutex::new("Servidor GDB offline".to_string()),
        thread_handle: Mutex::new(None),
        registers: Mutex::new(Am34RegisterState {
            d: [0x00000000, 0x00000000, 0x00000000, 0x00000000],
            a: [0x00000000, 0x00000000, 0x00000000, 0x0003FFF0],
            mdr: 0x00000000,
            psw: 0x00000100,
            pc: 0x80000000,
            sp: 0x0003FFF0,
            is_halted: true,
        }),
        telemetry_ring: Mutex::new(VecDeque::with_capacity(128)),
    })
}

/// Checks if the GDB TCP-to-Serial bridge currently owns the serial port.
pub fn is_gdb_bridge_active() -> bool {
    get_controller().running.load(Ordering::Relaxed)
}

/// Synchronizes the controller's AM34 shadow register file from a 96-char hex string.
fn sync_controller_registers(hex_str: &str) {
    let ctrl = get_controller();
    let halted = ctrl.registers.lock().map(|r| r.is_halted).unwrap_or(true);
    if let Some(updated) = decode_am34_registers_hex(hex_str, halted) {
        if let Ok(mut regs) = ctrl.registers.lock() {
            *regs = updated;
        }
    }
}

/// Records a packet event into the circular telemetry ring buffer.
pub fn record_telemetry(direction: &str, raw_packet: &str) {
    let ctrl = get_controller();
    let (command_type, checksum_valid) = verify_and_classify_rsp_packet(raw_packet);
    let event = GdbPacketTelemetryEvent {
        timestamp_ms: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64,
        direction: direction.to_string(),
        command_type,
        raw_packet: raw_packet.to_string(),
        checksum_valid,
    };

    let mut ring = ctrl.telemetry_ring.lock().unwrap();
    if ring.len() >= 128 {
        ring.pop_front();
    }
    ring.push_back(event);
}

/// Retrieves all recorded GDB packet telemetry events.
pub fn get_gdb_telemetry() -> Vec<GdbPacketTelemetryEvent> {
    let ctrl = get_controller();
    ctrl.telemetry_ring
        .lock()
        .unwrap()
        .iter()
        .cloned()
        .collect()
}

/// Starts the GDB TCP-to-Serial bridge server in a background thread.
pub fn start_gdb_bridge(port_name: String, listen_addr: String) -> Result<GdbBridgeStatus, String> {
    let ctrl = get_controller();
    if ctrl.running.load(Ordering::Relaxed) {
        return get_gdb_bridge_status();
    }

    let clean_port = extract_port_name(&port_name);
    if clean_port.is_empty() {
        return Err("Nenhuma porta serial selecionada para a Ponte GDB RSP. Conecte o Pico ou selecione uma porta COM primeiro.".to_string());
    }

    let addr = if listen_addr.trim().is_empty() {
        "127.0.0.1:3333".to_string()
    } else {
        listen_addr.trim().to_string()
    };

    // Pre-validate that TCP port can be bound
    let pre_test_listener = TcpListener::bind(&addr).map_err(|e| {
        format!("Falha ao vincular porta TCP {}: {}. Verifique se outra instância do GDB ou processo não está utilizando a porta.", addr, e)
    })?;
    drop(pre_test_listener);

    // If hardware port, pre-validate serial accessibility
    if !clean_port.starts_with("MOCK") {
        match serialport::new(&clean_port, 115200)
            .timeout(Duration::from_millis(150))
            .open()
        {
            Ok(_) => {}
            Err(e) => {
                return Err(format!(
                    "Falha ao abrir porta serial {} para a Ponte GDB RSP: {}. Verifique se o Pico está conectado e não ocupado por outro terminal.",
                    clean_port, e
                ));
            }
        }
    }

    ctrl.running.store(true, Ordering::Relaxed);
    ctrl.client_connected.store(false, Ordering::Relaxed);
    ctrl.client_count.store(0, Ordering::Relaxed);

    *ctrl.port_name.lock().unwrap() = clean_port.clone();
    *ctrl.listen_addr.lock().unwrap() = addr.clone();
    *ctrl.last_event.lock().unwrap() = format!("Ouvindo conexões GDB em {}", addr);

    let running_flag = ctrl.running.clone();
    let connected_flag = ctrl.client_connected.clone();
    let count_flag = ctrl.client_count.clone();
    let thread_port = clean_port.clone();
    let thread_addr = addr.clone();

    let handle = thread::spawn(move || {
        let listener = match TcpListener::bind(&thread_addr) {
            Ok(l) => {
                let _ = l.set_nonblocking(true);
                l
            }
            Err(e) => {
                crate::log_warn!(
                    "[GDB Bridge] Falha ao vincular porta TCP {}: {}",
                    thread_addr,
                    e
                );
                running_flag.store(false, Ordering::Relaxed);
                return;
            }
        };

        let mut maybe_serial: Option<Box<dyn serialport::SerialPort>> =
            if !thread_port.starts_with("MOCK") {
                match serialport::new(&thread_port, 115200)
                    .timeout(Duration::from_millis(120))
                    .open()
                {
                    Ok(mut p) => {
                        let _ = p.write_all(b"\r\nGDB\r\n");
                        let _ = p.flush();
                        let mut drain_buf = [0u8; 512];
                        let mut accumulated = String::new();
                        for _ in 0..8 {
                            match p.read(&mut drain_buf) {
                                Ok(n) if n > 0 => {
                                    accumulated.push_str(&String::from_utf8_lossy(&drain_buf[..n]));
                                    if accumulated.contains("Auto-ready") {
                                        break;
                                    }
                                }
                                _ => break,
                            }
                        }
                        let _ = p.set_timeout(Duration::from_millis(50));
                        Some(p)
                    }
                    Err(e) => {
                        crate::log_warn!(
                            "[GDB Bridge] Falha ao abrir porta serial {}: {}",
                            thread_port,
                            e
                        );
                        running_flag.store(false, Ordering::Relaxed);
                        return;
                    }
                }
            } else {
                None
            };

        while running_flag.load(Ordering::Relaxed) {
            match listener.accept() {
                Ok((tcp_stream, _peer)) => {
                    connected_flag.store(true, Ordering::Relaxed);
                    count_flag.fetch_add(1, Ordering::Relaxed);
                    if thread_port.starts_with("MOCK") {
                        handle_mock_session(tcp_stream, &running_flag, &connected_flag);
                    } else if let Some(ref mut serial_port) = maybe_serial {
                        handle_serial_session(
                            tcp_stream,
                            serial_port,
                            &running_flag,
                            &connected_flag,
                        );
                    }
                    connected_flag.store(false, Ordering::Relaxed);
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(50));
                }
                Err(_) => break,
            }
        }

        // Send GDB Detach packet (+$D#44) before closing serial port so Pico exits binary RSP mode
        if let Some(ref mut serial_port) = maybe_serial {
            let _ = serial_port.write_all(b"+$D#44\r\n");
            let _ = serial_port.flush();
            let mut detach_drain = [0u8; 64];
            let _ = serial_port.read(&mut detach_drain);
        }
    });

    *ctrl.thread_handle.lock().unwrap() = Some(handle);
    get_gdb_bridge_status()
}

fn handle_mock_session(
    mut tcp_stream: TcpStream,
    running_flag: &Arc<AtomicBool>,
    connected_flag: &Arc<AtomicBool>,
) {
    let _ = tcp_stream.set_nonblocking(false);
    let _ = tcp_stream.set_read_timeout(Some(Duration::from_millis(200)));

    let mut buf = [0u8; 1024];
    while running_flag.load(Ordering::Relaxed) && connected_flag.load(Ordering::Relaxed) {
        match tcp_stream.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                let packet = String::from_utf8_lossy(&buf[..n]);
                record_telemetry("INBOUND", &packet);

                let cmd = packet
                    .split('$')
                    .nth(1)
                    .and_then(|s| s.split('#').next())
                    .unwrap_or(&packet)
                    .trim_start_matches('+');
                if cmd.starts_with('G') {
                    sync_controller_registers(cmd);
                }
                let regs = get_controller().registers.lock().unwrap().clone();
                let resp = compute_mock_rsp_response_pure(cmd, &regs);

                record_telemetry("OUTBOUND", &String::from_utf8_lossy(&resp));
                let _ = tcp_stream.write_all(&resp);
                if cmd.starts_with('k') || cmd.starts_with('D') {
                    break;
                }
            }
            Err(ref e)
                if e.kind() == std::io::ErrorKind::WouldBlock
                    || e.kind() == std::io::ErrorKind::TimedOut =>
            {
                thread::sleep(Duration::from_millis(10));
            }
            Err(_) => break,
        }
    }
}

fn handle_serial_session(
    mut tcp_stream: TcpStream,
    serial_port: &mut Box<dyn serialport::SerialPort>,
    running_flag: &Arc<AtomicBool>,
    connected_flag: &Arc<AtomicBool>,
) {
    let _ = tcp_stream.set_nonblocking(false);
    let _ = tcp_stream.set_read_timeout(Some(Duration::from_millis(50)));

    let mut s_buf = [0u8; 1024];
    let mut t_buf = [0u8; 1024];
    let mut last_inbound_was_g = false;
    let mut g_accum_buf = String::with_capacity(128);

    while running_flag.load(Ordering::Relaxed) && connected_flag.load(Ordering::Relaxed) {
        match tcp_stream.read(&mut t_buf) {
            Ok(0) => break,
            Ok(n) => {
                let packet = String::from_utf8_lossy(&t_buf[..n]);
                record_telemetry("INBOUND", &packet);
                let cmd = packet
                    .split('$')
                    .nth(1)
                    .and_then(|s| s.split('#').next())
                    .unwrap_or("");
                if cmd == "g" {
                    last_inbound_was_g = true;
                    g_accum_buf.clear();
                }
                if cmd.starts_with('G') {
                    sync_controller_registers(cmd);
                }
                let _ = serial_port.write_all(&t_buf[..n]);
            }
            Err(ref e)
                if e.kind() == std::io::ErrorKind::WouldBlock
                    || e.kind() == std::io::ErrorKind::TimedOut => {}
            Err(_) => break,
        }

        match serial_port.read(&mut s_buf) {
            Ok(0) => {}
            Ok(n) => {
                let packet = String::from_utf8_lossy(&s_buf[..n]);
                record_telemetry("OUTBOUND", &packet);
                if last_inbound_was_g {
                    g_accum_buf.push_str(&packet);
                    if g_accum_buf.contains('#') {
                        sync_controller_registers(&g_accum_buf);
                        last_inbound_was_g = false;
                        g_accum_buf.clear();
                    }
                }
                if tcp_stream.write_all(&s_buf[..n]).is_err() {
                    break;
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {}
            Err(_) => break,
        }
    }
}

/// Stops the GDB TCP-to-Serial bridge server.
pub fn stop_gdb_bridge() -> Result<GdbBridgeStatus, String> {
    let ctrl = get_controller();
    if !ctrl.running.load(Ordering::Relaxed) {
        return get_gdb_bridge_status();
    }

    ctrl.running.store(false, Ordering::Relaxed);
    ctrl.client_connected.store(false, Ordering::Relaxed);

    let addr = ctrl.listen_addr.lock().unwrap().clone();
    let _ = TcpStream::connect_timeout(
        &addr.parse().unwrap_or("127.0.0.1:3333".parse().unwrap()),
        Duration::from_millis(100),
    );

    if let Some(h) = ctrl.thread_handle.lock().unwrap().take() {
        let _ = h.join();
    }

    *ctrl.last_event.lock().unwrap() = "Servidor GDB finalizado".to_string();
    get_gdb_bridge_status()
}

/// Retrieves the current status and metrics of the GDB bridge.
pub fn get_gdb_bridge_status() -> Result<GdbBridgeStatus, String> {
    let ctrl = get_controller();
    Ok(GdbBridgeStatus {
        running: ctrl.running.load(Ordering::Relaxed),
        port_name: ctrl.port_name.lock().unwrap().clone(),
        listen_addr: ctrl.listen_addr.lock().unwrap().clone(),
        connected_clients: ctrl.client_count.load(Ordering::Relaxed),
        last_event: ctrl.last_event.lock().unwrap().clone(),
    })
}

/// Retrieves the current AM34 register state.
pub fn get_am34_registers() -> Result<Am34RegisterState, String> {
    let ctrl = get_controller();
    Ok(ctrl.registers.lock().unwrap().clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_telemetry_ring_capacity() {
        for i in 0..150 {
            record_telemetry("INBOUND", &format!("$m{:x},4#00", i));
        }
        let telemetry = get_gdb_telemetry();
        assert_eq!(telemetry.len(), 128);
    }
}
