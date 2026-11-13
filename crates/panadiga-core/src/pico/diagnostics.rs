use std::time::Duration;

use anyhow::{Context, Result};

/// Probes a serial port to test connection and query target 3.3V VDD rail telemetry
pub fn ping_pico(port_name: &str, baud_rate: u32) -> Result<String> {
    let mut port = crate::pico::cdc::open_cdc(port_name, baud_rate, Duration::from_millis(2000))
        .with_context(|| format!("Failed to open serial port '{}'", port_name))?;

    // Query PING and VDD voltage through shared CDC prompt-safe helper
    let banner = crate::pico::cdc::send_and_read_until(
        &mut *port,
        "PING",
        &["PONG"],
        Duration::from_millis(1500),
    );
    let vdd_resp = crate::pico::cdc::send_and_read_until(
        &mut *port,
        "STATUS",
        &["VDD:"],
        Duration::from_millis(1500),
    );

    Ok(format!(
        "--- Host Controller Telemetry Report ---\n\
         Port: {}\n\
         Baud: {}\n\
         Banner:\n{}\n\
         Telemetry:\n{}\n",
        port_name, baud_rate, banner, vdd_resp
    ))
}
