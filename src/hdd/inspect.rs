use std::path::Path;
use anyhow::{bail, Result};
use byteorder::{ByteOrder as _, LittleEndian};
use crate::{constants::*, nvram::{calculate_hashes, resolve_input_path}};
use super::{
    types::{AllocationJournalReport, ChunkDensityReport, HddAnalysisReport, MbrTableReport, PartitionReport},
    utils::{calculate_entropy, export_report_to_json, open_mmap},
};

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

    let (global_md5, global_sha256) = calculate_hashes(&mmap);

    log_println!("================================================================================");
    log_println!(" PANASONIC DMR-EH55 HDD FORENSIC PARSER & STORAGE ARCHITECTURE AUDITOR          ");
    log_println!("================================================================================");
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
    log_println!(" MD5 Global        : {}", global_md5);
    log_println!(" SHA-256 Global    : {}", global_sha256);
    log_println!("--------------------------------------------------------------------------------");

    // 1. Inspect MBR (LBA 0)
    if mmap.len() < LBA_SECTOR_SIZE_BYTES {
        bail!("Image file is too small to contain LBA 0");
    }
    let lba0 = &mmap[..LBA_SECTOR_SIZE_BYTES];
    let (lba0_md5, lba0_sha256) = calculate_hashes(lba0);

    let format_version = LittleEndian::read_u16(&lba0[4..6]);
    let magic_ascii = String::from_utf8_lossy(&lba0[6..8]).to_string();
    let volume_guid = hex::encode_upper(&lba0[8..16]);
    let has_55aa = lba0[510..512] == STANDARD_MBR_BOOT_SIGNATURE;
    let zeros_clean = lba0[16..512].iter().all(|&b| b == 0);

    // 2. Inspect Mirror MBR (LBA 57344)
    let mirror_offset = LBA_MIRROR_MBR_OFFSET as usize;
    let mirror_end = mirror_offset + LBA_SECTOR_SIZE_BYTES;
    if mmap.len() < mirror_end {
        bail!(
            "Image too small to contain Mirror MBR at offset 0x{:08X}",
            mirror_offset
        );
    }
    let lba_mirror = &mmap[mirror_offset..mirror_end];
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

    log_println!("\n[+] ESTRUTURA DO MASTER BOOT RECORD & TABELA ESPELHO (LBA 0 & LBA 57344)");
    log_println!("  * LBA 0 (Offset 0x00000000):");
    log_println!("    - Assinatura Bruta  : {}", hex::encode_upper(&lba0[..16]));
    log_println!(
        "    - Versao de Formato : Rev {}.0 (0x{:04X})",
        format_version,
        format_version
    );
    log_println!(
        "    - Magic Tag ASCII   : '{}' (System Storage Header)",
        magic_ascii
    );
    log_println!(
        "    - Disk Volume GUID  : {} (Identificador de Formatacao)",
        volume_guid
    );
    log_println!(
        "    - MBR Padrao 0x55AA : {}",
        if has_55aa {
            "SIM"
        } else {
            "NAO (Assinatura Proprietaria Panasonic MEIHDFS RAW)"
        }
    );
    log_println!(
        "    - Zeros em 16..511  : {}",
        if zeros_clean {
            "100% Zeros limpos"
        } else {
            "Contem dados adicionais"
        }
    );
    log_println!("    - SHA-256 LBA 0     : {}", lba0_sha256);

    log_println!(
        "  * LBA {} (Offset 0x{:08X} = 28.00 MB):",
        LBA_MIRROR_MBR_SECTOR,
        LBA_MIRROR_MBR_OFFSET
    );
    log_println!(
        "    - Status do Espelho : {}",
        if mirror_identical {
            "IDENTICO BIT-A-BIT AO LBA 0 (Integridade Confirmada)"
        } else {
            "DIVERGENCIA DETECTADA"
        }
    );
    log_println!("    - SHA-256 LBA Mirror: {}", mirror_sha256);

    // 3. Inspect Allocation Descriptors & Transactional Journal
    let alloc_start = ALLOCATION_DESCRIPTORS_OFFSET as usize;
    let alloc_end = alloc_start + ALLOCATION_DESCRIPTORS_SIZE;
    let alloc_slice = &mmap[alloc_start..alloc_end];
    let (_alloc_md5, alloc_sha256) = calculate_hashes(alloc_slice);

    let alloc_mirror_start = ALLOCATION_DESCRIPTORS_MIRROR_OFFSET as usize;
    let alloc_mirror_end = alloc_mirror_start + ALLOCATION_DESCRIPTORS_SIZE;
    let alloc_mirror_slice = &mmap[alloc_mirror_start..alloc_mirror_end];
    let alloc_mirror_identical = alloc_slice == alloc_mirror_slice;

    let journal_cat_start = LBA_JOURNAL_CATALOG_OFFSET as usize;
    let journal_cat = &mmap[journal_cat_start..journal_cat_start + LBA_SECTOR_SIZE_BYTES];
    let j_total_blocks = LittleEndian::read_u32(&journal_cat[0..4]);
    let j_max_index = LittleEndian::read_u32(&journal_cat[4..8]);

    let mut journal_trailers = Vec::new();
    log_println!("\n[+] METADADOS DE ALOCACAO & JOURNAL TRANSACIONAL");
    log_println!(
        "  * Descritores Setores 24..31 (0x{:08X}, {} bytes):",
        alloc_start,
        ALLOCATION_DESCRIPTORS_SIZE
    );
    log_println!(
        "    - Espelho em LBA 57368 (0x{:08X}): {}",
        alloc_mirror_start,
        if alloc_mirror_identical {
            "100% IDENTICO (Integridade Confirmada)"
        } else {
            "Divergente"
        }
    );
    log_println!("    - SHA-256 Descritores  : {}", alloc_sha256);

    log_println!(
        "  * LBA 16384 (Offset 0x{:08X} = 8.00 MB):",
        journal_cat_start
    );
    log_println!(
        "    - Catalogo Journal     : {} blocos (Max Index: {})",
        j_total_blocks,
        j_max_index
    );

    log_println!(
        "  * Regiao de Journal (0x{:08X} a 0x{:08X} = {} bytes):",
        JOURNAL_AREA_OFFSET,
        JOURNAL_AREA_END_OFFSET,
        JOURNAL_AREA_SIZE
    );
    for i in 0..JOURNAL_BLOCK_COUNT {
        let blk_start = JOURNAL_AREA_OFFSET as usize + i * JOURNAL_BLOCK_SIZE;
        let blk_slice = &mmap[blk_start..blk_start + JOURNAL_BLOCK_SIZE];
        let last_nz = blk_slice.iter().rposition(|&b| b != 0).unwrap_or(0);
        let trailer_hex = if last_nz >= 7 {
            hex::encode_upper(&blk_slice[last_nz - 7..=last_nz])
        } else {
            "0000000000000000".to_string()
        };
        let start_lba = blk_start / LBA_SECTOR_SIZE_BYTES;
        log_println!(
            "    - Bloco {:2} | LBA {:5} (0x{:08X}) | Trailer: {}",
            i,
            start_lba,
            blk_start,
            trailer_hex
        );
        journal_trailers.push(trailer_hex);
    }

    let state_marker_offset = LBA_STATE_TRANSITION_MARKER_OFFSET as usize;
    let state_marker = &mmap[state_marker_offset..state_marker_offset + 24];
    let state_marker_hex = hex::encode_upper(state_marker);
    log_println!(
        "  * LBA 49152 (Offset 0x{:08X} = 24.00 MB):",
        state_marker_offset
    );
    log_println!("    - Marcador de Transicao: {}", state_marker_hex);

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
    if mmap.len() >= p1_offset + LBA_PARTITION_1_ALIGNED_SIZE
        && mmap[p1_offset..p1_offset + 8] == PANASONIC_HDD_PARTITION_MAGIC
    {
        let payload = &mmap[p1_offset..p1_offset + LBA_PARTITION_1_PAYLOAD_SIZE];
        let aligned = &mmap[p1_offset..p1_offset + LBA_PARTITION_1_ALIGNED_SIZE];
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
    if mmap.len() >= p2_offset + LBA_PARTITION_2_PAYLOAD_SIZE
        && mmap[p2_offset..p2_offset + 8] == PANASONIC_HDD_PARTITION_MAGIC
    {
        let payload = &mmap[p2_offset..p2_offset + LBA_PARTITION_2_PAYLOAD_SIZE];
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

    log_println!(
        "\n[+] PARTICÕES DE FIRMWARE PROPRIETÁRIAS (Magic: 51 87 13 8D 5B 0A FB 22)"
    );
    for p in &partitions {
        log_println!("  ------------------------------------------------------------------------");
        log_println!("  PARTICAO {}: {}", p.partition_id, p.name);
        log_println!("  ------------------------------------------------------------------------");
        log_println!("  Offset Inicial    : {} (LBA {})", p.offset_hex, p.lba_start);
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

    // 5. Block Density Mapping across 1 MB chunks
    let mut density_map = Vec::new();
    let chunk_size = 1024 * 1024;
    let total_chunks = (file_size as usize) / chunk_size;

    if show_density {
        log_println!("\n[+] MAPEAMENTO VISUAL DE DENSIDADE (Blocos de 1 MB):");
    }

    for mb in 0..total_chunks {
        let start = mb * chunk_size;
        let end = start + chunk_size;
        let chunk = &mmap[start..end];
        let non_zero = chunk.iter().filter(|&&b| b != 0).count();
        let pct = (non_zero as f64 / chunk_size as f64) * 100.0;

        if show_density && non_zero > 0 {
            let bar_len = (pct / 5.0).round() as usize;
            let bar: String = "#".repeat(bar_len);
            log_println!(
                "  MB {:02} (0x{:08X}): {:>9} bytes [{:<20}] ({:5.1}%)",
                mb,
                start,
                non_zero,
                bar,
                pct
            );
        }

        density_map.push(ChunkDensityReport {
            mb_index: mb,
            offset: start as u64,
            non_zero_bytes: non_zero,
            percentage: (pct * 100.0).round() / 100.0,
        });
    }

    let report = HddAnalysisReport {
        image_path: resolved_path.to_string_lossy().to_string(),
        file_size_bytes: file_size,
        file_size_mb: ((file_size as f64 / (1024.0 * 1024.0)) * 100.0).round() / 100.0,
        total_sectors,
        global_md5,
        global_sha256,
        mbr_tables: mbr_report,
        allocation_and_journal: alloc_journal_report,
        partitions,
        density_map,
    };

    if let Some(json_path) = json_export_path {
        export_report_to_json(&report, json_path, "HDD analysis report")?;
    }

    log_println!("================================================================================");
    log_println!(" ANALISE ESTRUTURAL DE HDD CONCLUIDA COM SUCESSO                                ");
    log_println!("================================================================================");

    Ok(report)
}
