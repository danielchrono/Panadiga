//! Panasonic DMR-EH55 DIGA Suite - In-Circuit NVRAM Erase Executor
//!
//! Single Responsibility: In-circuit execution of RAM loader injection and
//! surgical sector erase on Panasonic DIGA NOR Flash via P7501 / RAM Loader.

use std::io::Write;
use std::time::{Duration, Instant};

use super::backup::{
    dump_p7501_region_bytes, open_and_handshake_p7501, save_live_backup_files, sha256_hex,
};
use super::parser::parse_autodetected_nvram_base;
use crate::pico::serial::read_serial_response_complete;
use crate::pico::types::InCircuitNvramEraseResult;

pub(crate) fn query_port_vdd(port: &mut dyn serialport::SerialPort) -> f32 {
    let _ = port.write_all(b"STATUS\r\n");
    let _ = port.flush();
    if let Ok(resp) = read_serial_response_complete(port, Duration::from_millis(600))
        && let Some(v) = panadiga_core::pico::cdc::parse_vdd_reply(&resp)
    {
        return v;
    }
    3.30
}

/// Executes full in-circuit sequence:
/// 1. Multi-mode Handshake (Mode 2 -> Mode 0 -> Mode 3 -> Mode 1) + Auto-Detect Flash Size
/// 2. Pre-Erase Live Secret & Full 64KB NVRAM Binary Dump (`nvram_live_backup.bin`, `sram_secrets.bin`, `ipl_bootcamp_header.bin`)
/// 3. `INJECT_LOADER CONFIRM` into SRAM at `0x00004000` (`AM34_SRAM_LOADER_BASE`)
/// 4. `ERASE_NVRAM CONFIRM` erasing Top-Boot 64KB EEMUL NVRAM
pub fn pico_execute_in_circuit_erase_full(
    port_name: String,
) -> Result<InCircuitNvramEraseResult, String> {
    let start_ts = Instant::now();
    let (_io_guard, mut port, hs_out) = open_and_handshake_p7501(&port_name)?;
    let voltage_before = query_port_vdd(&mut *port);
    let nvram_base_hex = parse_autodetected_nvram_base(&hs_out);
    let mut log_lines = vec![format!("[HANDSHAKE] {}", hs_out)];

    let is_16mb = nvram_base_hex == "0x80FE0000";
    let nvram_words = if is_16mb { 32768 } else { 16384 };
    let nvram_timeout = if is_16mb { 30000 } else { 15000 };
    let erase_timeout = if is_16mb { 20000 } else { 12000 };
    let bytes_erased = if is_16mb {
        panadiga_core::constants::FLASH_16MB_NVRAM_SIZE_BYTES as u32
    } else {
        panadiga_core::constants::SPANSION_NVRAM_SIZE_BYTES as u32
    };

    // Step 2: Pre-Erase Live Secret & Binary NVRAM Dump at auto-detected EEMUL base
    let nv_bytes = dump_p7501_region_bytes(&mut *port, nvram_base_hex, nvram_words, nvram_timeout);
    let sr_bytes = dump_p7501_region_bytes(&mut *port, "0x00000000", 4096, 6000);
    let ipl_bytes = dump_p7501_region_bytes(&mut *port, "0x80000000", 1024, 4000);
    save_live_backup_files(&nv_bytes, &sr_bytes, &ipl_bytes);

    let nv_sha256 = sha256_hex(&nv_bytes);
    log_lines.push(format!(
        "First-Contact Binary Backup ({}): nvram_live_backup.bin ({} bytes, SHA256={}), sram_secrets.bin ({} bytes) & ipl_bootcamp_header.bin ({} bytes) salvos.",
        nvram_base_hex,
        nv_bytes.len(),
        &nv_sha256[..16],
        sr_bytes.len(),
        ipl_bytes.len()
    ));

    // Step 3: INJECT_LOADER CONFIRM (into 0x00004000)
    port.write_all(b"INJECT_LOADER CONFIRM\r\n")
        .map_err(|e| format!("Falha ao enviar INJECT_LOADER CONFIRM: {}", e))?;
    let _ = port.flush();
    let inj_out = read_serial_response_complete(&mut *port, Duration::from_millis(5000))?;
    log_lines.push(inj_out.clone());
    if inj_out.contains("ERROR") || inj_out.contains("SAFETY INTERLOCK") {
        return Err(format!("Falha na injeção do RAM Loader: {}", inj_out));
    }

    // Step 4: ERASE_NVRAM CONFIRM (~5.2s across 8 micro-sectors)
    port.write_all(b"ERASE_NVRAM CONFIRM\r\n")
        .map_err(|e| format!("Falha ao enviar ERASE_NVRAM CONFIRM: {}", e))?;
    let _ = port.flush();
    let erase_out =
        read_serial_response_complete(&mut *port, Duration::from_millis(erase_timeout))?;
    for line in erase_out.lines() {
        log_lines.push(line.to_string());
    }
    let success = !erase_out.contains("ERROR") && !erase_out.contains("SAFETY INTERLOCK");
    let voltage_after = query_port_vdd(&mut *port);
    if success {
        crate::pico::serial::set_ram_loader_injected(true);
    }

    Ok(InCircuitNvramEraseResult {
        success,
        elapsed_ms: start_ts.elapsed().as_millis() as u64,
        voltage_before,
        voltage_after,
        ram_loader_addr: format!("0x{:08X}", panadiga_core::constants::AM34_SRAM_LOADER_BASE),
        bytes_erased,
        verification_sha256: nv_sha256,
        log_lines,
    })
}

/// Executes in-circuit surgical erase of Spansion S29JL064H NVRAM (string summary wrapper).
pub fn pico_execute_in_circuit_erase(port_name: String) -> Result<String, String> {
    let res = pico_execute_in_circuit_erase_full(port_name)?;
    Ok(res.log_lines.join("\n"))
}
