use super::{
    types::{
        AllocationJournalReport, ChunkDensityReport, HddAnalysisReport, MbrTableReport,
        PartitionReport,
    },
    utils::{calculate_entropy, export_report_to_json, open_mmap},
};
use crate::{
    constants::*,
    nvram::{calculate_hashes, resolve_input_path},
};
use anyhow::{bail, Result};
use byteorder::{ByteOrder as _, LittleEndian};
use std::path::Path;

/// 100% pure functional inspection of a Panasonic HDD image memory slice.
/// Performs complete structural forensic analysis, MBR validation, journal inspection,
/// partition discovery, and density computation with zero disk I/O side-effects.
pub fn inspect_hdd_slice(data: &[u8], image_label: &str) -> Result<HddAnalysisReport> {
    let file_size = data.len() as u64;
    let total_sectors = file_size / LBA_SECTOR_SIZE_BYTES as u64;

    let (global_md5, global_sha256) = calculate_hashes(data);

    // 1. Inspect MBR (LBA 0)
    if data.len() < LBA_SECTOR_SIZE_BYTES {
        bail!("Image slice is too small to contain LBA 0");
    }
    let lba0 = &data[..LBA_SECTOR_SIZE_BYTES];
    let (lba0_md5, lba0_sha256) = calculate_hashes(lba0);

    let format_version = LittleEndian::read_u16(&lba0[4..6]);
    let magic_ascii = String::from_utf8_lossy(&lba0[6..8]).to_string();
    let volume_guid = hex::encode_upper(&lba0[8..16]);
    let has_55aa = lba0[510..512] == STANDARD_MBR_BOOT_SIGNATURE;
    let zeros_clean = lba0[16..512].iter().all(|&b| b == 0);

    // 2. Inspect Mirror MBR (LBA 57344)
    let mirror_offset = LBA_MIRROR_MBR_OFFSET as usize;
    let mirror_end = mirror_offset + LBA_SECTOR_SIZE_BYTES;
    if data.len() < mirror_end {
        bail!(
            "Image too small to contain Mirror MBR at offset 0x{:08X}",
            mirror_offset
        );
    }
    let lba_mirror = &data[mirror_offset..mirror_end];
    let (mirror_md5, mirror_sha256) = calculate_hashes(lba_mirror);
    let mirror_identical = lba0 == lba_mirror;

    let mbr_report = MbrTableReport {
        lba0_offset: 0,
        lba0_hex_signature: hex::encode_upper(&lba0[..16]),
        format_version,
        magic_ascii: magic_ascii.clone(),
        disk_volume_guid: volume_guid.clone(),
        has_standard_mbr_55aa: has_55aa,
        zeros_after_header_clean: zeros_clean,
        lba0_md5,
        lba0_sha256: lba0_sha256.clone(),
        mirror_lba: LBA_MIRROR_MBR_SECTOR,
        mirror_offset: LBA_MIRROR_MBR_OFFSET,
        mirror_is_bit_identical: mirror_identical,
        mirror_md5,
        mirror_sha256: mirror_sha256.clone(),
    };

    // 3. Inspect Allocation Descriptors & Transactional Journal
    let alloc_start = ALLOCATION_DESCRIPTORS_OFFSET as usize;
    let alloc_end = alloc_start + ALLOCATION_DESCRIPTORS_SIZE;
    let (alloc_mirror_identical, alloc_sha256) = if data.len() >= alloc_end {
        let alloc_slice = &data[alloc_start..alloc_end];
        let (_alloc_md5, sha) = calculate_hashes(alloc_slice);
        let alloc_mirror_start = ALLOCATION_DESCRIPTORS_MIRROR_OFFSET as usize;
        let alloc_mirror_end = alloc_mirror_start + ALLOCATION_DESCRIPTORS_SIZE;
        let mirror_match = if data.len() >= alloc_mirror_end {
            alloc_slice == &data[alloc_mirror_start..alloc_mirror_end]
        } else {
            false
        };
        (mirror_match, sha)
    } else {
        (false, String::new())
    };

    let journal_cat_start = LBA_JOURNAL_CATALOG_OFFSET as usize;
    let (j_total_blocks, j_max_index) = if data.len() >= journal_cat_start + LBA_SECTOR_SIZE_BYTES {
        let cat = &data[journal_cat_start..journal_cat_start + LBA_SECTOR_SIZE_BYTES];
        (
            LittleEndian::read_u32(&cat[0..4]),
            LittleEndian::read_u32(&cat[4..8]),
        )
    } else {
        (0, 0)
    };

    let mut journal_trailers = Vec::new();
    for i in 0..JOURNAL_BLOCK_COUNT {
        let blk_start = JOURNAL_AREA_OFFSET as usize + i * JOURNAL_BLOCK_SIZE;
        if data.len() >= blk_start + JOURNAL_BLOCK_SIZE {
            let blk_slice = &data[blk_start..blk_start + JOURNAL_BLOCK_SIZE];
            let last_nz = blk_slice.iter().rposition(|&b| b != 0).unwrap_or(0);
            let trailer_hex = if last_nz >= 7 {
                hex::encode_upper(&blk_slice[last_nz - 7..=last_nz])
            } else {
                "0000000000000000".to_string()
            };
            journal_trailers.push(trailer_hex);
        } else {
            journal_trailers.push("0000000000000000".to_string());
        }
    }

    let state_marker_offset = LBA_STATE_TRANSITION_MARKER_OFFSET as usize;
    let state_marker_hex = if data.len() >= state_marker_offset + 24 {
        hex::encode_upper(&data[state_marker_offset..state_marker_offset + 24])
    } else {
        String::new()
    };

    let alloc_journal_report = AllocationJournalReport {
        descriptors_offset: ALLOCATION_DESCRIPTORS_OFFSET,
        descriptors_size: ALLOCATION_DESCRIPTORS_SIZE,
        descriptors_mirror_identical: alloc_mirror_identical,
        descriptors_sha256: alloc_sha256,
        journal_catalog_offset: LBA_JOURNAL_CATALOG_OFFSET,
        journal_total_blocks: j_total_blocks,
        journal_max_index: j_max_index,
        journal_block_trailers: journal_trailers,
        state_marker_offset: LBA_STATE_TRANSITION_MARKER_OFFSET,
        state_marker_hex,
    };

    // 4. Discover and Validate Firmware Partitions
    let mut partitions = Vec::new();

    // Check Partition 1 (Offset 0x0200_0000 / 32 MB)
    let p1_offset = LBA_PARTITION_1_OFFSET as usize;
    if data.len() >= p1_offset + LBA_PARTITION_1_ALIGNED_SIZE
        && data[p1_offset..p1_offset + 8] == PANASONIC_HDD_PARTITION_MAGIC
    {
        let payload = &data[p1_offset..p1_offset + LBA_PARTITION_1_PAYLOAD_SIZE];
        let aligned = &data[p1_offset..p1_offset + LBA_PARTITION_1_ALIGNED_SIZE];
        let (exact_md5, exact_sha256) = calculate_hashes(payload);
        let (aligned_md5, aligned_sha256) = calculate_hashes(aligned);
        let ent = calculate_entropy(payload);
        let hdr_64 = hex::encode_upper(&payload[..64]);

        partitions.push(PartitionReport {
            partition_id: 1,
            name: "Firmware Principal & Codecs (AM34 / DIGA OS)".to_string(),
            offset: LBA_PARTITION_1_OFFSET,
            offset_hex: format!("0x{:08X}", LBA_PARTITION_1_OFFSET),
            lba_start: LBA_PARTITION_1_SECTOR,
            exact_payload_size: LBA_PARTITION_1_PAYLOAD_SIZE,
            sector_aligned_size: LBA_PARTITION_1_ALIGNED_SIZE,
            sectors_count: LBA_PARTITION_1_ALIGNED_SIZE / LBA_SECTOR_SIZE_BYTES,
            signature_hex: hex::encode_upper(PANASONIC_HDD_PARTITION_MAGIC),
            entropy: (ent * 10000.0).round() / 10000.0,
            exact_md5,
            exact_sha256,
            aligned_md5: Some(aligned_md5),
            aligned_sha256: Some(aligned_sha256),
            header_hex_64: hdr_64,
        });
    }

    // Check Partition 2 (Offset 0x0400_0000 / 64 MB)
    let p2_offset = LBA_PARTITION_2_OFFSET as usize;
    if data.len() >= p2_offset + LBA_PARTITION_2_PAYLOAD_SIZE
        && data[p2_offset..p2_offset + 8] == PANASONIC_HDD_PARTITION_MAGIC
    {
        let payload = &data[p2_offset..p2_offset + LBA_PARTITION_2_PAYLOAD_SIZE];
        let (exact_md5, exact_sha256) = calculate_hashes(payload);
        let ent = calculate_entropy(payload);
        let hdr_64 = hex::encode_upper(&payload[..64]);

        partitions.push(PartitionReport {
            partition_id: 2,
            name: "Sub-sistema / Microcodigo & Fontes OSD".to_string(),
            offset: LBA_PARTITION_2_OFFSET,
            offset_hex: format!("0x{:08X}", LBA_PARTITION_2_OFFSET),
            lba_start: LBA_PARTITION_2_SECTOR,
            exact_payload_size: LBA_PARTITION_2_PAYLOAD_SIZE,
            sector_aligned_size: LBA_PARTITION_2_PAYLOAD_SIZE,
            sectors_count: LBA_PARTITION_2_PAYLOAD_SIZE / LBA_SECTOR_SIZE_BYTES,
            signature_hex: hex::encode_upper(PANASONIC_HDD_PARTITION_MAGIC),
            entropy: (ent * 10000.0).round() / 10000.0,
            exact_md5,
            exact_sha256,
            aligned_md5: None,
            aligned_sha256: None,
            header_hex_64: hdr_64,
        });
    }

    // 5. Block Density Mapping across 1 MB chunks
    let mut density_map = Vec::new();
    let chunk_size = 1024 * 1024;
    let total_chunks = (file_size as usize) / chunk_size;

    for mb in 0..total_chunks {
        let start = mb * chunk_size;
        let end = start + chunk_size;
        let chunk = &data[start..end];
        let non_zero = chunk.iter().filter(|&&b| b != 0).count();
        let pct = (non_zero as f64 / chunk_size as f64) * 100.0;

        density_map.push(ChunkDensityReport {
            mb_index: mb,
            offset: start as u64,
            non_zero_bytes: non_zero,
            percentage: (pct * 100.0).round() / 100.0,
        });
    }

    Ok(HddAnalysisReport {
        image_path: image_label.to_string(),
        file_size_bytes: file_size,
        file_size_mb: ((file_size as f64 / (1024.0 * 1024.0)) * 100.0).round() / 100.0,
        total_sectors,
        global_md5,
        global_sha256,
        mbr_tables: mbr_report,
        allocation_and_journal: alloc_journal_report,
        partitions,
        density_map,
    })
}

/// Performs complete structural inspection and outputs rich visual console telemetry.
pub fn inspect_hdd_image(
    image_path: &Path,
    show_density: bool,
    json_export_path: Option<&Path>,
) -> Result<HddAnalysisReport> {
    let resolved_path = resolve_input_path(image_path)?;
    let (_file, mmap) = open_mmap(&resolved_path)?;
    let file_size = mmap.len() as u64;
    let total_sectors = file_size / LBA_SECTOR_SIZE_BYTES as u64;

    let report = inspect_hdd_slice(&mmap, &resolved_path.to_string_lossy())?;

    log_println!(
        "================================================================================"
    );
    log_println!(
        " PANASONIC DMR-EH55 HDD FORENSIC PARSER & STORAGE ARCHITECTURE AUDITOR          "
    );
    log_println!(
        "================================================================================"
    );
    log_println!(" Arquivo da Imagem : {}", resolved_path.display());
    log_println!(
        " Tamanho Total     : {:>11} bytes ({:.2} MB)",
        file_size,
        file_size as f64 / (1024.0 * 1024.0)
    );
    log_println!(
        " Total de Setores  : {:>11} LBAs ({} bytes/setor)",
        total_sectors,
        LBA_SECTOR_SIZE_BYTES
    );
    log_println!(" MD5 Global        : {}", report.global_md5);
    log_println!(" SHA-256 Global    : {}", report.global_sha256);
    log_println!(
        "--------------------------------------------------------------------------------"
    );

    log_println!("\n[+] ESTRUTURA DO MASTER BOOT RECORD & TABELA ESPELHO (LBA 0 & LBA 57344)");
    log_println!("  * LBA 0 (Offset 0x00000000):");
    log_println!(
        "    - Assinatura Bruta  : {}",
        report.mbr_tables.lba0_hex_signature
    );
    log_println!(
        "    - Versao de Formato : Rev {}.0 (0x{:04X})",
        report.mbr_tables.format_version,
        report.mbr_tables.format_version
    );
    log_println!(
        "    - Magic Tag ASCII   : '{}' (System Storage Header)",
        report.mbr_tables.magic_ascii
    );
    log_println!(
        "    - Disk Volume GUID  : {} (Identificador de Formatacao)",
        report.mbr_tables.disk_volume_guid
    );
    log_println!(
        "    - MBR Padrao 0x55AA : {}",
        if report.mbr_tables.has_standard_mbr_55aa {
            "SIM"
        } else {
            "NAO (Assinatura Proprietaria Panasonic MEIHDFS RAW)"
        }
    );
    log_println!(
        "    - Zeros em 16..511  : {}",
        if report.mbr_tables.zeros_after_header_clean {
            "100% Zeros limpos"
        } else {
            "Contem dados adicionais"
        }
    );
    log_println!(
        "    - SHA-256 LBA 0     : {}",
        report.mbr_tables.lba0_sha256
    );

    log_println!(
        "  * LBA {} (Offset 0x{:08X} = 28.00 MB):",
        LBA_MIRROR_MBR_SECTOR,
        LBA_MIRROR_MBR_OFFSET
    );
    log_println!(
        "    - Status do Espelho : {}",
        if report.mbr_tables.mirror_is_bit_identical {
            "IDENTICO BIT-A-BIT AO LBA 0 (Integridade Confirmada)"
        } else {
            "DIVERGENCIA DETECTADA"
        }
    );
    log_println!(
        "    - SHA-256 LBA Mirror: {}",
        report.mbr_tables.mirror_sha256
    );

    log_println!("\n[+] METADADOS DE ALOCACAO & JOURNAL TRANSACIONAL");
    log_println!(
        "  * Descritores Setores 24..31 (0x{:08X}, {} bytes):",
        report.allocation_and_journal.descriptors_offset,
        report.allocation_and_journal.descriptors_size
    );
    log_println!(
        "    - Espelho em LBA 57368 (0x{:08X}): {}",
        ALLOCATION_DESCRIPTORS_MIRROR_OFFSET,
        if report.allocation_and_journal.descriptors_mirror_identical {
            "100% IDENTICO (Integridade Confirmada)"
        } else {
            "Divergente"
        }
    );
    log_println!(
        "    - SHA-256 Descritores  : {}",
        report.allocation_and_journal.descriptors_sha256
    );

    log_println!(
        "  * LBA 16384 (Offset 0x{:08X} = 8.00 MB):",
        report.allocation_and_journal.journal_catalog_offset
    );
    log_println!(
        "    - Catalogo Journal     : {} blocos (Max Index: {})",
        report.allocation_and_journal.journal_total_blocks,
        report.allocation_and_journal.journal_max_index
    );

    log_println!(
        "  * Regiao de Journal (0x{:08X} a 0x{:08X} = {} bytes):",
        JOURNAL_AREA_OFFSET,
        JOURNAL_AREA_END_OFFSET,
        JOURNAL_AREA_SIZE
    );
    for (i, trailer) in report
        .allocation_and_journal
        .journal_block_trailers
        .iter()
        .enumerate()
    {
        let blk_start = JOURNAL_AREA_OFFSET as usize + i * JOURNAL_BLOCK_SIZE;
        let start_lba = blk_start / LBA_SECTOR_SIZE_BYTES;
        log_println!(
            "    - Bloco {:2} | LBA {:5} (0x{:08X}) | Trailer: {}",
            i,
            start_lba,
            blk_start,
            trailer
        );
    }

    log_println!(
        "  * LBA 49152 (Offset 0x{:08X} = 24.00 MB):",
        report.allocation_and_journal.state_marker_offset
    );
    log_println!(
        "    - Marcador de Transicao: {}",
        report.allocation_and_journal.state_marker_hex
    );

    log_println!("\n[+] PARTICÕES DE FIRMWARE PROPRIETÁRIAS (Magic: 51 87 13 8D 5B 0A FB 22)");
    for p in &report.partitions {
        log_println!("  ------------------------------------------------------------------------");
        log_println!("  PARTICAO {}: {}", p.partition_id, p.name);
        log_println!("  ------------------------------------------------------------------------");
        log_println!(
            "  Offset Inicial    : {} (LBA {})",
            p.offset_hex,
            p.lba_start
        );
        log_println!(
            "  Tamanho Util      : {:>10} bytes ({:.3} MB / 0x{:X})",
            p.exact_payload_size,
            p.exact_payload_size as f64 / (1024.0 * 1024.0),
            p.exact_payload_size
        );
        log_println!(
            "  Setores Ocupados  : {:>10} LBAs ({} bytes)",
            p.sectors_count,
            p.sectors_count * LBA_SECTOR_SIZE_BYTES
        );
        log_println!("  Assinatura Magic  : {}", p.signature_hex);
        log_println!(
            "  Entropia Shannon  : {:.4} bits/byte (Alta Densidade / Criptografia Feistel)",
            p.entropy
        );
        log_println!("  MD5 Payload Util  : {}", p.exact_md5);
        log_println!("  SHA-256 Util      : {}", p.exact_sha256);
        if let Some(sha_al) = &p.aligned_sha256 {
            log_println!("  SHA-256 Alinhado  : {}", sha_al);
        }
    }

    if show_density {
        log_println!("\n[+] MAPEAMENTO VISUAL DE DENSIDADE (Blocos de 1 MB):");
        for chunk in &report.density_map {
            if chunk.non_zero_bytes > 0 {
                let bar_len = (chunk.percentage / 5.0).round() as usize;
                let bar: String = "#".repeat(bar_len);
                log_println!(
                    "  MB {:02} (0x{:08X}): {:>9} bytes [{:<20}] ({:5.1}%)",
                    chunk.mb_index,
                    chunk.offset,
                    chunk.non_zero_bytes,
                    bar,
                    chunk.percentage
                );
            }
        }
    }

    if let Some(json_path) = json_export_path {
        export_report_to_json(&report, json_path, "HDD analysis report")?;
    }

    log_println!(
        "================================================================================"
    );
    log_println!(
        " ANALISE ESTRUTURAL DE HDD CONCLUIDA COM SUCESSO                                "
    );
    log_println!(
        "================================================================================"
    );

    Ok(report)
}

#[cfg(test)]
mod pure_hdd_tests {
    use super::*;

    #[test]
    fn test_inspect_hdd_slice_pure() {
        let total_size = (LBA_MIRROR_MBR_OFFSET as usize) + LBA_SECTOR_SIZE_BYTES;
        let mut mock_data = vec![0u8; total_size];

        // Format LBA 0
        mock_data[4] = 0x01; // Format version 1
        mock_data[6..8].copy_from_slice(b"SH");
        mock_data[8..16].copy_from_slice(&[0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88]);

        // Copy to mirror MBR
        let sector0 = mock_data[..LBA_SECTOR_SIZE_BYTES].to_vec();
        let mirror_offset = LBA_MIRROR_MBR_OFFSET as usize;
        mock_data[mirror_offset..mirror_offset + LBA_SECTOR_SIZE_BYTES].copy_from_slice(&sector0);

        let report = inspect_hdd_slice(&mock_data, "memory://mock_hdd.bin")
            .expect("inspect_hdd_slice should succeed on valid mock data");

        assert_eq!(report.mbr_tables.format_version, 1);
        assert_eq!(report.mbr_tables.magic_ascii, "SH");
        assert!(report.mbr_tables.mirror_is_bit_identical);
    }
}
