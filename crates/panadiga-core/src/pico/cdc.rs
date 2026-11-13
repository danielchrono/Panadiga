//! USB CDC ACM port opening helper for the Panadiga P7501 Async UART probe controller.
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

/// Appends one 32-bit hexadecimal word string (`0xAABBCCDD` or `AABBCCDD`) as Little-Endian bytes.
pub fn push_hex_word_le(token: &str, out: &mut Vec<u8>) {
    let clean_tok = token
        .trim()
        .trim_start_matches("0x")
        .trim_start_matches("0X");
    if clean_tok.len() == 8
        && clean_tok.chars().all(|c| c.is_ascii_hexdigit())
        && let Ok(word) = u32::from_str_radix(clean_tok, 16)
    {
        out.extend_from_slice(&word.to_le_bytes());
    }
}

/// Parses an auto-detected Top-Boot 64 KB EEMUL NVRAM base address (`0x801F0000`, `0x803F0000`,
/// `0x807F0000`, `0x80FE0000`, or `0x80FF0000`) from Pico `HANDSHAKE` or `AUTODETECT` output.
pub fn parse_autodetected_nvram_base(handshake_output: &str) -> &'static str {
    if handshake_output.contains("0x801F0000") || handshake_output.contains("2 MB NOR Flash") {
        "0x801F0000"
    } else if handshake_output.contains("0x803F0000") || handshake_output.contains("4 MB NOR Flash")
    {
        "0x803F0000"
    } else if handshake_output.contains("0x80FE0000")
        || handshake_output.contains("0x80FF0000")
        || handshake_output.contains("16 MB NOR Flash")
    {
        if handshake_output.contains("0x80FE0000") {
            "0x80FE0000"
        } else {
            "0x80FF0000"
        }
    } else {
        "0x807F0000"
    }
}

/// Normalized parsed word row from a Pico `DUMP` reply.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedDumpLine {
    pub address: String,
    pub words: Vec<String>,
    pub raw_bytes: Vec<u8>,
}

/// Parses formatted Pico `DUMP` address lines (`807F0000: AABBCCDD 11223344 ...`) into normalized
/// uppercase addresses, 32-bit hex word strings, and Little-Endian byte vectors.
pub fn parse_pico_dump_lines(ascii_dump: &str) -> Vec<ParsedDumpLine> {
    let mut rows = Vec::new();
    for line in ascii_dump.lines() {
        let trimmed = line.trim();
        let Some((addr_part, vals_part)) = trimmed.split_once(':') else {
            continue;
        };
        let addr_clean = addr_part
            .trim()
            .trim_start_matches("0x")
            .trim_start_matches("0X");
        if addr_clean.len() != 8 || !addr_clean.chars().all(|c| c.is_ascii_hexdigit()) {
            continue;
        }
        let mut words = Vec::new();
        let mut raw_bytes = Vec::new();
        for token in vals_part.split_whitespace() {
            let clean_tok = token.trim_start_matches("0x").trim_start_matches("0X");
            if clean_tok.len() == 8 && clean_tok.chars().all(|c| c.is_ascii_hexdigit()) {
                words.push(format!("0x{}", clean_tok.to_ascii_uppercase()));
                push_hex_word_le(clean_tok, &mut raw_bytes);
            }
        }
        if !words.is_empty() {
            rows.push(ParsedDumpLine {
                address: format!("0x{}", addr_clean.to_ascii_uppercase()),
                words,
                raw_bytes,
            });
        }
    }
    rows
}

/// Converts Pico `DUMP` ASCII output (`807F0000: AABBCCDD 11223344 ...` or `[0x807F0000] = 0xAABBCCDD`)
/// into Little-Endian binary bytes (`u32::to_le_bytes`).
pub fn parse_pico_dump_hex_to_bytes(ascii_dump: &str) -> Vec<u8> {
    let mut binary = Vec::new();
    for row in parse_pico_dump_lines(ascii_dump) {
        binary.extend_from_slice(&row.raw_bytes);
    }
    if binary.is_empty() {
        for line in ascii_dump.lines() {
            if let Some((_, rhs)) = line.trim().split_once('=') {
                push_hex_word_le(rhs, &mut binary);
            }
        }
    }
    binary
}

/// Strips leading CLI prompt tokens (`PANADIGA>`, `P7501>`) and surrounding whitespace
/// left in-flight from prior commands so prompt-only bursts never truncate active replies.
pub fn strip_leading_prompts(mut raw: &str) -> &str {
    loop {
        let trimmed = raw.trim_start();
        if let Some(rest) = trimmed
            .strip_prefix("PANADIGA>")
            .or_else(|| trimmed.strip_prefix("P7501>"))
        {
            raw = rest;
        } else {
            return trimmed;
        }
    }
}

/// Checks whether a buffered serial reply contains a matched stop token or a prompt
/// preceded by real command payload (never a bare stale leading prompt).
pub fn reply_has_completed(raw: &str, stop_tokens: &[&str]) -> bool {
    let body = strip_leading_prompts(raw);
    if body.is_empty() {
        return false;
    }
    if stop_tokens.iter().any(|tok| {
        if matches!(*tok, "PANADIGA>" | "P7501>") {
            body.contains(tok)
        } else {
            body.find(tok).is_some_and(|idx| body[idx..].contains('\n'))
        }
    }) {
        return true;
    }
    body.rsplit_once("PANADIGA>")
        .or_else(|| body.rsplit_once("P7501>"))
        .is_some_and(|(prefix, _)| !prefix.trim().is_empty())
}

/// Sends a single command line over CDC and collects the reply until any stop token or timeout.
pub fn send_and_read_until(
    port: &mut dyn SerialPort,
    command: &str,
    stop_tokens: &[&str],
    max_wait: Duration,
) -> String {
    let _ = port.clear(ClearBuffer::Input);
    if port
        .write_all(format!("{}\r\n", command).as_bytes())
        .is_err()
    {
        return String::new();
    }
    let _ = port.flush();
    let _ = port.set_timeout(Duration::from_millis(120));

    let deadline = std::time::Instant::now() + max_wait;
    let mut out = String::new();
    let mut buf = [0u8; 1024];
    while std::time::Instant::now() < deadline {
        if let Ok(n) = port.read(&mut buf)
            && n > 0
        {
            out.push_str(&String::from_utf8_lossy(&buf[..n]));
            if reply_has_completed(&out, stop_tokens) {
                break;
            }
        }
    }
    let clean = strip_leading_prompts(&out).trim();
    if clean.is_empty() {
        out.trim().to_string()
    } else {
        clean.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_vdd_reply, reply_has_completed, strip_leading_prompts};

    #[test]
    fn parses_vdd_with_unit_and_status() {
        assert_eq!(
            parse_vdd_reply("PANADIGA> STATUS\r\nVDD: 3.24V (OK)\r\n"),
            Some(3.24)
        );
        assert_eq!(
            parse_vdd_reply("P7501> STATUS\r\nVDD: 2.70V (OK)\r\n"),
            Some(2.70)
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

    #[test]
    fn stale_leading_prompt_does_not_abort_status_read() {
        assert_eq!(
            strip_leading_prompts("PANADIGA> \r\nVDD: 3.24V (OK)\r\n"),
            "VDD: 3.24V (OK)\r\n"
        );
        assert!(!reply_has_completed("PANADIGA> ", &["VDD:"]));
        assert!(!reply_has_completed("P7501> \r\n", &["VDD:"]));
        assert!(reply_has_completed(
            "PANADIGA> \r\nVDD: 3.24V (OK)\r\nPANADIGA> ",
            &["VDD:"]
        ));
        assert!(reply_has_completed(
            "[CLI] Unknown command\r\nPANADIGA> ",
            &["VDD:"]
        ));
    }
}
