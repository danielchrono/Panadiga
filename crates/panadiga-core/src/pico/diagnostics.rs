use std::thread;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};

/// Helper to send a command and read the response until keyword or timeout
fn send_and_read(
    port: &mut dyn serialport::SerialPort,
    cmd: &str,
    keyword: &str,
    timeout: Duration,
) -> String {
    let _ = port.write_all(format!("{}\r\n", cmd).as_bytes());
    let _ = port.flush();
    let start = Instant::now();
    let mut out = String::new();
    let mut chunk = [0u8; 512];
    while start.elapsed() < timeout {
        match port.read(&mut chunk) {
            Ok(n) if n > 0 => {
                out.push_str(&String::from_utf8_lossy(&chunk[..n]));
                if out.contains(keyword) || out.contains("PANADIGA>") || out.contains("P7501>") {
                    break;
                }
            }
            _ => thread::sleep(Duration::from_millis(40)),
        }
    }
    out.trim().to_string()
}

/// Probes a serial port to test connection and query target 3.3V VDD rail telemetry
pub fn ping_pico(port_name: &str, baud_rate: u32) -> Result<String> {
    let mut port = crate::pico::cdc::open_cdc(port_name, baud_rate, Duration::from_millis(2000))
        .with_context(|| format!("Failed to open serial port '{}'", port_name))?;

    // Discard any residual buffer
    let _ = port.clear(serialport::ClearBuffer::Input);

    // Query PING
    let banner = send_and_read(&mut *port, "PING", "PONG", Duration::from_millis(1500));

    // Query VDD voltage
    let vdd_resp = send_and_read(&mut *port, "STATUS", "VDD:", Duration::from_millis(1500));

    Ok(format!(
        "--- Host Controller Telemetry Report ---\n\
         Port: {}\n\
         Baud: {}\n\
         Banner:\n{}\n\
         Telemetry:\n{}\n",
        port_name, baud_rate, banner, vdd_resp
    ))
}
