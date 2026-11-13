//! CLI handlers for RP2040/RP2350 Pico hardware probing, CDDIAG DCSR check, memory DUMP, flashing, and GDB RSP bridge.

use super::super::args::{GdbConfigArgs, PicoActions, PicoArgs};
use anyhow::{Result, bail};
use panadiga_core::constants::AM34_NVRAM_BASE;
use panadiga_core::headers::hex_dump;
use panadiga_core::nvram::inspect_partial_nvram_capture;
use panadiga_core::pico::cdc::{
    open_cdc, parse_pico_dump_hex_to_bytes, parse_vdd_reply, send_and_read_until,
};
use panadiga_core::pico::{
    detect_bootsel_drives, flash_uf2_to_drive, list_pico_devices, merge_unified_uf2, ping_pico,
    run_gdb_bridge, trigger_1200_baud_bootsel,
};
use panadiga_core::{log_info, log_println};
use std::path::PathBuf;
use std::time::Duration;

fn parse_hex_u32(raw: &str) -> Result<u32> {
    let clean = raw.trim().trim_start_matches("0x").trim_start_matches("0X");
    u32::from_str_radix(clean, 16)
        .map_err(|_| anyhow::anyhow!("Invalid 32-bit hex address: '{}'", raw))
}

fn extract_dcsr_value(text: &str) -> Option<u32> {
    for token in text.split_whitespace() {
        let cleaned = token.trim_matches(|c: char| !c.is_ascii_hexdigit() && c != 'x' && c != 'X');
        if let Some(hex_body) = cleaned
            .strip_prefix("0x")
            .or_else(|| cleaned.strip_prefix("0X"))
            && hex_body.len() == 8
            && let Ok(val) = u32::from_str_radix(hex_body, 16)
        {
            return Some(val);
        }
    }
    None
}

fn describe_dcsr_state(dcsr: u32) -> &'static str {
    if dcsr == 0xFFFF_FFFF {
        "FLOATING HIGH (0xFFFFFFFF) - Target powered off or UART lines disconnected"
    } else if dcsr == 0x0000_0000 {
        "ACTIVE BUS / NO HALT BIT (0x00000000) - Typical during CDDIAG/factory loop"
    } else if (dcsr & 0x0000_0001) != 0 {
        "HALTED (Bit 0 = 1) - AM34 CPU frozen under debug control"
    } else {
        "ACTIVE BUS RESPONSE - Target responding on debug interface, Halt bit unset"
    }
}

fn query_pico_dcsr_flow(port: &str, baud: u32, cmd: &str) -> Result<()> {
    let mut ser = open_cdc(port, baud, Duration::from_millis(1200))?;
    let ping = send_and_read_until(&mut *ser, "PING", &["PONG"], Duration::from_millis(800));
    let vdd_raw = send_and_read_until(&mut *ser, "STATUS", &["VDD"], Duration::from_millis(800));
    let vdd = parse_vdd_reply(&vdd_raw);
    let reply = send_and_read_until(
        &mut *ser,
        cmd,
        &["PANADIGA>", "P7501>", "DCSR"],
        Duration::from_millis(2500),
    );

    log_println!("====================================================================");
    log_println!("   PANADIGA PICO P7501 ASYNC UART TELEMETRY ({})", port);
    log_println!("====================================================================");
    log_println!(
        "PING:   {}",
        if ping.contains("PONG") {
            "PONG OK"
        } else {
            ping.trim()
        }
    );
    if let Some(v) = vdd {
        log_println!("VDD:    {:.2} V (3.3V LVTTL target sense)", v);
    } else {
        log_println!("VDD:    {}", vdd_raw.trim());
    }
    if let Some(dcsr) = extract_dcsr_value(&reply) {
        log_println!("DCSR:   0x{:08X} -> {}", dcsr, describe_dcsr_state(dcsr));
    }
    log_println!("RAW:    {}", reply.trim());
    log_println!("====================================================================");
    Ok(())
}

pub fn handle_pico(args: PicoArgs) -> Result<()> {
    match args.action {
        PicoActions::List => {
            let devices = list_pico_devices()?;
            log_println!("====================================================================");
            log_println!("   PANADIGA HARDWARE DISCOVERY: SERIAL COM PORTS & PICO CONTROLLERS");
            log_println!("====================================================================");
            if devices.is_empty() {
                log_println!("No USB serial devices detected on system.");
            } else {
                for dev in &devices {
                    log_println!(
                        "Port: {:<10} | {:<32} | VID:{:04X} PID:{:04X} | {}",
                        dev.port_name,
                        dev.architecture.to_string(),
                        dev.vid,
                        dev.pid,
                        dev.description
                    );
                }
            }
            log_println!("--------------------------------------------------------------------");
        }
        PicoActions::Ping { port, baud } => {
            let report = ping_pico(&port, baud)?;
            log_println!("{}", report);
        }
        PicoActions::Status { port, baud } | PicoActions::Dcsr { port, baud } => {
            query_pico_dcsr_flow(&port, baud, "HANDSHAKE")?;
        }
        PicoActions::BreakIn { port, mode, baud } => {
            if mode > 3 {
                bail!("Invalid Cold Break-In mode {} (expected 0..=3)", mode);
            }
            let cmd = format!("BREAK_IN {}", mode);
            query_pico_dcsr_flow(&port, baud, &cmd)?;
        }
        PicoActions::Dump {
            port,
            addr,
            words,
            out,
            baud,
        } => {
            if words == 0 || words > 16384 {
                bail!(
                    "Word count {} out of supported range (1..=16384 words)",
                    words
                );
            }
            let base_addr = parse_hex_u32(&addr)?;
            let mut ser = open_cdc(&port, baud, Duration::from_millis(2000))?;
            let mut captured: Vec<u8> = Vec::with_capacity(words * 4);
            let mut remaining = words;
            let mut cursor_addr = base_addr;

            while remaining > 0 {
                let chunk_words = remaining.min(1024);
                let cmd = format!("DUMP 0x{:08X} {}", cursor_addr, chunk_words);
                let reply = send_and_read_until(
                    &mut *ser,
                    &cmd,
                    &["PANADIGA>", "P7501>", "ERR"],
                    Duration::from_millis(5000),
                );
                let chunk_bytes = parse_pico_dump_hex_to_bytes(&reply);
                if chunk_bytes.is_empty() {
                    bail!(
                        "Pico returned no word data for '{}' (raw response: {})",
                        cmd,
                        reply.trim()
                    );
                }
                cursor_addr = cursor_addr.wrapping_add(chunk_bytes.len() as u32);
                remaining = remaining.saturating_sub(chunk_bytes.len() / 4);
                captured.extend_from_slice(&chunk_bytes);
            }

            log_println!("====================================================================");
            log_println!(
                "   LIVE MONITOR APERTURE DUMP @ 0x{:08X} ({} bytes captured)",
                base_addr,
                captured.len()
            );
            log_println!("====================================================================");
            log_println!(
                "{}",
                hex_dump(&captured, base_addr as usize, captured.len().min(512))
            );

            if (AM34_NVRAM_BASE..AM34_NVRAM_BASE + 0x0001_0000).contains(&base_addr)
                && let Ok(report) =
                    inspect_partial_nvram_capture(&captured, base_addr, &format!("pico:{}", port))
            {
                log_println!(
                    "NVRAM Audit: non-0xFF={}/{} bytes (sectors={}) | SHA-256={}",
                    report.nvram_non_ff_bytes,
                    captured.len(),
                    report.microsectors.len(),
                    report.original_sha256
                );
            }

            if let Some(out_path) = out {
                if let Some(parent) = out_path.parent()
                    && !parent.as_os_str().is_empty()
                {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::write(&out_path, &captured)?;
                log_println!("[OK] Captured bytes saved to {}", out_path.display());
            }
        }
        PicoActions::GdbBridge { port, baud, listen } => {
            run_gdb_bridge(&port, baud, &listen)?;
        }
        PicoActions::Flash { port, file } => {
            log_println!("====================================================================");
            log_println!("   PANADIGA AUTOMATED PICO FIRMWARE FLASHER (P7501 UART + OCD)");
            log_println!("====================================================================");
            if let Some(ref p) = port {
                log_println!("Initiating 1200-baud touch reset on port {}...", p);
                let _ = trigger_1200_baud_bootsel(p);
            }
            let bootsel_drives = detect_bootsel_drives();
            if bootsel_drives.is_empty() {
                bail!(
                    "No Raspberry Pi BOOTSEL drives (RPI-RP2 / RP2350) detected! Hold BOOTSEL button while plugging USB."
                );
            }
            let target_drive = &bootsel_drives[0];
            log_println!("Found active BOOTSEL drive: {}", target_drive.display());

            let uf2_data = if let Some(ref custom_path) = file {
                std::fs::read(custom_path)?
            } else {
                let candidates = [
                    PathBuf::from("dist/panasonic_p7501_uart_unified.uf2"),
                    PathBuf::from(
                        "crates/panadiga-core/src/pico/firmware_bins/panasonic_p7501_uart_unified.uf2",
                    ),
                    PathBuf::from("firmware/pico-p7501/panasonic_p7501_uart_unified.uf2"),
                    PathBuf::from("dist/panasonic_p7501_uart_rp2040.uf2"),
                    PathBuf::from(
                        "crates/panadiga-core/src/pico/firmware_bins/panasonic_p7501_uart_rp2040.uf2",
                    ),
                    PathBuf::from("firmware/pico-p7501/panasonic_p7501_uart_rp2040.uf2"),
                ];
                let mut found = None;
                for c in &candidates {
                    if c.exists()
                        && let Ok(data) = std::fs::read(c)
                    {
                        found = Some(data);
                        break;
                    }
                }
                match found {
                    Some(data) => data,
                    None => panadiga_core::pico::flasher::EMBEDDED_UNIFIED_UF2.to_vec(),
                }
            };

            let report = flash_uf2_to_drive(target_drive, &uf2_data)?;
            log_println!("[SUCCESS] {}", report.message);
        }
        PicoActions::MergeUf2 {
            rp2040,
            rp2350,
            out,
        } => {
            let (blocks, sha256) = merge_unified_uf2(&rp2040, &rp2350, &out)?;
            log_info!(
                "[OK] Unified Universal UF2 generated: {} ({} blocks, SHA-256: {})",
                out.display(),
                blocks,
                sha256
            );
        }
    }
    Ok(())
}

pub fn handle_gdb_config(args: GdbConfigArgs) -> Result<()> {
    log_println!("====================================================================");
    log_println!("      Panadiga GDB RSP Bridge Configuration (P7501 Async UART / GDB RSP)");
    log_println!("====================================================================");
    log_println!("• Serial Port:   {}", args.port);
    log_println!("• Listen Socket: {}:{}", args.host, args.tcp_port);
    log_println!();
    log_println!("GDB Command to Connect:");
    log_println!("  (gdb) target remote {}:{}", args.host, args.tcp_port);
    log_println!("  (gdb) set architecture mn10300");
    log_println!("  (gdb) info registers");
    log_println!("====================================================================");
    Ok(())
}
