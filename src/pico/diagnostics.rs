use std::io::{Read, Write};
use std::thread;
use std::time::Duration;

use anyhow::{Context, Result};

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
        "--- Host Controller Telemetry Report ---\n\
         Port: {}\n\
         Baud: {}\n\
         Banner:\n{}\n\
         Telemetry:\n{}\n",
        port_name, baud_rate, banner, vdd_resp
    ))
}
