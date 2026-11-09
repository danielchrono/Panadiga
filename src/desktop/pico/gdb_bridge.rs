//! Panasonic DMR-EH55 DIGA Suite - GDB Remote Serial Protocol (RSP) Bridge Controller
//!
//! Manages the background TCP-to-Serial proxy thread, mock RSP packet simulation,
//! client connection lifecycle, and AM34 shadow register file state.

use std::collections::VecDeque;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;
use std::time::Duration;

use super::types::{
    extract_port_name, Am34RegisterState, GdbBridgeStatus, GdbPacketTelemetryEvent,
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

/// Pure helper to verify GDB RSP packet checksum and classify the command.
pub fn verify_and_classify_rsp_packet(packet: &str) -> (String, bool) {
    let raw = packet.trim();
    if raw == "+" || raw == "-" {
        return ("AckNak".to_string(), true);
    }

    let trimmed = raw.trim_start_matches('+').trim();
    if !trimmed.starts_with('$') {
        return ("RawBytes".to_string(), false);
    }

    let without_dollar = &trimmed[1..];
    let parts: Vec<&str> = without_dollar.split('#').collect();
    if parts.len() != 2 {
        return ("Malformed".to_string(), false);
    }

    let payload = parts[0];
    let expected_csum_str = parts[1];
    let expected_csum = u8::from_str_radix(expected_csum_str, 16).unwrap_or(0);

    let actual_csum: u8 = payload.bytes().fold(0u8, |acc, b| acc.wrapping_add(b));
    let checksum_valid = actual_csum == expected_csum;

    let cmd_type = if payload.starts_with("qSupported") {
        "qSupported"
    } else if payload.starts_with('?') {
        "HaltReason"
    } else if payload == "g" {
        "ReadRegs"
    } else if payload.starts_with('G') {
        "WriteRegs"
    } else if payload.starts_with('m') {
        "ReadMem"
    } else if payload.starts_with('M') || payload.starts_with('X') {
        "WriteMem"
    } else if payload.starts_with('c') {
        "Continue"
    } else if payload.starts_with('s') {
        "Step"
    } else if payload.starts_with('k') || payload.starts_with('D') {
        "Detach/Kill"
    } else {
        "RspPayload"
    };

    (cmd_type.to_string(), checksum_valid)
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

/// Checks if the GDB TCP-to-Serial bridge currently owns the serial port.
pub fn is_gdb_bridge_active() -> bool {
    get_controller().running.load(Ordering::Relaxed)
}

/// Pure helper to frame a GDB RSP payload with `+$<payload>#<checksum>`.
pub fn build_rsp_packet(payload: &str) -> Vec<u8> {
    let csum: u8 = payload.bytes().fold(0u8, |acc, b| acc.wrapping_add(b));
    format!("+${}#{:02x}", payload, csum).into_bytes()
}

/// Encodes the AM34 register file into 48 little-endian bytes (96 hex chars) for GDB `g` reply.
fn encode_am34_registers_hex(regs: &Am34RegisterState) -> String {
    let mut out = String::with_capacity(96);
    for &d in &regs.d {
        for b in d.to_le_bytes() {
            out.push_str(&format!("{:02x}", b));
        }
    }
    for &a in &regs.a {
        for b in a.to_le_bytes() {
            out.push_str(&format!("{:02x}", b));
        }
    }
    for &val in &[regs.mdr, regs.psw, regs.pc, regs.sp] {
        for b in val.to_le_bytes() {
            out.push_str(&format!("{:02x}", b));
        }
    }
    out
}

/// Parses a 96-char hex string from a GDB `g` reply or `G` command and updates the shadow registers.
fn try_sync_registers_from_hex(hex_str: &str) {
    let clean = hex_str
        .trim()
        .trim_start_matches('+')
        .trim_start_matches('$')
        .split('#')
        .next()
        .unwrap_or("")
        .trim_start_matches('G');
    if clean.len() < 96 || !clean.chars().all(|c| c.is_ascii_hexdigit()) {
        return;
    }
    let parse_u32_le = |slice: &str| -> u32 {
        let mut bytes = [0u8; 4];
        for i in 0..4 {
            bytes[i] = u8::from_str_radix(&slice[i * 2..i * 2 + 2], 16).unwrap_or(0);
        }
        u32::from_le_bytes(bytes)
    };
    let ctrl = get_controller();
    if let Ok(mut regs) = ctrl.registers.lock() {
        for i in 0..4 {
            regs.d[i] = parse_u32_le(&clean[i * 8..(i + 1) * 8]);
        }
        for i in 0..4 {
            regs.a[i] = parse_u32_le(&clean[32 + i * 8..32 + (i + 1) * 8]);
        }
        regs.mdr = parse_u32_le(&clean[64..72]);
        regs.psw = parse_u32_le(&clean[72..80]);
        regs.pc = parse_u32_le(&clean[80..88]);
        regs.sp = parse_u32_le(&clean[88..96]);
    }
}

/// Pure responder for mock/simulated GDB RSP commands with AM34 memory map and IPL protection.
pub fn compute_mock_rsp_response(cmd: &str) -> Vec<u8> {
    if cmd.starts_with("qSupported") {
        build_rsp_packet("PacketSize=800;qXfer:memory-map:read+")
    } else if cmd.starts_with("qXfer:memory-map:read::") {
        let xml = concat!(
            "l<memory-map>",
            "<memory type=\"ram\" start=\"0x00000000\" length=\"0x00040000\"/>",
            "<memory type=\"flash\" start=\"0x80000000\" length=\"0x007f0000\">",
            "<property name=\"blocksize\">0x10000</property></memory>",
            "<memory type=\"flash\" start=\"0x807f0000\" length=\"0x00010000\">",
            "<property name=\"blocksize\">0x2000</property></memory>",
            "</memory-map>"
        );
        build_rsp_packet(xml)
    } else if cmd.starts_with('?') {
        build_rsp_packet("S05")
    } else if cmd == "g" {
        let regs = get_controller().registers.lock().unwrap().clone();
        build_rsp_packet(&encode_am34_registers_hex(&regs))
    } else if cmd.starts_with('G') {
        try_sync_registers_from_hex(cmd);
        build_rsp_packet("OK")
    } else if let Some(rest) = cmd.strip_prefix('m') {
        let mut parts = rest.split(',');
        let addr = u32::from_str_radix(parts.next().unwrap_or("0"), 16).unwrap_or(0);
        let len = usize::from_str_radix(parts.next().unwrap_or("4"), 16)
            .unwrap_or(4)
            .min(512);
        let mut hex_out = String::with_capacity(len * 2);
        for i in 0..len {
            let cur = addr.wrapping_add(i as u32);
            let byte_val: u8 = if (0x807F_0000..=0x807F_FFFF).contains(&cur) {
                0xFF // Virgin / erased NVRAM sector state
            } else if (0x8000_0000..=0x8000_000F).contains(&cur) {
                crate::constants::PANA_FRM_GLOBAL_HEADER[(cur - 0x8000_0000) as usize]
            } else if (0x8000_0000..=0x807E_FFFF).contains(&cur) {
                ((cur & 0xFF) as u8) ^ 0xA5
            } else {
                0x00
            };
            hex_out.push_str(&format!("{:02x}", byte_val));
        }
        build_rsp_packet(&hex_out)
    } else if cmd.starts_with('M') || cmd.starts_with('X') {
        let rest = &cmd[1..];
        let addr_str = rest.split(',').next().unwrap_or("0");
        let addr = u32::from_str_radix(addr_str, 16).unwrap_or(0);
        // Hardware safety invariant: IPL/OS Flash region (0x80000000..0x807EFFFF) is strictly Read-Only
        if (0x8000_0000..=0x807E_FFFF).contains(&addr) {
            build_rsp_packet("E05")
        } else {
            build_rsp_packet("OK")
        }
    } else if cmd.starts_with('c') || cmd.starts_with('s') {
        build_rsp_packet("S05")
    } else if cmd.starts_with('k') || cmd.starts_with('D') {
        build_rsp_packet("OK")
    } else {
        build_rsp_packet("")
    }
}

/// Starts the GDB TCP-to-Serial bridge server in a background thread.
pub fn start_gdb_bridge(port_name: String, listen_addr: String) -> Result<GdbBridgeStatus, String> {
    let ctrl = get_controller();
    if ctrl.running.load(Ordering::Relaxed) {
        return get_gdb_bridge_status();
    }

    let clean_port = extract_port_name(&port_name);
    let addr = if listen_addr.trim().is_empty() {
        "127.0.0.1:3333".to_string()
    } else {
        listen_addr.trim().to_string()
    };

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
                        // Drain initial ASCII banner ("[GDB RSP] Entering... Auto-ready...")
                        // so GDB client only receives clean RSP packets.
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
                let resp = compute_mock_rsp_response(cmd);

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

    while running_flag.load(Ordering::Relaxed) && connected_flag.load(Ordering::Relaxed) {
        // Read TCP -> Write Serial
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
                last_inbound_was_g = cmd == "g";
                if cmd.starts_with('G') {
                    try_sync_registers_from_hex(cmd);
                }
                let _ = serial_port.write_all(&t_buf[..n]);
            }
            Err(ref e)
                if e.kind() == std::io::ErrorKind::WouldBlock
                    || e.kind() == std::io::ErrorKind::TimedOut => {}
            Err(_) => break,
        }

        // Read Serial -> Write TCP
        match serial_port.read(&mut s_buf) {
            Ok(0) => {}
            Ok(n) => {
                let packet = String::from_utf8_lossy(&s_buf[..n]);
                record_telemetry("OUTBOUND", &packet);
                if last_inbound_was_g {
                    try_sync_registers_from_hex(&packet);
                    last_inbound_was_g = false;
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

    // Wake up listener if blocking
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
    fn test_verify_and_classify_rsp_packet() {
        let (cmd, valid) = verify_and_classify_rsp_packet("$g#67");
        assert_eq!(cmd, "ReadRegs");
        assert!(valid);

        let (cmd_halt, valid_halt) = verify_and_classify_rsp_packet("$?#3f");
        assert_eq!(cmd_halt, "HaltReason");
        assert!(valid_halt);

        let (cmd_invalid, valid_invalid) = verify_and_classify_rsp_packet("$g#00");
        assert_eq!(cmd_invalid, "ReadRegs");
        assert!(!valid_invalid);

        let (cmd_ack, valid_ack) = verify_and_classify_rsp_packet("+");
        assert_eq!(cmd_ack, "AckNak");
        assert!(valid_ack);
    }

    #[test]
    fn test_telemetry_ring_capacity() {
        for i in 0..150 {
            record_telemetry("INBOUND", &format!("$m{:x},4#00", i));
        }
        let telemetry = get_gdb_telemetry();
        assert_eq!(telemetry.len(), 128);
    }

    #[test]
    fn test_mock_rsp_memory_map_and_ipl_protection() {
        let map_resp =
            String::from_utf8(compute_mock_rsp_response("qXfer:memory-map:read::0,1ff")).unwrap();
        assert!(map_resp.starts_with("+$l<memory-map>"));
        assert!(map_resp.contains("start=\"0x80000000\""));
        let (_, valid) = verify_and_classify_rsp_packet(&map_resp);
        assert!(valid);

        let mem_resp = String::from_utf8(compute_mock_rsp_response("m807f0000,4")).unwrap();
        assert!(mem_resp.starts_with("+$ffffffff#"));
        let (_, valid_mem) = verify_and_classify_rsp_packet(&mem_resp);
        assert!(valid_mem);

        // IPL Write protection must reject writes to 0x80000000..0x807EFFFF with E05
        let ipl_write =
            String::from_utf8(compute_mock_rsp_response("M80000000,4:deadbeef")).unwrap();
        assert!(ipl_write.starts_with("+$E05#"));

        // SRAM write at 0x00004000 must succeed with OK
        let sram_write =
            String::from_utf8(compute_mock_rsp_response("M00004000,4:deadbeef")).unwrap();
        assert!(sram_write.starts_with("+$OK#"));
    }
}
