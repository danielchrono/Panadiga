use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

use anyhow::{Context, Result};

use crate::constants::*;
use crate::nvram::{calculate_hashes, resolve_input_path};

use super::inspect::inspect_hdd_image;
use super::types::ExtractedPartitionReport;
use super::utils::open_image_buffer;

/// Surgical extraction of all firmware partitions and filesystem descriptors.
pub fn extract_hdd_partitions(
    image_path: &Path,
    output_dir: &Path,
    json_export_path: Option<&Path>,
) -> Result<Vec<ExtractedPartitionReport>> {
    let resolved_path = resolve_input_path(image_path)?;
    let mmap = open_image_buffer(&resolved_path)?;

    fs::create_dir_all(output_dir)
        .with_context(|| format!("Failed creating output directory: {}", output_dir.display()))?;

    log_println!(
        "================================================================================"
    );
    log_println!(
        " PANASONIC DMR-EH55 PARTITION & DESCRIPTOR SURGICAL EXTRACTOR                   "
    );
    log_println!(
        "================================================================================"
    );
    log_println!(" Imagem Fonte : {}", resolved_path.display());
    log_println!(" Destino      : {}", output_dir.display());
    log_println!(
        "--------------------------------------------------------------------------------"
    );

    let mut manifest_entries: Vec<(String, String)> = Vec::new();
    let mut extracted_reports: Vec<ExtractedPartitionReport> = Vec::new();

    // 1. Master MBR (LBA 0)
    let mbr_data = &mmap[..LBA_SECTOR_SIZE_BYTES];
    let mbr_file = output_dir.join("hdd_mbr_master_lba0.bin");
    fs::write(&mbr_file, mbr_data)?;
    let (_, mbr_sha256) = calculate_hashes(mbr_data);
    manifest_entries.push(("hdd_mbr_master_lba0.bin".to_string(), mbr_sha256.clone()));
    extracted_reports.push(ExtractedPartitionReport {
        name: "Master MBR (LBA 0)".to_string(),
        filename: "hdd_mbr_master_lba0.bin".to_string(),
        path: mbr_file.to_string_lossy().to_string(),
        size_bytes: mbr_data.len(),
        sha256: mbr_sha256,
    });
    log_println!("  [+] Extraido MBR Master  : hdd_mbr_master_lba0.bin (512 bytes)");

    // 2. Mirror MBR (LBA 57344)
    let mirror_start = LBA_MIRROR_MBR_OFFSET as usize;
    let mirror_data = &mmap[mirror_start..mirror_start + LBA_SECTOR_SIZE_BYTES];
    let mirror_file = output_dir.join("hdd_mbr_mirror_lba57344.bin");
    fs::write(&mirror_file, mirror_data)?;
    let (_, mirror_sha256) = calculate_hashes(mirror_data);
    manifest_entries.push((
        "hdd_mbr_mirror_lba57344.bin".to_string(),
        mirror_sha256.clone(),
    ));
    extracted_reports.push(ExtractedPartitionReport {
        name: "Mirror MBR (LBA 57344)".to_string(),
        filename: "hdd_mbr_mirror_lba57344.bin".to_string(),
        path: mirror_file.to_string_lossy().to_string(),
        size_bytes: mirror_data.len(),
        sha256: mirror_sha256,
    });
    log_println!("  [+] Extraido MBR Mirror  : hdd_mbr_mirror_lba57344.bin (512 bytes)");

    // 3. Allocation Descriptors (0x00003000)
    let alloc_start = ALLOCATION_DESCRIPTORS_OFFSET as usize;
    let alloc_data = &mmap[alloc_start..alloc_start + ALLOCATION_DESCRIPTORS_SIZE];
    let alloc_file = output_dir.join("hdd_allocation_descriptors_0x00003000.bin");
    fs::write(&alloc_file, alloc_data)?;
    let (_, alloc_sha256) = calculate_hashes(alloc_data);
    manifest_entries.push((
        "hdd_allocation_descriptors_0x00003000.bin".to_string(),
        alloc_sha256.clone(),
    ));
    extracted_reports.push(ExtractedPartitionReport {
        name: "MEIHDFS Allocation Descriptors (0x3000)".to_string(),
        filename: "hdd_allocation_descriptors_0x00003000.bin".to_string(),
        path: alloc_file.to_string_lossy().to_string(),
        size_bytes: alloc_data.len(),
        sha256: alloc_sha256,
    });
    log_println!(
        "  [+] Extraido Descritores : hdd_allocation_descriptors_0x00003000.bin (4,096 bytes)"
    );

    // 4. Journal Area (0x00820000 - 0x00960000)
    let j_start = JOURNAL_AREA_OFFSET as usize;
    let j_data = &mmap[j_start..j_start + JOURNAL_AREA_SIZE];
    let j_file = output_dir.join("hdd_journal_blocks_0x00820000.bin");
    fs::write(&j_file, j_data)?;
    let (_, j_sha256) = calculate_hashes(j_data);
    manifest_entries.push((
        "hdd_journal_blocks_0x00820000.bin".to_string(),
        j_sha256.clone(),
    ));
    extracted_reports.push(ExtractedPartitionReport {
        name: "Transactional Journal Area (0x820000)".to_string(),
        filename: "hdd_journal_blocks_0x00820000.bin".to_string(),
        path: j_file.to_string_lossy().to_string(),
        size_bytes: j_data.len(),
        sha256: j_sha256,
    });
    log_println!(
        "  [+] Extraido Journal     : hdd_journal_blocks_0x00820000.bin (1,310,720 bytes)"
    );

    // 5. Firmware Partition 1 - Exact Payload (0x5E3250)
    let p1_start = LBA_PARTITION_1_OFFSET as usize;
    let p1_data = &mmap[p1_start..p1_start + LBA_PARTITION_1_PAYLOAD_SIZE];
    let p1_file = output_dir.join("hdd_partition_firmware_0x02000000.bin");
    fs::write(&p1_file, p1_data)?;
    let (_, p1_sha256) = calculate_hashes(p1_data);
    manifest_entries.push((
        "hdd_partition_firmware_0x02000000.bin".to_string(),
        p1_sha256.clone(),
    ));
    extracted_reports.push(ExtractedPartitionReport {
        name: "Firmware Partition 1 (Exact Payload)".to_string(),
        filename: "hdd_partition_firmware_0x02000000.bin".to_string(),
        path: p1_file.to_string_lossy().to_string(),
        size_bytes: p1_data.len(),
        sha256: p1_sha256,
    });
    log_println!(
        "  [+] Extraida Particao 1  : hdd_partition_firmware_0x02000000.bin (6,173,264 bytes)"
    );

    // 6. Firmware Partition 1 - Sector Aligned (0x5E3400)
    let p1_s_data = &mmap[p1_start..p1_start + LBA_PARTITION_1_ALIGNED_SIZE];
    let p1_s_file = output_dir.join("hdd_partition_firmware_0x02000000_sector_aligned.bin");
    fs::write(&p1_s_file, p1_s_data)?;
    let (_, p1_s_sha256) = calculate_hashes(p1_s_data);
    manifest_entries.push((
        "hdd_partition_firmware_0x02000000_sector_aligned.bin".to_string(),
        p1_s_sha256.clone(),
    ));
    extracted_reports.push(ExtractedPartitionReport {
        name: "Firmware Partition 1 (Sector Aligned)".to_string(),
        filename: "hdd_partition_firmware_0x02000000_sector_aligned.bin".to_string(),
        path: p1_s_file.to_string_lossy().to_string(),
        size_bytes: p1_s_data.len(),
        sha256: p1_s_sha256,
    });
    log_println!(
        "  [+] Extraida P1 Alinhada : hdd_partition_firmware_0x02000000_sector_aligned.bin (6,173,696 bytes)"
    );

    // 7. Firmware Partition 2 - Exact Payload (0xB0000)
    let p2_start = LBA_PARTITION_2_OFFSET as usize;
    let p2_data = &mmap[p2_start..p2_start + LBA_PARTITION_2_PAYLOAD_SIZE];
    let p2_file = output_dir.join("hdd_partition_firmware_0x04000000.bin");
    fs::write(&p2_file, p2_data)?;
    let (_, p2_sha256) = calculate_hashes(p2_data);
    manifest_entries.push((
        "hdd_partition_firmware_0x04000000.bin".to_string(),
        p2_sha256.clone(),
    ));
    extracted_reports.push(ExtractedPartitionReport {
        name: "Firmware Partition 2 (Exact Payload)".to_string(),
        filename: "hdd_partition_firmware_0x04000000.bin".to_string(),
        path: p2_file.to_string_lossy().to_string(),
        size_bytes: p2_data.len(),
        sha256: p2_sha256,
    });
    log_println!(
        "  [+] Extraida Particao 2  : hdd_partition_firmware_0x04000000.bin (720,896 bytes)"
    );

    // Write SHA256SUMS.txt
    let manifest_path = output_dir.join("SHA256SUMS.txt");
    let mut manifest_file = File::create(&manifest_path)?;
    for (filename, sha) in &manifest_entries {
        writeln!(manifest_file, "{}  {}", sha, filename)?;
    }
    log_println!(
        "--------------------------------------------------------------------------------"
    );
    log_println!("  [+] Manifesto Criptografico: {}", manifest_path.display());

    // Also run inspect to generate json report if requested
    if let Some(jpath) = json_export_path {
        let _ = inspect_hdd_image(&resolved_path, false, Some(jpath))?;
    }

    log_println!(
        "================================================================================"
    );
    log_println!(" EXTRAÇÃO CIRÚRGICA CONCLUÍDA COM TOTAL FIDELIDADE                             ");
    log_println!(
        "================================================================================"
    );

    Ok(extracted_reports)
}
