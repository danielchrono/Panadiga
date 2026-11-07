//! Firmware and Storage Header Inspection & Hex Dumper
//!
//! Provides hex and ASCII dumps of critical firmware partition boundaries,
//! boot sectors, and optical update container headers.

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::constants::*;
use crate::nvram::resolve_input_path;

/// Generates a standard formatted hex and ASCII representation of binary data.
pub fn hex_dump(data: &[u8], start_offset: usize, length: usize) -> String {
    let limit = length.min(data.len());
    let mut lines = Vec::new();

    for i in (0..limit).step_by(16) {
        let chunk_end = (i + 16).min(limit);
        let chunk = &data[i..chunk_end];

        let mut hex_parts = Vec::with_capacity(16);
        for b in chunk {
            hex_parts.push(format!("{:02X}", b));
        }
        let hex_str = hex_parts.join(" ");

        let mut ascii_str = String::with_capacity(chunk.len());
        for &b in chunk {
            if (32..127).contains(&b) {
                ascii_str.push(b as char);
            } else {
                ascii_str.push('.');
            }
        }

        lines.push(format!(
            "{:08X}  {:<48}  |{}|",
            start_offset + i,
            hex_str,
            ascii_str
        ));
    }

    lines.join("\n")
}

/// Dumps a slice of any file at a given byte offset and length.
pub fn dump_file_hex(file_path: &Path, offset: usize, length: usize) -> Result<()> {
    let resolved = resolve_input_path(file_path)?;
    let mut f = File::open(&resolved)
        .with_context(|| format!("Failed to open file: {}", resolved.display()))?;

    let file_len = f.metadata()?.len() as usize;
    if offset >= file_len {
        anyhow::bail!(
            "Offset 0x{:08X} is beyond file length (0x{:08X} bytes)",
            offset,
            file_len
        );
    }

    f.seek(SeekFrom::Start(offset as u64))?;
    let to_read = length.min(file_len - offset);
    let mut buf = vec![0u8; to_read];
    f.read_exact(&mut buf)?;

    log_println!("=== Hex Dump of {} ===", resolved.display());
    log_println!("Offset: 0x{:08X} ({} bytes)", offset, to_read);
    log_println!("{}", hex_dump(&buf, offset, to_read));

    Ok(())
}

/// Inspects the key architectural firmware headers in HDD image and PANA_DVD.FRM.
pub fn inspect_firmware_headers(
    hdd_path: Option<&Path>,
    frm_path: Option<&Path>,
    length: usize,
) -> Result<()> {
    let default_hdd = PathBuf::from("firmware_and_images/panasonic DMR EH-55 Firmware hdd_100.bin");
    let hdd_file = hdd_path.unwrap_or(&default_hdd);

    let default_frm = PathBuf::from("firmware_and_images/PANA_DVD.FRM");
    let frm_file = frm_path.unwrap_or(&default_frm);

    log_println!("================================================================================");
    log_println!(" PANASONIC DMR-EH55 CRITICAL FIRMWARE HEADERS & CONTAINER INSPECTION");
    log_println!("================================================================================");

    // 1. HDD Inspection
    if let Ok(resolved_hdd) = resolve_input_path(hdd_file) {
        log_println!("[*] HDD Image: {}", resolved_hdd.display());
        let mut f = File::open(&resolved_hdd)?;

        // Partition 1 at 32MB (0x0200_0000)
        let mut hdr32 = vec![0u8; length];
        f.seek(SeekFrom::Start(LBA_PARTITION_1_OFFSET))?;
        f.read_exact(&mut hdr32)?;

        let p1_magic_ok = hdr32.starts_with(&PANASONIC_HDD_PARTITION_MAGIC);
        log_println!(
            "\n--- HDD Partition 1 Header (32 MB / LBA {} / 0x{:08X}) ---",
            LBA_PARTITION_1_SECTOR, LBA_PARTITION_1_OFFSET
        );
        log_println!(
            "Magic Signature: {} [{}]",
            hex::encode_upper(&hdr32[..hdr32.len().min(8)]),
            if p1_magic_ok {
                "VALID PANASONIC PARTITION"
            } else {
                "UNKNOWN"
            }
        );
        log_println!("{}", hex_dump(&hdr32, LBA_PARTITION_1_OFFSET as usize, length));

        // Partition 2 at 64MB (0x0400_0000)
        let mut hdr64 = vec![0u8; length];
        f.seek(SeekFrom::Start(LBA_PARTITION_2_OFFSET))?;
        f.read_exact(&mut hdr64)?;

        let p2_magic_ok = hdr64.starts_with(&PANASONIC_HDD_PARTITION_MAGIC);
        log_println!(
            "\n--- HDD Partition 2 Header (64 MB / LBA {} / 0x{:08X}) ---",
            LBA_PARTITION_2_SECTOR, LBA_PARTITION_2_OFFSET
        );
        log_println!(
            "Magic Signature: {} [{}]",
            hex::encode_upper(&hdr64[..hdr64.len().min(8)]),
            if p2_magic_ok {
                "VALID PANASONIC PARTITION"
            } else {
                "UNKNOWN"
            }
        );
        log_println!("{}", hex_dump(&hdr64, LBA_PARTITION_2_OFFSET as usize, length));
    } else {
        log_println!("[!] HDD image not found at {}", hdd_file.display());
    }

    // 2. Optical Container PANA_DVD.FRM
    if let Ok(resolved_frm) = resolve_input_path(frm_file) {
        log_println!("\n[*] Optical Update Container: {}", resolved_frm.display());
        let mut f = File::open(&resolved_frm)?;
        let mut frm_hdr = vec![0u8; length];
        f.read_exact(&mut frm_hdr)?;

        log_println!("\n--- PANA_DVD.FRM Container Header (first {} bytes) ---", length);
        log_println!("{}", hex_dump(&frm_hdr, 0, length));
    } else {
        log_println!("[!] PANA_DVD.FRM not found at {}", frm_file.display());
    }

    log_println!("\n================================================================================");
    Ok(())
}
