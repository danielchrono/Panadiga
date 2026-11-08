//! Panasonic DMR-EH55 DIGA Suite - GDB Remote Serial Protocol (RSP) Bridge Controller
//!
//! Manages the background TCP-to-Serial proxy thread, mock RSP packet simulation,
//! client connection lifecycle, and AM34 shadow register file state.

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;
use std::time::Duration;

use super::types::{extract_port_name, Am34RegisterState, GdbBridgeStatus};

struct GdbBridgeController {
    running: Arc<AtomicBool>,
    client_connected: Arc<AtomicBool>,
    client_count: Arc<AtomicU32>,
    port_name: Mutex<String>,
    listen_addr: Mutex<String>,
    last_event: Mutex<String>,
    thread_handle: Mutex<Option<thread::JoinHandle<()>>>,
    registers: Mutex<Am34RegisterState>,
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
    })
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
                    .timeout(Duration::from_millis(100))
                    .open()
                {
                    Ok(mut p) => {
                        let _ = p.write_all(b"\r\nGDB\r\n");
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
                let cmd = packet
                    .split('$')
                    .nth(1)
                    .and_then(|s| s.split('#').next())
                    .unwrap_or(&packet)
                    .trim_start_matches('+');
                if cmd.starts_with("qSupported") {
                    let _ = tcp_stream.write_all(b"+$PacketSize=800;qXfer:memory-map:read+#3f");
                } else if cmd.starts_with('?') {
                    let _ = tcp_stream.write_all(b"+$S05#b8");
                } else if cmd == "g" {
                    let mock_g = "+$00000000000000000000000000000000000000000000000000000000f0ff0300000000000001000000000080f0ff0300#44";
                    let _ = tcp_stream.write_all(mock_g.as_bytes());
                } else if cmd.starts_with('c') || cmd.starts_with('s') {
                    let _ = tcp_stream.write_all(b"+$S05#b8");
                } else if cmd.starts_with('k') || cmd.starts_with('D') {
                    let _ = tcp_stream.write_all(b"+$OK#9a");
                    break;
                } else {
                    let _ = tcp_stream.write_all(b"+$#00");
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

    while running_flag.load(Ordering::Relaxed) && connected_flag.load(Ordering::Relaxed) {
        // Read TCP -> Write Serial
        match tcp_stream.read(&mut t_buf) {
            Ok(0) => break,
            Ok(n) => {
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
