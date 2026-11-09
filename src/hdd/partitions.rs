use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

use anyhow::{Context, Result, bail};

use crate::constants::*;
use crate::nvram::{calculate_hashes, resolve_input_path};

use super::inspect::inspect_hdd_image;
use super::types::ExtractedPartitionReport;
use super::utils::open_image_buffer;

/// Declarative specification for a Panasonic HDD firmware partition or metadata table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PartitionSpec {
    pub name: &'static str,
    pub filename: &'static str,
    pub offset: usize,
    pub size: usize,
    pub log_label: &'static str,
}

/// Static, immutable partition layout table defining the exact physical geometry.
pub const PARTITION_LAYOUT_SPECS: &[PartitionSpec] = &[
    PartitionSpec {
        name: "Master MBR (LBA 0)",
        filename: "hdd_mbr_master_lba0.bin",
        offset: 0,
        size: LBA_SECTOR_SIZE_BYTES,
        log_label: "MBR Master",
    },
    PartitionSpec {
        name: "Mirror MBR (LBA 57344)",
        filename: "hdd_mbr_mirror_lba57344.bin",
        offset: LBA_MIRROR_MBR_OFFSET as usize,
        size: LBA_SECTOR_SIZE_BYTES,
        log_label: "MBR Mirror",
    },
    PartitionSpec {
        name: "MEIHDFS Allocation Descriptors (0x3000)",
        filename: "hdd_allocation_descriptors_0x00003000.bin",
        offset: ALLOCATION_DESCRIPTORS_OFFSET as usize,
        size: ALLOCATION_DESCRIPTORS_SIZE,
        log_label: "Descritores",
    },
    PartitionSpec {
        name: "Transactional Journal Area (0x820000)",
        filename: "hdd_journal_blocks_0x00820000.bin",
        offset: JOURNAL_AREA_OFFSET as usize,
        size: JOURNAL_AREA_SIZE,
        log_label: "Journal",
    },
    PartitionSpec {
        name: "Firmware Partition 1 (Exact Payload)",
        filename: "hdd_partition_firmware_0x02000000.bin",
        offset: LBA_PARTITION_1_OFFSET as usize,
        size: LBA_PARTITION_1_PAYLOAD_SIZE,
        log_label: "Particao 1",
    },
    PartitionSpec {
        name: "Firmware Partition 1 (Sector Aligned)",
        filename: "hdd_partition_firmware_0x02000000_sector_aligned.bin",
        offset: LBA_PARTITION_1_OFFSET as usize,
        size: LBA_PARTITION_1_ALIGNED_SIZE,
        log_label: "P1 Alinhada",
    },
    PartitionSpec {
        name: "Firmware Partition 2 (Exact Payload)",
        filename: "hdd_partition_firmware_0x04000000.bin",
        offset: LBA_PARTITION_2_OFFSET as usize,
        size: LBA_PARTITION_2_PAYLOAD_SIZE,
        log_label: "Particao 2",
    },
];

/// Pure in-memory partition slice extracted without any I/O side effects.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PureExtractedPartition<'a> {
    pub spec: &'static PartitionSpec,
    pub data: &'a [u8],
    pub sha256: String,
}

/// Pure Functional Core: Extracts partition slices and computes SHA-256 hashes completely in memory.
pub fn extract_pure_partitions<'a>(
    image_bytes: &'a [u8],
    specs: &'static [PartitionSpec],
) -> Result<Vec<PureExtractedPartition<'a>>> {
    specs
        .iter()
        .map(|spec| {
            let end = spec.offset + spec.size;
            if image_bytes.len() < end {
                bail!(
                    "Image buffer too small for partition '{}': needs {} bytes, but buffer has {} bytes",
                    spec.name,
                    end,
                    image_bytes.len()
                );
            }
            let slice = &image_bytes[spec.offset..end];
            let (_, sha256) = calculate_hashes(slice);
            Ok(PureExtractedPartition {
                spec,
                data: slice,
                sha256,
            })
        })
        .collect()
}

/// Imperative Shell: Surgical extraction of all firmware partitions and filesystem descriptors to disk.
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

    // 1. Pure Functional Transformation
    let extracted_slices = extract_pure_partitions(&mmap, PARTITION_LAYOUT_SPECS)?;

    // 2. Imperative Shell: I/O Persistence and Manifest Generation
    let mut manifest_entries: Vec<(String, String)> = Vec::with_capacity(extracted_slices.len());
    let mut extracted_reports: Vec<ExtractedPartitionReport> =
        Vec::with_capacity(extracted_slices.len());

    for item in extracted_slices {
        let target_file = output_dir.join(item.spec.filename);
        fs::write(&target_file, item.data)?;
        log_println!(
            "  [+] Extraido {:<16}: {} ({} bytes)",
            item.spec.log_label,
            item.spec.filename,
            item.data.len()
        );
        manifest_entries.push((item.spec.filename.to_string(), item.sha256.clone()));
        extracted_reports.push(ExtractedPartitionReport {
            name: item.spec.name.to_string(),
            filename: item.spec.filename.to_string(),
            path: target_file.to_string_lossy().to_string(),
            size_bytes: item.data.len(),
            sha256: item.sha256,
        });
    }

    // 3. Write SHA256SUMS.txt cryptographic manifest
    let manifest_path = output_dir.join("SHA256SUMS.txt");
    let mut manifest_file = File::create(&manifest_path)?;
    for (filename, sha) in &manifest_entries {
        writeln!(manifest_file, "{}  {}", sha, filename)?;
    }
    log_println!(
        "--------------------------------------------------------------------------------"
    );
    log_println!("  [+] Manifesto Criptografico: {}", manifest_path.display());

    // 4. Run inspect to generate json report if requested
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

#[cfg(test)]
pub mod pure_partitions_tests {
    use super::*;

    #[test]
    fn test_pure_partition_extraction_in_memory() {
        const BUFFER_SIZE: usize = 0x04000000 + LBA_PARTITION_2_PAYLOAD_SIZE;
        let mut buffer = vec![0u8; BUFFER_SIZE];

        // Seed some known bytes at MBR and Partition 1
        buffer[6..8].copy_from_slice(b"SH");
        buffer[LBA_PARTITION_1_OFFSET as usize] = 0xAA;

        let result = extract_pure_partitions(&buffer, PARTITION_LAYOUT_SPECS);
        assert!(result.is_ok());

        let slices = result.unwrap();
        assert_eq!(slices.len(), 7);
        assert_eq!(slices[0].spec.filename, "hdd_mbr_master_lba0.bin");
        assert_eq!(slices[0].data.len(), 512);
        assert_eq!(&slices[0].data[6..8], b"SH");
        assert_eq!(
            slices[4].spec.filename,
            "hdd_partition_firmware_0x02000000.bin"
        );
        assert_eq!(slices[4].data[0], 0xAA);
        assert!(!slices[0].sha256.is_empty());
    }

    #[test]
    fn test_pure_partition_extraction_buffer_too_small() {
        let tiny_buffer = vec![0u8; 100];
        let result = extract_pure_partitions(&tiny_buffer, PARTITION_LAYOUT_SPECS);
        assert!(result.is_err());
    }
}
