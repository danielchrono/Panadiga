use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use anyhow::{Context, Result};

fn pump_stream<R: Read, W: Write>(
    mut reader: R,
    mut writer: W,
    flag: Arc<AtomicBool>,
    on_eof_msg: Option<&str>,
) {
    let mut buf = [0u8; 2048];
    while flag.load(Ordering::Relaxed) {
        match reader.read(&mut buf) {
            Ok(0) => {
                if let Some(msg) = on_eof_msg {
                    log_println!("{}", msg);
                }
                break;
            }
            Ok(n) => {
                if writer.write_all(&buf[..n]).is_err() || writer.flush().is_err() {
                    break;
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {}
            Err(_) => break,
        }
    }
    flag.store(false, Ordering::Relaxed);
}

/// Runs bidirectional TCP-to-Serial GDB Remote Serial Protocol (RSP) bridge
pub fn run_gdb_bridge(port_name: &str, baud_rate: u32, listen_addr: &str) -> Result<()> {
    log_println!("====================================================================");
    log_println!("   PANASONIC AM34 / DWIRE32A GDB REMOTE SERIAL PROTOCOL (RSP) BRIDGE");
    log_println!("   Host Architecture: Multi-Platform (RP2040 / RP2350 Dual-Core)");
    log_println!("   Serial Interface : {} @ {} baud", port_name, baud_rate);
    log_println!("   TCP GDB Listener : {}", listen_addr);
    log_println!("====================================================================");

    let mut serial_port =
        crate::pico::cdc::open_cdc(port_name, baud_rate, Duration::from_millis(100))
            .with_context(|| format!("Failed to open serial port '{}'", port_name))?;

    // Request firmware switch to GDB mode
    log_info!("[Bridge] Requesting controller firmware enter GDB RSP server mode...");
    let _ = serial_port.write_all(b"\r\nGDB\r\n");
    thread::sleep(Duration::from_millis(100));

    let listener = TcpListener::bind(listen_addr)
        .with_context(|| format!("Failed to bind TCP listener on '{}'", listen_addr))?;

    log_println!(
        "[Bridge] Listening for GDB debugger connections on {}...",
        listen_addr
    );
    log_println!("[Bridge] In your debugger terminal, run:");
    log_println!("         (gdb) target remote {}", listen_addr);
    log_println!("--------------------------------------------------------------------");

    for stream_res in listener.incoming() {
        match stream_res {
            Ok(mut tcp_stream) => {
                let _ = tcp_stream.set_read_timeout(Some(Duration::from_millis(100)));
                let peer_addr = tcp_stream
                    .peer_addr()
                    .map(|a| a.to_string())
                    .unwrap_or_else(|_| "unknown".to_string());
                log_info!("[Bridge] GDB client connected from: {}", peer_addr);

                let running = Arc::new(AtomicBool::new(true));

                // Thread 1: Serial -> TCP
                let serial_reader = serial_port
                    .try_clone()
                    .context("Failed to clone serial port handle")?;
                let tcp_writer = tcp_stream
                    .try_clone()
                    .context("Failed to clone TCP stream handle")?;
                let running_clone = running.clone();

                let s2t_handle = thread::spawn(move || {
                    pump_stream(serial_reader, tcp_writer, running_clone, None);
                });

                // Thread 2: TCP -> Serial
                pump_stream(
                    &mut tcp_stream,
                    &mut serial_port,
                    running,
                    Some("[Bridge] GDB client disconnected."),
                );

                let _ = s2t_handle.join();
                log_info!("[Bridge] Session finished. Waiting for next GDB connection...");
            }
            Err(e) => {
                log_warn!("[Bridge] TCP connection failed: {}", e);
            }
        }
    }

    Ok(())
}
