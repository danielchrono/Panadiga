//! Raspberry Pi Pico (RP2040 & RP2350) Automated Firmware Flasher
//!
//! Provides zero-friction USB flashing for DWire32A probe controllers:
//! 1. Detection of BOOTSEL USB Mass Storage drives (`RPI-RP2` / `RP2350` via `INFO_UF2.TXT`).
//! 2. Automatic 1200-baud touch reset to transition running Pico CDC devices into BOOTSEL mode.
//! 3. Binary UF2 validation, integrity hashing, and drag-and-drop file streaming.

use anyhow::{Context, Result, bail};
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
/// Microsoft UF2 Spec constants:
pub const RP2040_FAMILY_ID: u32 = 0xE484_9208;
pub const RP2350_ARM_S_FAMILY_ID: u32 = 0xE484_920A;
pub const RP2350_RISCV_FAMILY_ID: u32 = 0xE484_920B;

/// Official Raspberry Pi Pico SDK (elf2uf2 / picotool) Family IDs:
pub const RP2040_SDK_FAMILY_ID: u32 = 0xE48B_FF56;
pub const RP2350_ABS_SDK_FAMILY_ID: u32 = 0xE48B_FF57;
pub const RP2350_SDK_FAMILY_ID: u32 = RP2350_ABS_SDK_FAMILY_ID;
pub const RP2350_ARM_S_SDK_FAMILY_ID: u32 = 0xE48B_FF59;
pub const RP2350_RISCV_SDK_FAMILY_ID: u32 = 0xE48B_FF5A;

/// Pre-compiled DWire32A firmware binary containers embedded into the application executable.
pub const EMBEDDED_RP2040_UF2: &[u8] =
    include_bytes!("firmware_bins/panasonic_dwire32a_rp2040.uf2");
pub const EMBEDDED_RP2350_UF2: &[u8] =
    include_bytes!("firmware_bins/panasonic_dwire32a_rp2350.uf2");
pub const EMBEDDED_UNIFIED_UF2: &[u8] =
    include_bytes!("firmware_bins/panasonic_dwire32a_unified.uf2");

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

/// Validates that every 512-byte block in a buffer conforms to the UF2 specification.
pub fn validate_uf2_buffer(data: &[u8]) -> Result<(usize, String)> {
    if data.len() < UF2_BLOCK_SIZE || (data.len() % UF2_BLOCK_SIZE != 0) {
        bail!(
            "Invalid UF2 payload length: {} bytes (must be a positive multiple of 512 bytes)",
            data.len()
        );
    }

    let num_blocks = data.len() / UF2_BLOCK_SIZE;
    let mut has_rp2040 = false;
    let mut has_rp2350_arm = false;
    let mut has_rp2350_riscv = false;
    let mut has_rp2350_abs = false;
    let mut first_other_family = None;

    for (blk_idx, blk) in data.chunks_exact(UF2_BLOCK_SIZE).enumerate() {
        let start0 = u32::from_le_bytes(blk[0..4].try_into().unwrap());
        let start1 = u32::from_le_bytes(blk[4..8].try_into().unwrap());
        let end_magic = u32::from_le_bytes(blk[508..512].try_into().unwrap());

        if start0 != UF2_MAGIC_START0 || start1 != UF2_MAGIC_START1 || end_magic != UF2_MAGIC_END {
            bail!(
                "Corrupt UF2 block {} magic (expected 0x{:08X}/0x{:08X}/0x{:08X}, got 0x{:08X}/0x{:08X}/0x{:08X})",
                blk_idx,
                UF2_MAGIC_START0,
                UF2_MAGIC_START1,
                UF2_MAGIC_END,
                start0,
                start1,
                end_magic
            );
        }

        let family_id = u32::from_le_bytes(blk[28..32].try_into().unwrap());
        match family_id {
            RP2040_FAMILY_ID | RP2040_SDK_FAMILY_ID => has_rp2040 = true,
            RP2350_ABS_SDK_FAMILY_ID => has_rp2350_abs = true,
            RP2350_ARM_S_FAMILY_ID | RP2350_ARM_S_SDK_FAMILY_ID => has_rp2350_arm = true,
            RP2350_RISCV_FAMILY_ID | RP2350_RISCV_SDK_FAMILY_ID => has_rp2350_riscv = true,
            other => {
                if first_other_family.is_none() {
                    first_other_family = Some(other);
                }
            }
        }
    }

    let family_name = if has_rp2040 && (has_rp2350_arm || has_rp2350_riscv || has_rp2350_abs) {
        "Universal RP2040 + RP2350 (Unified Multi-Family UF2)".to_string()
    } else if has_rp2350_riscv {
        "Raspberry Pi Pico 2 RISC-V (RP2350)".to_string()
    } else if has_rp2350_arm || has_rp2350_abs {
        "Raspberry Pi Pico 2 ARM-S (RP2350)".to_string()
    } else if has_rp2040 {
        "Raspberry Pi RP2040".to_string()
    } else {
        format!(
            "Pico-Compatible Controller (Family ID 0x{:08X})",
            first_other_family.unwrap_or(0)
        )
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

/// Triggers an automatic reboot into BOOTSEL mode.
///
/// Strategy (two-phase for DWire32A compatibility):
///
/// **Phase 1 – Firmware-aware reset (DWire32A preferred path):**
/// Open the port at 115200 with DTR=true (required by `tud_cdc_connected()` in Pico SDK stdio_usb),
/// then immediately assert DTR=false and close.  The DWire32A firmware watches for a rising-then-falling
/// DTR edge and executes `reset_usb_boot(0, 0)` to enter BOOTSEL.
///
/// **Phase 2 – Classic 1200-baud touch (stock Pico SDK / Arduino fallback):**
/// If Phase 1 did not produce a BOOTSEL drive within ~1 s, open at 1200 baud with DTR=false.
/// The Pico SDK USB stack treats any 1200-baud open+close as a signal to call `reset_usb_boot`.
/// On Windows the CDC driver sometimes rejects 1200 baud on devices that report only 115200 in their
/// USB descriptor; a 300 ms retry handles that race.
///
/// The RP2040 typically appears as `RPI-RP2` mass storage within 600–800 ms of the reset.
pub fn trigger_1200_baud_bootsel(port_name: &str) -> Result<()> {
    // Extract the raw port name (e.g. "COM10") from descriptive UI labels like "COM10 [Raspberry Pi Pico...]"
    let clean = port_name
        .trim()
        .split_whitespace()
        .next()
        .unwrap_or(port_name);

    // ── Phase 1: Direct CDC BOOTSEL/RESET command probe (fastest path if supported) ──
    {
        if let Ok(mut port) = serialport::new(clean, 115_200)
            .timeout(Duration::from_millis(500))
            .open()
        {
            let _ = port.write_data_terminal_ready(true);
            let _ = port.write_request_to_send(true);
            thread::sleep(Duration::from_millis(60));
            // Send newline and standard reboot/bootsel commands
            let _ = port.write_all(b"\r\nBOOTSEL\r\nRESET\r\n");
            let _ = port.flush();
            thread::sleep(Duration::from_millis(100));
            drop(port);
            thread::sleep(Duration::from_millis(1_200));
            if !detect_bootsel_drives().is_empty() {
                return Ok(());
            }
        }
    }

    // ── Phase 2: Classic 1200-baud touch (official Pico SDK / Arduino standard) ────
    // On Windows with the Pico SDK, opening at 1200 baud causes usbser.sys to issue
    // SET_LINE_CODING with bit_rate = 1200. The RP2040 stdio_usb stack intercepts this
    // in tud_cdc_line_coding_cb and IMMEDIATELY invokes reset_usb_boot(0, 0).
    // This causes the device to disconnect from the USB bus instantaneously,
    // which makes Windows open() fail with ERROR_FILE_NOT_FOUND or device-not-present.
    // Therefore, we treat any error during 1200-baud open as expected, pause for USB
    // re-enumeration, and check if the RPI-RP2 drive has mounted!
    let mut last_err: Option<String> = None;
    for attempt in 0u8..2 {
        if attempt > 0 {
            thread::sleep(Duration::from_millis(300));
        }

        match serialport::new(clean, 1_200)
            .timeout(Duration::from_millis(500))
            .open()
        {
            Ok(mut port) => {
                let _ = port.write_data_terminal_ready(false);
                let _ = port.write_request_to_send(false);
                thread::sleep(Duration::from_millis(100));
                drop(port);
            }
            Err(e) => {
                last_err = Some(format!("attempt {}/2: {} — {}", attempt + 1, clean, e));
            }
        }

        // Allow USB bus re-enumeration as BOOTSEL Mass Storage device (RPI-RP2 / RP2350).
        thread::sleep(Duration::from_millis(1_500));
        if !detect_bootsel_drives().is_empty() {
            return Ok(());
        }
    }

    bail!(
        "Failed to initiate 1200-baud BOOTSEL touch after 2 attempts on port {} ({}). \
         If the drive is not visible, hold the BOOTSEL button while plugging the USB cable.",
        clean,
        last_err.unwrap_or_default()
    )
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

/// Automatically flashes the embedded DWire32A firmware to any connected BOOTSEL drive.
/// Inspects INFO_UF2.TXT to auto-detect whether the target chip is RP2040 or RP2350.
pub fn flash_embedded_firmware(target_override: Option<&str>) -> Result<PicoFlashReport> {
    let drives = detect_bootsel_drives();
    if drives.is_empty() {
        bail!(
            "No Raspberry Pi BOOTSEL drives (RPI-RP2 / RP2350) detected! Hold BOOTSEL button while plugging USB."
        );
    }
    let target_drive = &drives[0];

    // Auto-detect RP2040 vs RP2350 from INFO_UF2.TXT, or use unified UF2 container
    let info_txt_path = target_drive.join("INFO_UF2.TXT");
    let info_content = fs::read_to_string(&info_txt_path).unwrap_or_default();

    let uf2_payload = if let Some(over) = target_override {
        let lower = over.to_lowercase();
        if lower.contains("unified") || lower.contains("universal") {
            EMBEDDED_UNIFIED_UF2
        } else if lower.contains("2350") {
            EMBEDDED_RP2350_UF2
        } else if lower.contains("2040") {
            EMBEDDED_RP2040_UF2
        } else {
            EMBEDDED_UNIFIED_UF2
        }
    } else if info_content.contains("RP2350") || info_content.contains("Pico 2") {
        EMBEDDED_RP2350_UF2
    } else if info_content.contains("RP2040") || info_content.contains("RPI-RP2") {
        EMBEDDED_RP2040_UF2
    } else {
        EMBEDDED_UNIFIED_UF2
    };

    flash_uf2_to_drive(target_drive, uf2_payload)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uf2_buffer_validation_valid() {
        // Construct a synthetic 512-byte valid UF2 block for RP2040 (Microsoft Spec)
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

        // Test Official Pico SDK Family IDs (RP2040 & RP2350)
        block[28..32].copy_from_slice(&RP2040_SDK_FAMILY_ID.to_le_bytes());
        let res_sdk_2040 = validate_uf2_buffer(&block).unwrap();
        assert!(res_sdk_2040.1.contains("RP2040"));

        block[28..32].copy_from_slice(&RP2350_SDK_FAMILY_ID.to_le_bytes());
        let res_sdk_2350 = validate_uf2_buffer(&block).unwrap();
        assert!(res_sdk_2350.1.contains("RP2350"));
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

    #[test]
    fn test_embedded_uf2_containers_and_unified_layout() {
        let (b2040, f2040) = validate_uf2_buffer(EMBEDDED_RP2040_UF2).expect("RP2040 UF2 valid");
        assert!(b2040 > 0);
        assert!(f2040.contains("RP2040"));

        let (b2350, f2350) = validate_uf2_buffer(EMBEDDED_RP2350_UF2).expect("RP2350 UF2 valid");
        assert!(b2350 > 0);
        assert!(f2350.contains("RP2350"));

        let (bunified, funified) =
            validate_uf2_buffer(EMBEDDED_UNIFIED_UF2).expect("Unified UF2 valid");
        assert_eq!(bunified, b2040 + b2350);
        assert!(funified.contains("Unified Multi-Family"));

        // Block 0 must carry the RP2350-E10 absolute workaround family ID
        let blk0_family = u32::from_le_bytes(EMBEDDED_UNIFIED_UF2[28..32].try_into().unwrap());
        assert_eq!(blk0_family, RP2350_ABS_SDK_FAMILY_ID);

        // Final two blocks must finish both RP2040 and RP2350 images at EOF
        let tail_start = (bunified - 2) * UF2_BLOCK_SIZE;
        let second_last_fid = u32::from_le_bytes(
            EMBEDDED_UNIFIED_UF2[tail_start + 28..tail_start + 32]
                .try_into()
                .unwrap(),
        );
        let last_fid = u32::from_le_bytes(
            EMBEDDED_UNIFIED_UF2
                [tail_start + UF2_BLOCK_SIZE + 28..tail_start + UF2_BLOCK_SIZE + 32]
                .try_into()
                .unwrap(),
        );
        assert_eq!(second_last_fid, RP2040_SDK_FAMILY_ID);
        assert_eq!(last_fid, RP2350_ARM_S_SDK_FAMILY_ID);
    }
}
