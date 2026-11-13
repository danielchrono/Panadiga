//! Panasonic DMR-EH55 DIGA Suite - Granular Pico Diagnostic Execution & Parser
//!
//! Single Responsibility: Validates, executes, and structurally parses all
//! granular P7501 UART/FMU, electrical matrix, and NOR Flash probes.

use serde::{Deserialize, Serialize};
use std::io::Write;
use std::time::{Duration, Instant};

use crate::commands::dto::build_hardware_ssot_spec;
use crate::pico::serial::{ensure_serial_port_unlocked, read_serial_response_complete};
use crate::pico::types::extract_port_name;
use panadiga_core::pico::cdc::{parse_pico_dump_lines, parse_vdd_reply};

/// Single pin row parsed from `PINCHECK`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PincheckRowDto {
    pub pin_label: String,
    pub hiz: String,
    pub pull_down: String,
    pub pull_up: String,
    pub state: String,
}

/// Signal transition / level row parsed from `SNIFF` or `BUSDIAG`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusTransitionRowDto {
    pub signal: String,
    pub gpio: String,
    pub final_level: Option<u8>,
    pub transitions: u32,
}

/// Decoded 32-bit word row parsed from `DUMP`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DumpWordRowDto {
    pub address: String,
    pub words: Vec<String>,
    pub ascii: String,
}

/// Structured result of running any granular Pico firmware diagnostic command.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PicoDiagnosticResultDto {
    pub command_id: String,
    pub raw_command: String,
    pub success: bool,
    pub elapsed_ms: u64,
    pub summary: String,
    pub raw_output: String,
    pub vdd_volts: Option<f32>,
    pub detected_flash_mb: Option<u32>,
    pub xreset_rise_us: Option<String>,
    pub pincheck_rows: Vec<PincheckRowDto>,
    pub transition_rows: Vec<BusTransitionRowDto>,
    pub dump_rows: Vec<DumpWordRowDto>,
    pub captured_bytes: Option<u32>,
}

/// Validates command ID & optional argument against SSOT catalog and prevents CDC injection.
pub fn compose_diagnostic_command(
    command_id: &str,
    arg: Option<&str>,
) -> Result<(String, Duration), String> {
    let normalized = command_id.trim().to_ascii_uppercase();
    let spec = build_hardware_ssot_spec();
    let entry = spec
        .pico_commands
        .iter()
        .find(|c| c.id == normalized)
        .ok_or_else(|| format!("Comando diagnóstico desconhecido: '{}'", command_id))?;

    let clean_arg = arg
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| {
            if s.chars().any(|c| c == '\r' || c == '\n' || c.is_control()) {
                return Err("Argumento contém caracteres de controle inválidos.".to_string());
            }
            Ok(s)
        })
        .transpose()?;

    // Enforce bounded parameter formats for critical commands
    if normalized == "BREAK_IN" {
        if let Some(m) = clean_arg
            && !matches!(m, "0" | "1" | "2" | "3")
        {
            return Err(format!("Modo Break-In inválido '{}': use 0, 1, 2 ou 3.", m));
        }
    } else if normalized == "FLASH_SIZE" {
        if let Some(sz) = clean_arg
            && !matches!(sz, "2" | "4" | "8" | "16")
        {
            return Err(format!(
                "Tamanho Flash inválido '{}': use 2, 4, 8 ou 16.",
                sz
            ));
        }
    } else if normalized == "LED"
        && let Some(mode) = clean_arg
    {
        let upper = mode.to_ascii_uppercase();
        if !matches!(upper.as_str(), "IDLE" | "ACTIVE" | "GDB" | "FAULT") {
            return Err("Modo LED inválido: use IDLE, ACTIVE, GDB ou FAULT.".to_string());
        }
    }

    let raw_cmd = match clean_arg {
        Some(a) => format!("{} {}", normalized, a),
        None => normalized.clone(),
    };

    // Scale wait window dynamically for variable-duration captures
    let mut timeout_ms = entry.timeout_ms;
    if matches!(
        normalized.as_str(),
        "SNIFF" | "BOOTCAP" | "UARTMON" | "JIGBOOT"
    ) && let Some(ms_str) = clean_arg
        && let Ok(ms) = ms_str.parse::<u64>()
    {
        timeout_ms = ms.saturating_add(2000).clamp(2000, 35000);
    }

    Ok((raw_cmd, Duration::from_millis(timeout_ms)))
}

fn parse_pincheck_rows(raw_output: &str) -> (Vec<PincheckRowDto>, Option<String>) {
    let mut rows = Vec::new();
    let mut rise_time = None;

    for line in raw_output.lines() {
        let trimmed = line.trim();
        if trimmed.contains('|')
            && (trimmed.starts_with("GP") || trimmed.contains("Pin"))
            && !trimmed.starts_with("Pin ")
            && !trimmed.contains("---+")
        {
            let parts: Vec<&str> = trimmed.split('|').map(str::trim).collect();
            if parts.len() >= 5 {
                rows.push(PincheckRowDto {
                    pin_label: parts[0].to_string(),
                    hiz: parts[1].to_string(),
                    pull_down: parts[2].to_string(),
                    pull_up: parts[3].to_string(),
                    state: parts[4].to_string(),
                });
            }
        } else if let Some((_, rhs)) = trimmed.split_once("XRESET Release Rise Time")
            && let Some((_, val)) = rhs.split_once(':')
        {
            rise_time = Some(val.trim().to_string());
        }
    }
    (rows, rise_time)
}

fn parse_transition_rows(raw_output: &str) -> Vec<BusTransitionRowDto> {
    let mut rows = Vec::new();
    for line in raw_output.lines() {
        let trimmed = line.trim();
        if trimmed.contains("Final=")
            && trimmed.contains("Transitions=")
            && let Some((left, right)) = trimmed.split_once(':')
        {
            let final_level = right
                .split("Final=")
                .nth(1)
                .and_then(|s| s.split([',', ' ']).next())
                .and_then(|n| n.parse::<u8>().ok());
            let transitions = right
                .split("Transitions=")
                .nth(1)
                .and_then(|s| s.split_whitespace().next())
                .and_then(|n| n.parse::<u32>().ok())
                .unwrap_or(0);
            let parts: Vec<&str> = left.split_whitespace().collect();
            let signal = parts.first().copied().unwrap_or("SIG").to_string();
            let gpio = parts
                .get(1)
                .map(|g| g.trim_matches(['(', ')']).to_string())
                .unwrap_or_default();
            rows.push(BusTransitionRowDto {
                signal,
                gpio,
                final_level,
                transitions,
            });
        }
    }
    rows
}

fn parse_dump_rows(raw_output: &str) -> Vec<DumpWordRowDto> {
    parse_pico_dump_lines(raw_output)
        .into_iter()
        .map(|row| {
            let ascii = row
                .raw_bytes
                .iter()
                .map(|&b| {
                    if (0x20..=0x7E).contains(&b) {
                        b as char
                    } else {
                        '.'
                    }
                })
                .collect();
            DumpWordRowDto {
                address: row.address,
                words: row.words,
                ascii,
            }
        })
        .collect()
}

fn parse_captured_bytes(raw_output: &str) -> Option<u32> {
    for line in raw_output.lines() {
        if let Some(pos) = line.find(" bytes") {
            let prefix = &line[..pos];
            if let Some(token) = prefix.split(['(', ' ', ':']).next_back()
                && let Ok(val) = token.trim().parse::<u32>()
            {
                return Some(val);
            }
        }
    }
    None
}

fn parse_flash_mb(raw_output: &str) -> Option<u32> {
    for mb in [16u32, 8, 4, 2] {
        let needle = format!("{} MB", mb);
        if raw_output.contains(&needle) {
            return Some(mb);
        }
    }
    None
}

fn parse_voltage_cell(cell: &str) -> Option<f32> {
    cell.trim()
        .trim_end_matches(['V', 'v'])
        .trim()
        .parse::<f32>()
        .ok()
        .filter(|v| v.is_finite())
}

/// Extracts P7501 Pin 1 (`SW_UART` / `GP26`) target rail voltage from structured `PINCHECK` rows,
/// prioritizing the Pull-Down test (`< 1.50 V`) so floating/unplugged wires never report false HIGH.
pub fn extract_pincheck_vdd(rows: &[PincheckRowDto]) -> Option<f32> {
    let vdd_row = rows.iter().find(|r| {
        r.pin_label.contains("SWUART")
            || r.pin_label.contains("GP26")
            || r.pin_label.contains("VDD")
    })?;
    let pd = parse_voltage_cell(&vdd_row.pull_down);
    let hiz = parse_voltage_cell(&vdd_row.hiz);
    if let Some(pd_val) = pd
        && pd_val < 1.50
    {
        return Some(pd_val);
    }
    hiz.or(pd)
}

/// Pure structured parser converting raw Pico CLI output into typed diagnostic metrics.
pub fn parse_diagnostic_output(
    command_id: &str,
    raw_command: &str,
    raw_output: &str,
    elapsed_ms: u64,
) -> PicoDiagnosticResultDto {
    let norm = command_id.trim().to_ascii_uppercase();
    let lower = raw_output.to_ascii_lowercase();
    let failed = lower.contains("failed")
        || lower.contains("error")
        || lower.contains("abort")
        || lower.contains("safety interlock")
        || raw_output.trim().is_empty();

    let (pincheck_rows, xreset_rise_us) = if norm == "PINCHECK" {
        parse_pincheck_rows(raw_output)
    } else {
        (Vec::new(), None)
    };
    let transition_rows = parse_transition_rows(raw_output);
    let dump_rows = if norm == "DUMP" {
        parse_dump_rows(raw_output)
    } else {
        Vec::new()
    };
    let captured_bytes = parse_captured_bytes(raw_output);
    let detected_flash_mb = parse_flash_mb(raw_output);
    let vdd_volts = parse_vdd_reply(raw_output).or_else(|| extract_pincheck_vdd(&pincheck_rows));

    let summary = if let Some(last_meaningful) = raw_output
        .lines()
        .rev()
        .map(str::trim)
        .find(|l| !l.is_empty() && !l.starts_with("PANADIGA>") && !l.starts_with("P7501>"))
    {
        last_meaningful.to_string()
    } else if raw_output.trim().is_empty() {
        format!("Sem resposta no barramento para '{}'", raw_command)
    } else {
        raw_output.trim().to_string()
    };

    PicoDiagnosticResultDto {
        command_id: norm,
        raw_command: raw_command.to_string(),
        success: !failed,
        elapsed_ms,
        summary,
        raw_output: raw_output.to_string(),
        vdd_volts,
        detected_flash_mb,
        xreset_rise_us,
        pincheck_rows,
        transition_rows,
        dump_rows,
        captured_bytes,
    }
}

/// Executes a granular diagnostic command against the real USB CDC Pico or mock bench simulator.
pub fn execute_diagnostic_on_port(
    port_name: &str,
    command_id: &str,
    arg: Option<&str>,
) -> Result<PicoDiagnosticResultDto, String> {
    let (raw_cmd, timeout) = compose_diagnostic_command(command_id, arg)?;
    let clean = extract_port_name(port_name);
    let started = Instant::now();

    if clean.starts_with("MOCK") {
        #[cfg(test)]
        {
            let mock_out = simulate_mock_diagnostic(&raw_cmd);
            return Ok(parse_diagnostic_output(command_id, &raw_cmd, &mock_out, 45));
        }
        #[cfg(not(test))]
        return crate::pico::types::reject_mock_port(&clean).map(|_| unreachable!());
    }

    if clean.starts_with("BLE") || clean.starts_with("ANDROID-USB-OTG") {
        return Err(
            "Comandos em modo BLE NUS / USB-OTG Android devem ser despachados diretamente pela camada móvel."
                .to_string(),
        );
    }

    let _io_guard = crate::pico::serial::lock_serial_io();
    ensure_serial_port_unlocked(&clean)?;
    let mut port = panadiga_core::pico::cdc::open_cdc(&clean, 115200, Duration::from_millis(4000))
        .map_err(|e| format!("Falha ao abrir porta '{}': {}", clean, e))?;

    let wire = format!("{}\r\n", raw_cmd);
    port.write_all(wire.as_bytes())
        .map_err(|e| format!("Falha ao enviar '{}' para Pico: {}", raw_cmd, e))?;
    let _ = port.flush();

    let raw_out = read_serial_response_complete(&mut *port, timeout)?;
    let elapsed_ms = started.elapsed().as_millis() as u64;
    Ok(parse_diagnostic_output(
        command_id, &raw_cmd, &raw_out, elapsed_ms,
    ))
}

#[cfg(test)]
fn simulate_mock_diagnostic(raw_cmd: &str) -> String {
    let upper = raw_cmd.trim().to_ascii_uppercase();
    if upper == "PINCHECK" {
        return "\
=== P7501 6-PIN ELECTRICAL MATRIX (PINCHECK) ===
Pin               | Hi-Z | Pull-Down | Pull-Up | Electrical State
------------------+------+-----------+---------+----------------------------------------
GP2 (Pin6 RXD   ) |  1   |     0     |    1    | HI-Z / UART RX 115200 8N1 (R7572 Pull-Up)
GP3 (Pin5 NSW5V ) |  1   |     1     |    1    | HI-Z SENSE ONLY (Static rail ~3.29V; IC1506 ~5.15V)
GP4 (Pin3 TXD   ) |  1   |     1     |    1    | IDLE HIGH (R7570 10k / Q7514 TX)
GP5 (Pin2 XRESET) |  1   |     1     |    1    | STRONG HIGH (Open-Drain Control)
GP26(Pin1 SWUART) | 3.28V|   2.69V   |  3.29V  | SW_UART / ADC0 Rail Sense
------------------------------------------------------------------------
While GP5 (XRESET) asserted LOW (0V): GP2=1, GP3=1, GP4=1, GP5=0
XRESET Release Rise Time (Hi-Z)     : 40 us
Post-Reset 250ms Transitions        : GP2=872, GP3=0, GP4=0, GP5=2
PINCHECK: OK"
            .to_string();
    }
    if upper.starts_with("SNIFF") {
        return "\
SNIFF RESULT (250 ms):
  RXD/EXT(GP2) : Final=1, Transitions=0
  NSW5V  (GP3) : Final=1, Transitions=24
  TXD/DAT(GP4) : Final=1, Transitions=68
  XRESET (GP5) : Final=1, Transitions=0
  [STATUS] ACTIVE BUS ACTIVITY DETECTED! (GP2=0, GP3=24, GP4=68, GP5=0)
SNIFF: OK"
            .to_string();
    }
    if upper == "BUSDIAG" {
        return "\
=== P7501 ELECTRICAL CONTENTION & DUAL-CHANNEL DIAGNOSTIC (BUSDIAG) ===
State 0 (SW_UART=HiZ, XRESET=HiZ):
  GP4 (MAX RXD) @ 2mA  -> pad_low=0, release_rise_ticks=12
  GP4 (MAX RXD) @ 12mA -> pad_low=0, release_rise_ticks=8
Boot Transitions (NORMAL SW_UART=HiZ):
  Phase A (0..1300ms)   : GP2=284 GP3=12 GP4=420 GP5=2
  Phase B (1300..3500ms): GP2=96 GP3=0 GP4=140 GP5=0
BUSDIAG: OK"
            .to_string();
    }
    if upper == "CFI" {
        return "CFI: OK (Spansion S29JL064H QRY Validated, Size: 8 MB / 64 Mbit)".to_string();
    }
    if upper.starts_with("DUMP") {
        return "\
[DUMP] Reading 16 words (64 bytes) from 0x807F0000...
807F0000: FFFFFFFF FFFFFFFF 55AA0100 00000001
807F0010: 44494741 35354845 00000000 FFFFFFFF
807F0020: FFFFFFFF FFFFFFFF FFFFFFFF FFFFFFFF
807F0030: FFFFFFFF FFFFFFFF FFFFFFFF FFFFFFFF
DUMP: OK"
            .to_string();
    }
    if upper.starts_with("BOOTCAP") || upper.starts_with("UARTMON") || upper.starts_with("JIGBOOT")
    {
        return "\
=== BOOTCAP: PANASONIC 115200 8N1 UART CAPTURE ON GP2 (42 bytes, 7500 ms) ===
AM34 IPL BOOT v1.18 OK | DDR CHECK OK
BOOTCAP: OK"
            .to_string();
    }
    if upper.starts_with("FMU") || upper.starts_with("UART") {
        return "UART/FMU Probe Capture (18 bytes): SERI_READY OK\nFMU: OK".to_string();
    }
    if upper == "SCAN" {
        return "[SCAN] Probing 4 Modes x 256 Opcodes (1024 Vectors)...\nMode 2 Opcode 0x01: HIT! DCSR=0x00000001 (HALTED)\nSCAN: COMPLETED".to_string();
    }
    if upper == "AUTODETECT" {
        return "AUTODETECT: OK (8 MB NOR Flash, EEMUL NVRAM @ 0x807F0000)".to_string();
    }
    if upper.starts_with("LED")
        || upper == "ISOLATE"
        || upper == "ENGAGE"
        || upper == "RESET"
        || upper == "BANNER"
        || upper == "PING"
        || upper == "STATUS"
    {
        return format!(
            "{}: OK VDD=3.28V",
            upper.split_whitespace().next().unwrap_or("CMD")
        );
    }
    format!("{}: OK", upper)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compose_command_validates_modes_and_blocks_injection() {
        assert!(compose_diagnostic_command("NOPE", None).is_err());
        assert!(compose_diagnostic_command("BREAK_IN", Some("9")).is_err());
        assert!(compose_diagnostic_command("FLASH_SIZE", Some("32")).is_err());
        assert!(compose_diagnostic_command("SNIFF", Some("250\nRESET")).is_err());
        let (cmd, dur) = compose_diagnostic_command("BOOTCAP", Some("3000")).unwrap();
        assert_eq!(cmd, "BOOTCAP 3000");
        assert_eq!(dur, Duration::from_millis(5000));
    }

    #[test]
    fn parses_pincheck_dump_and_sniff_outputs_cleanly() {
        let pincheck = execute_diagnostic_on_port("MOCK_RP2040", "PINCHECK", None).unwrap();
        assert!(pincheck.success);
        assert_eq!(pincheck.pincheck_rows.len(), 5);
        assert_eq!(pincheck.xreset_rise_us.as_deref(), Some("40 us"));
        assert!((pincheck.vdd_volts.unwrap() - 3.28).abs() < 0.01);

        let dump =
            execute_diagnostic_on_port("MOCK_RP2040", "DUMP", Some("0x807F0000 16")).unwrap();
        assert!(dump.success);
        assert_eq!(dump.dump_rows.len(), 4);
        assert_eq!(dump.dump_rows[1].address, "0x807F0010");

        let sniff = execute_diagnostic_on_port("MOCK_RP2040", "SNIFF", Some("250")).unwrap();
        assert!(sniff.success);
        assert_eq!(sniff.transition_rows.len(), 4);
        assert_eq!(sniff.transition_rows[2].transitions, 68);
    }

    #[test]
    fn pincheck_vdd_rejects_floating_pulldown_even_when_hiz_is_high() {
        let floating = "\
=== P7501 6-PIN ELECTRICAL MATRIX (PINCHECK) ===
GP26(Pin1 SWUART) | 3.21V|   0.00V   |  3.28V  | P7501 SW_UART / Target Sense
PINCHECK: OK";
        let res = parse_diagnostic_output("PINCHECK", "PINCHECK", floating, 40);
        assert_eq!(res.vdd_volts, Some(0.0));
    }
}
