//! Panasonic DMR-EH55 DIGA Suite - Storage & NVRAM Forensics IPC Commands
//!
//! Single Responsibility: Tauri IPC command dispatchers for Spansion S29JL064H
//! NVRAM sanitization and HDD/MEIHDFS inspection, decoding, and partition extraction.

use std::path::Path;

use super::dto::{
    ExtractedPartitionItemDto, ExtractedPartitionReport, HddAnalysisReport,
    MeihdfsAllocationReport, NvramCleanReport, PartitionExtractionManifestDto, resolve_candidate,
    resolve_default_flash_path,
};

/// Surgically cleans and audits the Spansion S29JL064H flash dump NVRAM emulation partitions.
#[tauri::command]
pub fn clean_nvram_command(
    input_path: String,
    output_path: Option<String>,
) -> Result<NvramCleanReport, String> {
    let in_p = Path::new(&input_path);
    let out_p = output_path.as_deref().map(Path::new);
    panadiga_core::nvram::clean_flash_nvram(in_p, out_p).map_err(|e| e.to_string())
}

/// Inspects HDD image layout, MBR tables, partition metadata, and allocation density.
#[tauri::command]
pub fn inspect_hdd_command(
    image_path: String,
    show_density: bool,
) -> Result<HddAnalysisReport, String> {
    panadiga_core::hdd::inspect_hdd_image(Path::new(&image_path), show_density, None)
        .map_err(|e| e.to_string())
}

/// Decodes bit-inverted MEIHDFS filesystem allocation descriptors and journal blocks.
#[tauri::command]
pub fn decode_meihdfs_command(
    image_path: String,
    include_journal: bool,
) -> Result<MeihdfsAllocationReport, String> {
    panadiga_core::hdd::decode_meihdfs_descriptors(Path::new(&image_path), include_journal, None)
        .map_err(|e| e.to_string())
}

/// Surgically extracts firmware partitions, MBR structures, and cryptographic manifest.
#[tauri::command]
pub fn extract_hdd_partitions_command(
    image_path: String,
    output_dir: Option<String>,
) -> Result<Vec<ExtractedPartitionReport>, String> {
    let out_dir =
        output_dir.unwrap_or_else(|| panadiga_core::constants::DEFAULT_HDD_EXTRACT_DIR.to_string());
    panadiga_core::hdd::extract_hdd_partitions(Path::new(&image_path), Path::new(&out_dir), None)
        .map_err(|e| e.to_string())
}

/// Frontend `analyze_flash_dump` command (non-destructive inspection).
#[tauri::command]
pub fn analyze_flash_dump(input_path: Option<String>) -> Result<NvramCleanReport, String> {
    let path = resolve_default_flash_path(input_path);
    panadiga_core::nvram::inspect_flash_nvram(Path::new(&path)).map_err(|e| e.to_string())
}

/// Frontend `clean_nvram` command (surgical 64 KB NVRAM reset to 0xFF).
#[tauri::command]
pub fn clean_nvram(
    input_path: Option<String>,
    output_path: Option<String>,
) -> Result<NvramCleanReport, String> {
    let path = resolve_default_flash_path(input_path);
    let in_p = Path::new(&path);
    let out = output_path.unwrap_or_else(|| {
        let parent = in_p.parent().unwrap_or_else(|| Path::new("."));
        let stem = in_p
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("flash_dump");
        parent
            .join(format!("{stem}_CLEAN.bin"))
            .to_string_lossy()
            .to_string()
    });
    clean_nvram_command(path, Some(out))
}

/// Frontend `inspect_hdd` command.
#[tauri::command]
pub fn inspect_hdd(
    image_path: Option<String>,
    show_density: Option<bool>,
) -> Result<HddAnalysisReport, String> {
    let path = image_path.unwrap_or_else(|| {
        resolve_candidate(
            &[
                panadiga_core::constants::DEFAULT_HDD_IMAGE_PATH,
                "../../firmware_and_images/panasonic DMR EH-55 Firmware hdd_100.bin",
            ],
            panadiga_core::constants::DEFAULT_HDD_IMAGE_PATH,
        )
    });
    inspect_hdd_command(path, show_density.unwrap_or(true))
}

/// Frontend `decode_meihdfs_allocations` command.
#[tauri::command]
pub fn decode_meihdfs_allocations(
    image_path: Option<String>,
    include_journal: Option<bool>,
) -> Result<MeihdfsAllocationReport, String> {
    let path = image_path.unwrap_or_else(|| {
        resolve_candidate(
            &[
                panadiga_core::constants::DEFAULT_HDD_IMAGE_PATH,
                "../../firmware_and_images/panasonic DMR EH-55 Firmware hdd_100.bin",
            ],
            panadiga_core::constants::DEFAULT_HDD_IMAGE_PATH,
        )
    });
    decode_meihdfs_command(path, include_journal.unwrap_or(true))
}

/// Frontend `extract_partitions` command returning `PartitionExtractionManifestDto`.
#[tauri::command]
pub fn extract_partitions(
    image_path: Option<String>,
    output_dir: Option<String>,
) -> Result<PartitionExtractionManifestDto, String> {
    let img = image_path.unwrap_or_else(|| {
        resolve_candidate(
            &[
                panadiga_core::constants::DEFAULT_HDD_IMAGE_PATH,
                "../../firmware_and_images/panasonic DMR EH-55 Firmware hdd_100.bin",
            ],
            panadiga_core::constants::DEFAULT_HDD_IMAGE_PATH,
        )
    });
    let out_dir =
        output_dir.unwrap_or_else(|| panadiga_core::constants::DEFAULT_HDD_EXTRACT_DIR.to_string());
    let reports = extract_hdd_partitions_command(img.clone(), Some(out_dir.clone()))?;

    let mut sums = String::new();
    let mut items = Vec::with_capacity(reports.len());
    for (idx, r) in reports.iter().enumerate() {
        sums.push_str(&format!("{} *{}\n", r.sha256, r.filename));
        let offset = panadiga_core::hdd::PARTITION_LAYOUT_SPECS
            .get(idx)
            .map(|s| s.offset)
            .unwrap_or(0);
        let bytes_read = std::fs::read(&r.path).unwrap_or_default();
        let (md5_hex, _) = panadiga_core::nvram::calculate_hashes(&bytes_read);
        let entropy = panadiga_core::hdd::utils::calculate_entropy(&bytes_read);

        items.push(ExtractedPartitionItemDto {
            id: idx + 1,
            name: r.name.clone(),
            filename: r.filename.clone(),
            offset_hex: format!("0x{:08X}", offset),
            size_bytes: r.size_bytes,
            sha256: r.sha256.clone(),
            md5: md5_hex,
            entropy: (entropy * 100.0).round() / 100.0,
        });
    }

    Ok(PartitionExtractionManifestDto {
        timestamp: format_utc_iso8601_now(),
        source_image: img,
        output_directory: out_dir,
        manifest_sha256_sums: sums.trim_end().to_string(),
        extracted_partitions: items,
    })
}

/// Discovers connected physical storage drives and evaluates system safety locks.
#[tauri::command]
pub fn list_physical_drives() -> Result<Vec<panadiga_core::hdd::PhysicalDriveInfo>, String> {
    panadiga_core::hdd::list_physical_drives().map_err(|e| e.to_string())
}

/// Formats a physical disk with procedural MEIHDFS filesystem structures (mkfs.meihdfs).
#[tauri::command]
pub fn format_hdd_meihdfs(
    device_path: String,
    total_sectors: u64,
    confirmation_token: String,
) -> Result<panadiga_core::hdd::DriveWriteReport, String> {
    let layout = panadiga_core::hdd::mkfs::generate_meihdfs_layout(total_sectors, None)
        .map_err(|e| e.to_string())?;
    panadiga_core::hdd::write_meihdfs_procedural_to_drive(
        &device_path,
        &layout,
        &confirmation_token,
    )
    .map_err(|e| e.to_string())
}

/// Safely writes an existing raw image file to a verified physical drive.
#[tauri::command]
pub fn write_hdd_image(
    device_path: String,
    image_path: Option<String>,
    confirmation_token: String,
) -> Result<panadiga_core::hdd::DriveWriteReport, String> {
    let img_path = image_path.unwrap_or_else(|| {
        resolve_candidate(
            &[
                panadiga_core::constants::DEFAULT_HDD_IMAGE_PATH,
                "../../firmware_and_images/panasonic DMR EH-55 Firmware hdd_100.bin",
            ],
            panadiga_core::constants::DEFAULT_HDD_IMAGE_PATH,
        )
    });
    let bytes = std::fs::read(&img_path)
        .map_err(|e| format!("Failed reading image file {}: {}", img_path, e))?;
    panadiga_core::hdd::write_raw_image_to_drive(&device_path, &bytes, &confirmation_token)
        .map_err(|e| e.to_string())
}

/// Atomically restores primary MEIHDFS structures from dual redundancy mirrors.
#[tauri::command]
pub fn restore_hdd_mirrors(
    device_path: String,
    confirmation_token: String,
) -> Result<panadiga_core::hdd::DriveWriteReport, String> {
    panadiga_core::hdd::restore_meihdfs_from_mirrors(&device_path, &confirmation_token)
        .map_err(|e| e.to_string())
}

/// Audits a live NVRAM byte slice captured in-circuit via Pico (P7501 UART) / Android USB-OTG / BLE NUS.
#[tauri::command]
pub fn analyze_live_nvram_bytes(
    bytes: Vec<u8>,
    base_bus_address: Option<u32>,
    source_label: Option<String>,
) -> Result<panadiga_core::nvram::NvramCleanReport, String> {
    let base_addr = base_bus_address.unwrap_or(panadiga_core::constants::AM34_NVRAM_BASE);
    let label = source_label.unwrap_or_else(|| "Pico Live NVRAM Capture".to_string());
    panadiga_core::nvram::inspect_partial_nvram_capture(&bytes, base_addr, &label)
        .map_err(|e| e.to_string())
}

fn format_utc_iso8601_now() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    let days = secs.div_euclid(86_400);
    let rem = secs.rem_euclid(86_400);
    let hour = rem / 3_600;
    let minute = (rem % 3_600) / 60;
    let second = rem % 60;
    // Howard Hinnant civil-from-days conversion (proleptic Gregorian UTC)
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = y + if m <= 2 { 1 } else { 0 };
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        year, m, d, hour, minute, second
    )
}
