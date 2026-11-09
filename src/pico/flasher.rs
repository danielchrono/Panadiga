//! Raspberry Pi Pico (RP2040 & RP2350) Automated Firmware Flasher
//!
//! Provides zero-friction USB flashing for DWire32A probe controllers:
//! 1. Detection of BOOTSEL USB Mass Storage drives (`RPI-RP2` / `RP2350` via `INFO_UF2.TXT`).
//! 2. Automatic 1200-baud touch reset to transition running Pico CDC devices into BOOTSEL mode.
//! 3. Binary UF2 validation, integrity hashing, and drag-and-drop file streaming.

use anyhow::{bail, Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

/// Magic numbers identifying Microsoft UF2 (USB Flashing Format) containers.
pub const UF2_MAGIC_START0: u32 = 0x0A32_4655; // "UF2\n" in little-endian
pub const UF2_MAGIC_START1: u32 = 0x9E5D_5157;
pub const UF2_MAGIC_END: u32 = 0x0AB1_6F30;
pub const UF2_BLOCK_SIZE: usize = 512;
pub const UF2_PAYLOAD_SIZE: usize = 256;

/// Raspberry Pi RP2040 and RP2350 UF2 Family IDs.
pub const RP2040_FAMILY_ID: u32 = 0xE484_9208;
pub const RP2350_ARM_S_FAMILY_ID: u32 = 0xE484_920A;
pub const RP2350_RISCV_FAMILY_ID: u32 = 0xE484_920B;

/// Result summary of an automated Pico flashing operation.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PicoFlashReport {
    pub target_drive: String,
    pub uf2_size_bytes: usize,
    pub blocks_written: usize,
    pub board_family: String,
    pub sha256_hash: String,
    pub success: bool,
    pub message: String,
}

/// Validates that a buffer conforms to the UF2 specification.
pub fn validate_uf2_buffer(data: &[u8]) -> Result<(usize, String)> {
    if data.len() < UF2_BLOCK_SIZE || !data.len().is_multiple_of(UF2_BLOCK_SIZE) {
        bail!(
            "Invalid UF2 payload length: {} bytes (must be a positive multiple of 512 bytes)",
            data.len()
        );
    }

    let num_blocks = data.len() / UF2_BLOCK_SIZE;

    // Check first block magic numbers
    let start0 = u32::from_le_bytes(data[0..4].try_into().unwrap());
    let start1 = u32::from_le_bytes(data[4..8].try_into().unwrap());
    let end_magic = u32::from_le_bytes(data[508..512].try_into().unwrap());

    if start0 != UF2_MAGIC_START0 || start1 != UF2_MAGIC_START1 || end_magic != UF2_MAGIC_END {
        bail!(
            "Corrupt UF2 block magic (expected 0x{:08X}/0x{:08X}/0x{:08X}, got 0x{:08X}/0x{:08X}/0x{:08X})",
            UF2_MAGIC_START0,
            UF2_MAGIC_START1,
            UF2_MAGIC_END,
            start0,
            start1,
            end_magic
        );
    }

    // Inspect family ID from block 0
    let family_id = u32::from_le_bytes(data[28..32].try_into().unwrap());
    let family_name = match family_id {
        RP2040_FAMILY_ID => "Raspberry Pi RP2040".to_string(),
        RP2350_ARM_S_FAMILY_ID => "Raspberry Pi Pico 2 ARM-S (RP2350)".to_string(),
        RP2350_RISCV_FAMILY_ID => "Raspberry Pi Pico 2 RISC-V (RP2350)".to_string(),
        other => format!("Pico-Compatible Controller (Family ID 0x{:08X})", other),
    };

    Ok((num_blocks, family_name))
}

/// Discovers mounted BOOTSEL mass storage drives across Windows, Linux, and macOS.
pub fn detect_bootsel_drives() -> Vec<PathBuf> {
    let mut bootsel_drives = Vec::new();

    #[cfg(windows)]
    {
        for letter in b'D'..=b'Z' {
            let root = format!("{}:\\", letter as char);
            let path = PathBuf::from(&root);
            if path.exists() {
                let info_file = path.join("INFO_UF2.TXT");
                let index_file = path.join("INDEX.HTM");
                if info_file.exists() || index_file.exists() {
                    bootsel_drives.push(path);
                }
            }
        }
    }

    #[cfg(unix)]
    {
        let candidate_roots = [
            "/media",
            "/run/media",
            "/Volumes", // macOS
        ];

        for root in candidate_roots {
            let root_path = Path::new(root);
            if let Ok(entries) = fs::read_dir(root_path) {
                for entry in entries.flatten() {
                    let p = entry.path();
                    if p.is_dir() {
                        // Check inside user subfolders (e.g. /media/user/RPI-RP2)
                        if let Ok(sub_entries) = fs::read_dir(&p) {
                            for sub in sub_entries.flatten() {
                                let sp = sub.path();
                                if sp.join("INFO_UF2.TXT").exists() {
                                    bootsel_drives.push(sp);
                                }
                            }
                        }
                        if p.join("INFO_UF2.TXT").exists() {
                            bootsel_drives.push(p);
                        }
                    }
                }
            }
        }
    }

    bootsel_drives
}

/// Triggers an automatic reboot into BOOTSEL mode by opening and closing the serial port at 1200 baud.
pub fn trigger_1200_baud_bootsel(port_name: &str) -> Result<()> {
    // Standard 1200 baud touch pattern used by Pico SDK / Arduino
    match serialport::new(port_name, 1200)
        .timeout(Duration::from_millis(500))
        .open()
    {
        Ok(mut port) => {
            // Drop DTR / RTS to trigger firmware reset hook
            let _ = port.write_data_terminal_ready(false);
            let _ = port.write_request_to_send(false);
            thread::sleep(Duration::from_millis(150));
            drop(port);
            // Allow USB bus re-enumeration as Mass Storage
            thread::sleep(Duration::from_millis(800));
            Ok(())
        }
        Err(e) => {
            bail!(
                "Failed to initiate 1200-baud touch reset on port {}: {}",
                port_name,
                e
            );
        }
    }
}

/// Flashes a validated UF2 image buffer directly onto a detected BOOTSEL drive.
pub fn flash_uf2_to_drive(drive_path: &Path, uf2_bytes: &[u8]) -> Result<PicoFlashReport> {
    let (blocks_count, family_name) = validate_uf2_buffer(uf2_bytes)?;

    let target_file = drive_path.join("panadiga_dwire32a.uf2");
    fs::write(&target_file, uf2_bytes).with_context(|| {
        format!(
            "Failed to write UF2 firmware to destination drive at {}",
            target_file.display()
        )
    })?;

    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(uf2_bytes);
    let hash_hex = hex::encode(hasher.finalize());

    Ok(PicoFlashReport {
        target_drive: drive_path.to_string_lossy().to_string(),
        uf2_size_bytes: uf2_bytes.len(),
        blocks_written: blocks_count,
        board_family: family_name,
        sha256_hash: hash_hex,
        success: true,
        message: format!(
            "Successfully flashed {} blocks to {}. Pico will reboot into DWire32A mode.",
            blocks_count,
            drive_path.display()
        ),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uf2_buffer_validation_valid() {
        // Construct a synthetic 512-byte valid UF2 block for RP2040
        let mut block = vec![0u8; 512];
        block[0..4].copy_from_slice(&UF2_MAGIC_START0.to_le_bytes());
        block[4..8].copy_from_slice(&UF2_MAGIC_START1.to_le_bytes());
        block[28..32].copy_from_slice(&RP2040_FAMILY_ID.to_le_bytes());
        block[508..512].copy_from_slice(&UF2_MAGIC_END.to_le_bytes());

        let res = validate_uf2_buffer(&block);
        assert!(res.is_ok());
        let (blocks, family) = res.unwrap();
        assert_eq!(blocks, 1);
        assert!(family.contains("RP2040"));
    }

    #[test]
    fn test_uf2_buffer_validation_invalid_magic() {
        let block = vec![0u8; 512];
        let res = validate_uf2_buffer(&block);
        assert!(res.is_err());
        assert!(res.unwrap_err().to_string().contains("Corrupt UF2 block"));
    }

    #[test]
    fn test_uf2_buffer_validation_invalid_length() {
        let block = vec![0u8; 400];
        let res = validate_uf2_buffer(&block);
        assert!(res.is_err());
        assert!(res.unwrap_err().to_string().contains("multiple of 512"));
    }
}
