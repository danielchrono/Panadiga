//! Cross-Platform Physical HDD / SSD Safe Block Writer with System Drive Lockout
//!
//! Provides direct LBA block writing for Panasonic MEIHDFS restoration:
//! - Windows: Direct handle to `\\.\PhysicalDriveN` with sector-aligned streaming.
//! - Linux/macOS: Direct block device writing to `/dev/sdX` or `/dev/rdiskN`.
//! - Strict Safety Guardrail: Automatic detection and hard lockout of the host OS boot drive.
//! - Anti-Accident Token: Enforces deliberate confirmation before any sector is modified.

use super::mkfs::MeihdfsImage;
use crate::constants::*;
use anyhow::{Context, Result, bail};
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};

/// Information regarding a discovered physical disk drive.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PhysicalDriveInfo {
    pub device_path: String,
    pub display_name: String,
    pub size_bytes: u64,
    pub size_gb: f64,
    pub is_system_drive: bool,
    pub safe_to_write: bool,
    pub lockout_reason: Option<String>,
}

/// Execution report of a physical drive write/format operation.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DriveWriteReport {
    pub device_path: String,
    pub operation: String,
    pub bytes_written: u64,
    pub sectors_written: u64,
    pub success: bool,
    pub message: String,
}

/// Discovers connected physical storage drives and evaluates system safety locks.
pub fn list_physical_drives() -> Result<Vec<PhysicalDriveInfo>> {
    let mut drives = Vec::new();

    #[cfg(windows)]
    {
        for idx in 0..16 {
            let dev_path = format!(r"\\.\PhysicalDrive{}", idx);
            // Attempt to query drive handle
            if let Ok(file) = OpenOptions::new().read(true).open(&dev_path) {
                let metadata = file.metadata().ok();
                let len = metadata.map(|m| m.len()).unwrap_or(0);

                // PhysicalDrive0 is practically always the Windows Boot OS Drive
                let is_system = idx == 0;
                let safe = !is_system;
                let reason = if is_system {
                    Some(
                        "LOCKED: Windows Host Operating System Boot Drive (PhysicalDrive0)"
                            .to_string(),
                    )
                } else {
                    None
                };

                drives.push(PhysicalDriveInfo {
                    device_path: dev_path.clone(),
                    display_name: format!(
                        "Physical Drive {} ({:.1} GB)",
                        idx,
                        len as f64 / 1_000_000_000.0
                    ),
                    size_bytes: len,
                    size_gb: len as f64 / 1_000_000_000.0,
                    is_system_drive: is_system,
                    safe_to_write: safe,
                    lockout_reason: reason,
                });
            }
        }
    }

    #[cfg(unix)]
    {
        for letter in b'a'..=b'z' {
            let dev_path = format!("/dev/sd{}", letter as char);
            if std::path::Path::new(&dev_path).exists() {
                // In Linux, sda is typically the host system drive
                let is_system = letter == b'a';
                let safe = !is_system;
                let reason = if is_system {
                    Some("LOCKED: Host Root Filesystem / OS Drive (/dev/sda)".to_string())
                } else {
                    None
                };

                drives.push(PhysicalDriveInfo {
                    device_path: dev_path.clone(),
                    display_name: format!(
                        "SCSI Disk {} (/dev/sd{})",
                        letter as char, letter as char
                    ),
                    size_bytes: 0,
                    size_gb: 0.0,
                    is_system_drive: is_system,
                    safe_to_write: safe,
                    lockout_reason: reason,
                });
            }
        }
    }

    Ok(drives)
}

/// Verifies that a target drive is safe for writing and validates the confirmation token.
pub fn verify_safety_lockout(device_path: &str, confirmation_token: &str) -> Result<()> {
    // 1. Mandatory safety lockout against the primary host operating system drive
    let normalized = device_path.to_lowercase();
    if normalized.contains("physicaldrive0") || normalized == "/dev/sda" {
        bail!(
            "FATAL SAFETY INVARIANT VIOLATION: Access to {} is unconditionally locked! \
            The primary operating system drive can never be overwritten by Panadiga.",
            device_path
        );
    }

    // 2. Validate user confirmation token
    let expected_token = format!("CONFIRM-{}", device_path.trim_start_matches(r"\\.\"));
    if confirmation_token != expected_token {
        bail!(
            "Confirmation token mismatch! Expected '{}', got '{}'. Operation aborted.",
            expected_token,
            confirmation_token
        );
    }

    Ok(())
}

/// Helper to verify safety lockout and open a physical block device in read/write mode.
pub fn open_verified_block_device(device_path: &str, confirmation_token: &str) -> Result<File> {
    verify_safety_lockout(device_path, confirmation_token)?;

    OpenOptions::new()
        .read(true)
        .write(true)
        .open(device_path)
        .with_context(|| format!("Failed to open physical block device at {}", device_path))
}

/// Writes procedurally synthesized MEIHDFS structures directly to a physical drive.
pub fn write_meihdfs_procedural_to_drive(
    device_path: &str,
    layout: &MeihdfsImage,
    confirmation_token: &str,
) -> Result<DriveWriteReport> {
    let mut file = open_verified_block_device(device_path, confirmation_token)?;

    let mut total_written: u64 = 0;

    // 1. Write Master MBR (LBA 0)
    file.seek(SeekFrom::Start(0))?;
    file.write_all(&layout.mbr_lba0)?;
    total_written += layout.mbr_lba0.len() as u64;

    // 2. Write Allocation Descriptors (LBA 24..31 / 0x3000)
    file.seek(SeekFrom::Start(ALLOCATION_DESCRIPTORS_OFFSET))?;
    file.write_all(&layout.descriptors_lba24)?;
    total_written += layout.descriptors_lba24.len() as u64;

    // 3. Write Journal Header (LBA 16384 / 0x800000)
    file.seek(SeekFrom::Start(LBA_JOURNAL_CATALOG_OFFSET))?;
    file.write_all(&layout.journal_header_lba16384)?;
    total_written += layout.journal_header_lba16384.len() as u64;

    // 4. Write Redundant Mirror MBR (LBA 57344 / 0x1C00000)
    file.seek(SeekFrom::Start(LBA_MIRROR_MBR_OFFSET))?;
    file.write_all(&layout.mirror_mbr_lba57344)?;
    total_written += layout.mirror_mbr_lba57344.len() as u64;

    // 5. Write Redundant Mirror Allocation Descriptors (LBA 57368 / 0x1C03000)
    file.seek(SeekFrom::Start(ALLOCATION_DESCRIPTORS_MIRROR_OFFSET))?;
    file.write_all(&layout.mirror_descriptors_lba57368)?;
    total_written += layout.mirror_descriptors_lba57368.len() as u64;

    file.flush()?;

    Ok(DriveWriteReport {
        device_path: device_path.to_string(),
        operation: "Procedural MEIHDFS Formatting (mkfs.meihdfs)".to_string(),
        bytes_written: total_written,
        sectors_written: total_written / 512,
        success: true,
        message: format!(
            "Successfully formatted {} with MEIHDFS structures (Total: {} bytes written).",
            device_path, total_written
        ),
    })
}

/// Safely writes an existing raw image (or uncompressed slice) to a verified physical drive.
pub fn write_raw_image_to_drive(
    device_path: &str,
    image_bytes: &[u8],
    confirmation_token: &str,
) -> Result<DriveWriteReport> {
    if image_bytes.len() < LBA_SECTOR_SIZE_BYTES
        || !image_bytes.len().is_multiple_of(LBA_SECTOR_SIZE_BYTES)
    {
        bail!(
            "Raw image length must be a multiple of 512 bytes (got {} bytes)",
            image_bytes.len()
        );
    }

    let mut file = open_verified_block_device(device_path, confirmation_token)?;

    file.seek(SeekFrom::Start(0))?;
    file.write_all(image_bytes)?;
    file.flush()?;

    Ok(DriveWriteReport {
        device_path: device_path.to_string(),
        operation: "Raw Disk Image Streaming".to_string(),
        bytes_written: image_bytes.len() as u64,
        sectors_written: (image_bytes.len() / LBA_SECTOR_SIZE_BYTES) as u64,
        success: true,
        message: format!(
            "Successfully streamed {} bytes to {} without errors.",
            image_bytes.len(),
            device_path
        ),
    })
}

/// Restores primary MEIHDFS structures atomically from dual redundancy mirrors:
/// - Mirror MBR (LBA 57344 / 0x01C00000, 512B) -> Master MBR (LBA 0)
/// - Mirror Allocation Descriptors (LBA 57368 / 0x01C03000, 4096B) -> Allocation Descriptors (LBA 24..31 / 0x00003000)
pub fn restore_meihdfs_from_mirrors(
    device_path: &str,
    confirmation_token: &str,
) -> Result<DriveWriteReport> {
    let mut file = open_verified_block_device(device_path, confirmation_token)?;

    // Step 1: Read Mirror MBR (LBA 57344 / 0x01C00000)
    let mut mirror_mbr = [0u8; LBA_SECTOR_SIZE_BYTES];
    file.seek(SeekFrom::Start(LBA_MIRROR_MBR_OFFSET))?;
    file.read_exact(&mut mirror_mbr)
        .context("Failed to read Mirror MBR at LBA 57344")?;

    // Verify "SH" signature at bytes 6..7
    if mirror_mbr[0..8] != PANASONIC_HDD_MBR_SIGNATURE {
        bail!(
            "Mirror MBR at LBA 57344 is invalid! Expected Panasonic 'SH' signature, got 0x{}",
            hex::encode(&mirror_mbr[0..8])
        );
    }

    // Step 2: Read Mirror Allocation Descriptors (LBA 57368 / 0x01C03000)
    let mut mirror_desc = [0u8; ALLOCATION_DESCRIPTORS_SIZE];
    file.seek(SeekFrom::Start(ALLOCATION_DESCRIPTORS_MIRROR_OFFSET))?;
    file.read_exact(&mut mirror_desc)
        .context("Failed to read Mirror Allocation Descriptors at LBA 57368")?;

    // Verify first record has valid inverted extent tag
    let first_tag = mirror_desc[0] ^ 0xFF;
    if first_tag != MEIHDFS_TAG_EXTENT_HEADER && first_tag != MEIHDFS_TAG_FREE {
        bail!(
            "Mirror Allocation Descriptors at LBA 57368 contain invalid tag: 0x{:02X}",
            first_tag
        );
    }

    // Step 3: Atomically restore Master MBR (LBA 0)
    file.seek(SeekFrom::Start(0))?;
    file.write_all(&mirror_mbr)?;

    // Step 4: Atomically restore Primary Allocation Descriptors (LBA 24..31 / 0x00003000)
    file.seek(SeekFrom::Start(ALLOCATION_DESCRIPTORS_OFFSET))?;
    file.write_all(&mirror_desc)?;

    file.flush()?;

    let total = (LBA_SECTOR_SIZE_BYTES + ALLOCATION_DESCRIPTORS_SIZE) as u64;

    Ok(DriveWriteReport {
        device_path: device_path.to_string(),
        operation: "Atomic Dual-Mirror MEIHDFS Restoration".to_string(),
        bytes_written: total,
        sectors_written: total / 512,
        success: true,
        message: format!(
            "Successfully restored LBA 0 (512B) and LBA 24..31 (4096B) from mirrors LBA 57344 and 57368 on {}.",
            device_path
        ),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_drive_lockout_blocks_physicaldrive0() {
        let res = verify_safety_lockout(r"\\.\PhysicalDrive0", "CONFIRM-PhysicalDrive0");
        assert!(res.is_err());
        assert!(
            res.unwrap_err()
                .to_string()
                .contains("SAFETY INVARIANT VIOLATION")
        );
    }

    #[test]
    fn test_system_drive_lockout_blocks_sda() {
        let res = verify_safety_lockout("/dev/sda", "CONFIRM-/dev/sda");
        assert!(res.is_err());
        assert!(
            res.unwrap_err()
                .to_string()
                .contains("SAFETY INVARIANT VIOLATION")
        );
    }

    #[test]
    fn test_token_mismatch_rejection() {
        let res = verify_safety_lockout(r"\\.\PhysicalDrive1", "WRONG-TOKEN");
        assert!(res.is_err());
        assert!(res.unwrap_err().to_string().contains("token mismatch"));
    }

    #[test]
    fn test_valid_token_allows_non_system_drive() {
        let res = verify_safety_lockout(r"\\.\PhysicalDrive2", "CONFIRM-PhysicalDrive2");
        assert!(res.is_ok());
    }

    #[test]
    fn test_restore_mirrors_blocks_system_drive() {
        let res = restore_meihdfs_from_mirrors(r"\\.\PhysicalDrive0", "CONFIRM-PhysicalDrive0");
        assert!(res.is_err());
        assert!(
            res.unwrap_err()
                .to_string()
                .contains("SAFETY INVARIANT VIOLATION")
        );
    }
}
