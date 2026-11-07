//! Raspberry Pi Pico (RP2040 / RP2350) Serial & GDB RSP Bridge Module
//!
//! Provides hardware discovery, diagnostic telemetry probing, and a TCP-to-Serial
//! GDB Remote Serial Protocol (RSP) bridge connecting high-level debuggers
//! (mn10300-elf-gdb, gdb-multiarch, Ghidra, IDA Pro) to the Panasonic DWire32A OCD bus.

use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serialport::SerialPortType;

/// Raspberry Pi USB Vendor ID
pub const RASPBERRY_PI_VID: u16 = 0x2E8A;
/// Standard Raspberry Pi Pico (RP2040) USB CDC Serial Product ID
pub const PICO_RP2040_CDC_PID: u16 = 0x000A;
/// Standard Raspberry Pi Pico 2 (RP2350) USB CDC Serial Product ID
pub const PICO_RP2350_CDC_PID: u16 = 0x000F;
/// Raspberry Pi BOOTSEL Mode Product ID
pub const PICO_BOOTSEL_PID: u16 = 0x0003;

/// Architecture classification of connected Pico controller
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PicoArchitecture {
    Rp2040,
    Rp2350,
    Bootloader,
    Unknown,
}

impl std::fmt::Display for PicoArchitecture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Rp2040 => write!(f, "Raspberry Pi Pico (RP2040)"),
            Self::Rp2350 => write!(f, "Raspberry Pi Pico 2 (RP2350)"),
            Self::Bootloader => write!(f, "Raspberry Pi BOOTSEL Mode"),
            Self::Unknown => write!(f, "Unknown USB Serial Device"),
        }
    }
}

/// Metadata and hardware attributes of a detected serial port
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PicoDeviceInfo {
    pub port_name: String,
    pub architecture: PicoArchitecture,
    pub vid: u16,
    pub pid: u16,
    pub description: String,
}

/// Discovers and enumerates all serial ports, classifying Raspberry Pi Pico devices
pub fn list_pico_devices() -> Result<Vec<PicoDeviceInfo>> {
    let mut devices = Vec::new();
    let ports = serialport::available_ports().context("Failed to list serial ports on system")?;

    for p in ports {
        if let SerialPortType::UsbPort(info) = p.port_type {
            let (arch, desc) = if info.vid == RASPBERRY_PI_VID {
                match info.pid {
                    PICO_RP2040_CDC_PID => (
                        PicoArchitecture::Rp2040,
                        "Raspberry Pi Pico (RP2040) DWire32A Host".to_string(),
                    ),
                    PICO_RP2350_CDC_PID => (
                        PicoArchitecture::Rp2350,
                        "Raspberry Pi Pico 2 (RP2350) DWire32A Host (520KB SRAM)".to_string(),
                    ),
                    PICO_BOOTSEL_PID => (
                        PicoArchitecture::Bootloader,
                        "Raspberry Pi BOOTSEL UF2 Mode".to_string(),
                    ),
                    other => (
                        PicoArchitecture::Unknown,
                        format!("Raspberry Pi Unknown PID 0x{:04X}", other),
                    ),
                }
            } else {
                (
                    PicoArchitecture::Unknown,
                    info.product.unwrap_or_else(|| "USB Serial Device".to_string()),
                )
            };

            devices.push(PicoDeviceInfo {
                port_name: p.port_name,
                architecture: arch,
                vid: info.vid,
                pid: info.pid,
                description: desc,
            });
        }
    }

    Ok(devices)
}

/// Probes a serial port to test connection and query target 3.3V VDD rail telemetry
pub fn ping_pico(port_name: &str, baud_rate: u32) -> Result<String> {
    let mut port = serialport::new(port_name, baud_rate)
        .timeout(Duration::from_millis(1500))
        .open()
        .with_context(|| format!("Failed to open serial port '{}'", port_name))?;

    // Send newline and PING
    port.write_all(b"\r\nPING\r\n")
        .context("Failed to write PING probe to serial port")?;
    thread::sleep(Duration::from_millis(100));

    let mut buf = [0u8; 1024];
    let n = port.read(&mut buf).unwrap_or(0);
    let banner = String::from_utf8_lossy(&buf[..n]).trim().to_string();

    // Query VDD voltage
    let _ = port.write_all(b"STATUS\r\n");
    thread::sleep(Duration::from_millis(100));
    let n_vdd = port.read(&mut buf).unwrap_or(0);
    let vdd_resp = String::from_utf8_lossy(&buf[..n_vdd]).trim().to_string();

    Ok(format!(
        "--- Pico DWire32A Telemetry Report ---\n\
         Port: {}\n\
         Baud: {}\n\
         Banner:\n{}\n\
         Telemetry:\n{}\n",
        port_name, baud_rate, banner, vdd_resp
    ))
}

/// Runs bidirectional TCP-to-Serial GDB Remote Serial Protocol (RSP) bridge
pub fn run_gdb_bridge(port_name: &str, baud_rate: u32, listen_addr: &str) -> Result<()> {
    println!("====================================================================");
    println!("   PANASONIC AM34 / DWIRE32A GDB REMOTE SERIAL PROTOCOL (RSP) BRIDGE");
    println!("   Serial Interface : {} @ {} baud", port_name, baud_rate);
    println!("   TCP GDB Listener : {}", listen_addr);
    println!("====================================================================");

    let mut serial_port = serialport::new(port_name, baud_rate)
        .timeout(Duration::from_millis(100))
        .open()
        .with_context(|| format!("Failed to open serial port '{}'", port_name))?;

    // Request firmware switch to GDB mode
    println!("[Bridge] Requesting Pico firmware enter GDB RSP server mode...");
    let _ = serial_port.write_all(b"\r\nGDB\r\n");
    thread::sleep(Duration::from_millis(100));

    let listener = TcpListener::bind(listen_addr)
        .with_context(|| format!("Failed to bind TCP listener on '{}'", listen_addr))?;

    println!("[Bridge] Listening for GDB debugger connections on {}...", listen_addr);
    println!("[Bridge] In your debugger terminal, run:");
    println!("         (gdb) target remote {}", listen_addr);
    println!("--------------------------------------------------------------------");

    for stream_res in listener.incoming() {
        match stream_res {
            Ok(mut tcp_stream) => {
                let peer_addr = tcp_stream.peer_addr().map(|a| a.to_string()).unwrap_or_else(|_| "unknown".to_string());
                println!("[Bridge] GDB client connected from: {}", peer_addr);

                let running = Arc::new(AtomicBool::new(true));

                // Thread 1: Serial -> TCP
                let mut serial_reader = serial_port.try_clone().context("Failed to clone serial port handle")?;
                let mut tcp_writer = tcp_stream.try_clone().context("Failed to clone TCP stream handle")?;
                let running_clone = running.clone();

                let s2t_handle = thread::spawn(move || {
                    let mut s_buf = [0u8; 2048];
                    while running_clone.load(Ordering::Relaxed) {
                        match serial_reader.read(&mut s_buf) {
                            Ok(0) => {}
                            Ok(n) => {
                                if tcp_writer.write_all(&s_buf[..n]).is_err() {
                                    break;
                                }
                            }
                            Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {}
                            Err(_) => break,
                        }
                    }
                    running_clone.store(false, Ordering::Relaxed);
                });

                // Thread 2: TCP -> Serial
                let mut t_buf = [0u8; 2048];
                while running.load(Ordering::Relaxed) {
                    match tcp_stream.read(&mut t_buf) {
                        Ok(0) => {
                            println!("[Bridge] GDB client disconnected.");
                            break;
                        }
                        Ok(n) => {
                            if serial_port.write_all(&t_buf[..n]).is_err() {
                                break;
                            }
                        }
                        Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {}
                        Err(_) => break,
                    }
                }

                running.store(false, Ordering::Relaxed);
                let _ = s2t_handle.join();
                println!("[Bridge] Session finished. Waiting for next GDB connection...");
            }
            Err(e) => {
                eprintln!("[Bridge] Warning: TCP connection failed: {}", e);
            }
        }
    }

    Ok(())
}
