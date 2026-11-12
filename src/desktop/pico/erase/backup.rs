//! Panasonic DMR-EH55 DIGA Suite - Pre-Erase Secret Backup & Double-Read Verification
//!
//! Single Responsibility: Executes the non-destructive "What's Next" 1st-Contact
//! Read-Only Backup, double-read integrity check, and forensic binary serialization.

use sha2::{Digest, Sha256};
use std::fs;
use std::io::Write;
use std::time::Duration;

use super::parser::{parse_autodetected_nvram_base, parse_dwire_dump_hex_to_bytes};
use crate::desktop::commands::dto::detect_diga_model_from_slice;
use crate::desktop::pico::serial::{ensure_serial_port_unlocked, read_serial_response_complete};
use crate::desktop::pico::types::{FirstContactBackupResult, extract_port_name};

pub(crate) fn sha256_hex(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

pub(crate) fn open_and_handshake_dwire(
    clean_port: &str,
) -> Result<(Box<dyn serialport::SerialPort>, String), String> {
    ensure_serial_port_unlocked(clean_port)?;
    let mut port = crate::pico::cdc::open_cdc(clean_port, 115200, Duration::from_millis(10000))
        .map_err(|e| format!("Falha ao conectar na porta serial {}: {}", clean_port, e))?;

    port.write_all(b"HANDSHAKE\r\n")
        .map_err(|e| format!("Falha ao enviar HANDSHAKE: {}", e))?;
    let _ = port.flush();
    let hs_out = read_serial_response_complete(&mut *port, Duration::from_millis(3000))?;
    if !hs_out.contains("[HANDSHAKE] SUCCESS") {
        return Err(format!(
            "Cold Break-In falhou ao capturar o chip Matsushita AM34 (gravador desligado da tomada VDD=0V ou fiação solta). Resposta: {}",
            hs_out.trim()
        ));
    }
    Ok((port, hs_out))
}

pub(crate) fn dump_dwire_region_bytes(
    port: &mut dyn serialport::SerialPort,
    addr_hex: &str,
    words: u32,
    timeout_ms: u64,
) -> Vec<u8> {
    let cmd = format!("DUMP {} {}\r\n", addr_hex, words);
    let _ = port.write_all(cmd.as_bytes());
    let _ = port.flush();
    let ascii =
        read_serial_response_complete(port, Duration::from_millis(timeout_ms)).unwrap_or_default();
    parse_dwire_dump_hex_to_bytes(&ascii)
}

pub(crate) fn save_live_backup_files(nvram: &[u8], sram: &[u8], ipl: &[u8]) -> String {
    // Timestamped, never-overwriting directory: the first-contact capture is immutable evidence.
    let base = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let out_dir = base.join(format!("first_contact_{}", ts));
    let _ = fs::create_dir_all(&out_dir);
    let _ = fs::write(out_dir.join("nvram_live_backup.bin"), nvram);
    let _ = fs::write(out_dir.join("sram_secrets.bin"), sram);
    let _ = fs::write(out_dir.join("ipl_bootcamp_header.bin"), ipl);
    let manifest = format!(
        "{}  nvram_live_backup.bin\n{}  sram_secrets.bin\n{}  ipl_bootcamp_header.bin\n",
        sha256_hex(nvram),
        sha256_hex(sram),
        sha256_hex(ipl)
    );
    let _ = fs::write(out_dir.join("SHA256SUMS.txt"), manifest);
    out_dir.display().to_string()
}

/// True when a captured buffer is empty or one repeated byte (floating or shorted bus).
pub(crate) fn is_uniform_bus_pattern(data: &[u8]) -> bool {
    match data.first() {
        None => true,
        Some(&b0) => data.iter().all(|&b| b == b0),
    }
}

/// Executes the non-destructive "What's Next" 1st-Contact Read-Only Backup:
/// 1. `HANDSHAKE` + Passive Address Aliasing Auto-Detect (`2MB` / `4MB` / `8MB` / `16MB`)
/// 2. Double-Read of 64KB NVRAM at auto-detected EEMUL base verifying identical SHA-256
/// 3. Read of 16KB SRAM (`0x00000000`) & 4KB Bootloader IPL Header (`0x80000000`)
/// 4. Auto-Detection of Panasonic DIGA Model (`DMR-EH55`, `DMR-ES15`, `DMR-EX75`, `DMR-EZ27`, etc.)
pub fn pico_execute_first_contact_backup(
    port_name: String,
) -> Result<FirstContactBackupResult, String> {
    let clean = extract_port_name(&port_name);

    if clean.starts_with("MOCK") {
        let mut ipl_bin = crate::constants::PANA_FRM_GLOBAL_HEADER.to_vec();
        ipl_bin.extend_from_slice(b"PANASONIC DMR-EH55 IPL BOOTLOADER");
        let nvram_len = crate::constants::SPANSION_NVRAM_SIZE_BYTES;
        let digest = sha256_hex(&vec![0xFFu8; nvram_len]);
        let detected = detect_diga_model_from_slice(&ipl_bin);

        return Ok(FirstContactBackupResult {
            success: true,
            detected_model: detected.clone(),
            nvram_bytes: nvram_len,
            nvram_sha256: digest.clone(),
            double_read_verified: true,
            sram_bytes: 16384,
            ipl_bytes: ipl_bin.len(),
            log_lines: vec![
                format!("[MODEL DETECTED] {}", detected),
                format!(
                    "[DOUBLE-READ OK] 64KB NVRAM SHA-256 verified identical ({})",
                    &digest[..16]
                ),
                "Read-Only Backup complete. Zero bytes modified on target flash.".to_string(),
            ],
        });
    }

    let (mut port, hs_out) = open_and_handshake_dwire(&clean)?;
    let nvram_base_hex = parse_autodetected_nvram_base(&hs_out);
    let mut log_lines = vec![hs_out];

    let is_16mb = nvram_base_hex == "0x80FE0000";
    let nvram_words = if is_16mb { 32768 } else { 16384 };
    let nvram_timeout = if is_16mb { 30000 } else { 15000 };

    // Double-Read NVRAM at auto-detected EEMUL base to confirm zero bit-error on DWire32A cable
    let nv_bytes_1 =
        dump_dwire_region_bytes(&mut *port, nvram_base_hex, nvram_words, nvram_timeout);
    let nv_bytes_2 =
        dump_dwire_region_bytes(&mut *port, nvram_base_hex, nvram_words, nvram_timeout);
    let sha_1 = sha256_hex(&nv_bytes_1);
    let sha_2 = sha256_hex(&nv_bytes_2);

    let sr_bytes = dump_dwire_region_bytes(&mut *port, "0x00000000", 4096, 6000);
    let ipl_bytes = dump_dwire_region_bytes(&mut *port, "0x80000000", 1024, 4000);

    // A floating (0xFF..) or shorted (0x00..) bus yields identical reads; that is NOT a valid capture.
    // Require explicit HALT confirmation and a non-uniform bootloader header.
    let bus_alive =
        log_lines[0].contains("[HANDSHAKE] SUCCESS") && !is_uniform_bus_pattern(&ipl_bytes);
    let double_read_verified = bus_alive && !nv_bytes_1.is_empty() && sha_1 == sha_2;

    // Auto-detect DIGA model from IPL Header or NVRAM strings
    let mut combined_probe = ipl_bytes.clone();
    combined_probe.extend_from_slice(&nv_bytes_1);
    let detected_model = detect_diga_model_from_slice(&combined_probe);

    if double_read_verified {
        let dir = save_live_backup_files(&nv_bytes_1, &sr_bytes, &ipl_bytes);
        log_lines.push(format!(
            "[DOUBLE-READ VERIFIED] SHA-256 idêntico nas 2 leituras ({} @ {}). Modelo detectado: {}. Salvo em {}/",
            &sha_1[..16],
            nvram_base_hex,
            detected_model,
            dir
        ));
    } else if !bus_alive {
        log_lines.push(
            "[ABORTADO] CPU não confirmada em HALT ou barramento flutuante/em curto (IPL uniforme). Nada foi salvo. Verifique a pinagem e o GND.".to_string(),
        );
    } else {
        log_lines.push(format!(
            "[ALERTA DE INTEGRIDADE] Leituras divergentes no barramento (Read1={} vs Read2={}). Verifique o cabo de 6 pinos!",
            &sha_1[..12],
            &sha_2[..12]
        ));
    }

    Ok(FirstContactBackupResult {
        success: double_read_verified,
        detected_model,
        nvram_bytes: nv_bytes_1.len(),
        nvram_sha256: sha_1,
        double_read_verified,
        sram_bytes: sr_bytes.len(),
        ipl_bytes: ipl_bytes.len(),
        log_lines,
    })
}
