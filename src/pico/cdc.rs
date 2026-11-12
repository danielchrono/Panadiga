//! USB CDC ACM port opening helper for the Panadiga DWire32A controller.
//!
//! Single Responsibility: guarantees that every host-side serial session toward the
//! Pico asserts DTR/RTS right after `open()`. The Pico SDK `stdio_usb` driver only
//! transmits when the host raises DTR (`tud_cdc_connected()`); without it the firmware
//! silently drops every reply, making the host believe the device is mute (VDD = 0.00 V).

use std::time::Duration;

use serialport::{ClearBuffer, SerialPort};

/// Settling time allowing the firmware to notice the DTR edge and print its prompt.
const DTR_SETTLE: Duration = Duration::from_millis(120);

/// Opens `port_name` at `baud_rate`, asserts DTR + RTS and discards any stale input
/// (boot prompt, leftover replies from a previous session).
pub fn open_cdc(
    port_name: &str,
    baud_rate: u32,
    timeout: Duration,
) -> serialport::Result<Box<dyn SerialPort>> {
    let mut port = serialport::new(port_name, baud_rate)
        .timeout(timeout)
        .open()?;
    let _ = port.write_data_terminal_ready(true);
    let _ = port.write_request_to_send(true);
    std::thread::sleep(DTR_SETTLE);
    let _ = port.clear(ClearBuffer::Input);
    Ok(port)
}

/// Extracts the numeric voltage from a firmware reply such as `VDD: 3.24V (OK)`.
/// Returns `None` when the reply carries no `VDD:` field (e.g. the device was mute).
pub fn parse_vdd_reply(reply: &str) -> Option<f32> {
    let after = reply.split("VDD:").nth(1)?;
    let token = after.split_whitespace().next()?;
    token.trim_end_matches(['V', 'v']).parse::<f32>().ok()
}

#[cfg(test)]
mod tests {
    use super::parse_vdd_reply;

    #[test]
    fn parses_vdd_with_unit_and_status() {
        assert_eq!(
            parse_vdd_reply("DWIRE32A> STATUS\r\nVDD: 3.24V (OK)\r\n"),
            Some(3.24)
        );
    }

    #[test]
    fn parses_zero_vdd() {
        assert_eq!(parse_vdd_reply("VDD: 0.00V (LOW)"), Some(0.0));
    }

    #[test]
    fn mute_device_yields_none() {
        assert_eq!(parse_vdd_reply(""), None);
        assert_eq!(parse_vdd_reply("PONG"), None);
    }
}
