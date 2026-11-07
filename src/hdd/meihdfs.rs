use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{bail, Result};
use byteorder::{ByteOrder, LittleEndian};

use crate::constants::*;
use crate::nvram::{calculate_hashes, resolve_input_path};

use super::types::{
    JournalBlockDecoded, MeihdfsAllocationRecord, MeihdfsAllocationReport, MeihdfsExtentSegment,
    MeihdfsExtentSummary,
};
use super::utils::{export_report_to_json, open_mmap};

/// Decodes the bit-inverted (One's Complement / XOR 0xFF) MEIHDFS allocation descriptors
/// from a full HDD image or an extracted 4,096-byte descriptors file, and optionally audits journal blocks.
pub fn decode_meihdfs_descriptors(
    input_path: &Path,
    include_journal: bool,
    json_export_path: Option<&Path>,
) -> Result<MeihdfsAllocationReport> {
    let resolved = resolve_input_path(input_path)?;
    let (_file, mmap) = open_mmap(&resolved)?;
    let file_len = mmap.len();

    let is_full_image = file_len >= (LBA_MIRROR_MBR_OFFSET as usize + ALLOCATION_DESCRIPTORS_SIZE);

    let (alloc_slice, mirror_verified) = if is_full_image {
        let p_start = ALLOCATION_DESCRIPTORS_OFFSET as usize;
        let p_end = p_start + ALLOCATION_DESCRIPTORS_SIZE;
        if file_len < p_end {
            bail!(
                "Image file too small to contain allocation descriptors table at 0x{:08X}",
                p_start
            );
        }
        let desc = &mmap[p_start..p_end];

        let m_start = ALLOCATION_DESCRIPTORS_MIRROR_OFFSET as usize;
        let m_end = m_start + ALLOCATION_DESCRIPTORS_SIZE;
        let mirror_ok = if file_len >= m_end {
            desc == &mmap[m_start..m_end]
        } else {
            false
        };
        (desc, mirror_ok)
    } else {
        if file_len < ALLOCATION_DESCRIPTORS_SIZE {
            bail!(
                "Input descriptors file is too small (expected at least {} bytes, found {})",
                ALLOCATION_DESCRIPTORS_SIZE,
                file_len
            );
        }
        (&mmap[..ALLOCATION_DESCRIPTORS_SIZE], false)
    };

    let (_, descriptors_sha256) = calculate_hashes(alloc_slice);

    // Decode 256 records of 16 bytes by bitwise inversion (^ 0xFF)
    let mut records = Vec::with_capacity(MEIHDFS_DESCRIPTOR_RECORDS_COUNT);
    let mut tag_distribution: BTreeMap<String, usize> = BTreeMap::new();

    for i in 0..MEIHDFS_DESCRIPTOR_RECORDS_COUNT {
        let offset = i * MEIHDFS_RECORD_SIZE;
        let raw_chunk = &alloc_slice[offset..offset + MEIHDFS_RECORD_SIZE];
        let mut inv_chunk = [0u8; MEIHDFS_RECORD_SIZE];
        for (j, &b) in raw_chunk.iter().enumerate() {
            inv_chunk[j] = b ^ 0xFF;
        }

        let tag = inv_chunk[0];
        let raw_tag = raw_chunk[0];
        let subtag = inv_chunk[1];

        let (tag_name, extent_id, sector_count, lba_offset, flags, description) = match tag {
            MEIHDFS_TAG_EXTENT_HEADER => {
                let id = LittleEndian::read_u16(&inv_chunk[2..4]);
                let fl = LittleEndian::read_u16(&inv_chunk[4..6]);
                let p1 = LittleEndian::read_u16(&inv_chunk[6..8]);
                let p2 = LittleEndian::read_u32(&inv_chunk[8..12]);
                (
                    "0x5D (ExtentHeader)".to_string(),
                    Some(id),
                    None,
                    None,
                    Some(fl),
                    format!(
                        "Extent ID: 0x{:04X} | Subtag: 0x{:02X} | Flags: 0x{:04X} | Segments: {} | Param: 0x{:08X}",
                        id, subtag, fl, p1, p2
                    ),
                )
            }
            MEIHDFS_TAG_EXTENT_POINTER => {
                let id = LittleEndian::read_u16(&inv_chunk[2..4]);
                let sectors = LittleEndian::read_u16(&inv_chunk[4..6]) as u32;
                let lba = LittleEndian::read_u16(&inv_chunk[6..8]) as u32;
                let fl = subtag as u16;
                (
                    "0x55 (ExtentPointer)".to_string(),
                    Some(id),
                    Some(sectors),
                    Some(lba),
                    Some(fl),
                    format!(
                        "Extent ID: 0x{:04X} | Sectors: {} ({} bytes) | LBA Off: {} | Flags: 0x{:02X}",
                        id,
                        sectors,
                        sectors * 512,
                        lba,
                        subtag
                    ),
                )
            }
            MEIHDFS_TAG_EXTENT_CONTINUATION => {
                let id = LittleEndian::read_u16(&inv_chunk[2..4]);
                (
                    "0x57 (ExtentContinuation)".to_string(),
                    Some(id),
                    None,
                    None,
                    Some(subtag as u16),
                    format!(
                        "Continuation ID: 0x{:04X} | Subtag: 0x{:02X} | Payload: {}",
                        id,
                        subtag,
                        hex::encode_upper(&inv_chunk[4..12])
                    ),
                )
            }
            MEIHDFS_TAG_EXTENT_BOUNDARY => {
                let id = LittleEndian::read_u16(&inv_chunk[2..4]);
                (
                    "0x5F (ExtentBoundary)".to_string(),
                    Some(id),
                    None,
                    None,
                    Some(subtag as u16),
                    format!(
                        "Boundary ID: 0x{:04X} | Subtag: 0x{:02X} | Limiter: {}",
                        id,
                        subtag,
                        hex::encode_upper(&inv_chunk[4..12])
                    ),
                )
            }
            MEIHDFS_TAG_VNODE_ENTRY => {
                let id = LittleEndian::read_u16(&inv_chunk[2..4]);
                (
                    "0xE0 (VNodeEntry)".to_string(),
                    Some(id),
                    None,
                    None,
                    Some(subtag as u16),
                    format!(
                        "VNode Type: 0x{:02X} | Node Index: 0x{:04X} | Metadata: {}",
                        subtag,
                        id,
                        hex::encode_upper(&inv_chunk[4..12])
                    ),
                )
            }
            MEIHDFS_TAG_VINODE_LINK => {
                let parent = LittleEndian::read_u16(&inv_chunk[2..4]);
                let child = LittleEndian::read_u16(&inv_chunk[4..6]);
                (
                    "0xE1 (VInodeLink)".to_string(),
                    Some(child),
                    None,
                    None,
                    Some(subtag as u16),
                    format!(
                        "Directory Link | Flags: 0x{:02X} | Parent: 0x{:04X} -> Child: 0x{:04X}",
                        subtag, parent, child
                    ),
                )
            }
            _ => (
                format!("0x{:02X} (Unknown)", tag),
                None,
                None,
                None,
                None,
                format!("Unrecognized Record Tag: 0x{:02X}", tag),
            ),
        };

        *tag_distribution.entry(tag_name.clone()).or_insert(0) += 1;

        records.push(MeihdfsAllocationRecord {
            index: i,
            offset_in_table: offset,
            raw_hex: hex::encode_upper(raw_chunk),
            inverted_hex: hex::encode_upper(inv_chunk),
            raw_tag,
            decoded_tag: tag,
            tag_name,
            extent_id,
            sector_count,
            lba_offset,
            flags,
            tail_hex: hex::encode_upper(&inv_chunk[12..16]),
            description,
        });
    }

    // Group Extents
    let mut extents_map: BTreeMap<u16, (Option<usize>, Vec<MeihdfsExtentSegment>)> = BTreeMap::new();
    for rec in &records {
        if let Some(id) = rec.extent_id {
            let entry = extents_map.entry(id).or_insert((None, Vec::new()));
            if rec.decoded_tag == MEIHDFS_TAG_EXTENT_HEADER && entry.0.is_none() {
                entry.0 = Some(rec.index);
            }
            if rec.decoded_tag == MEIHDFS_TAG_EXTENT_POINTER {
                entry.1.push(MeihdfsExtentSegment {
                    record_index: rec.index,
                    subtag: rec.flags.unwrap_or(0) as u8,
                    sector_count: rec.sector_count.unwrap_or(0) as u16,
                    lba_offset_hint: rec.lba_offset.unwrap_or(0) as u16,
                    flags: rec.flags.unwrap_or(0),
                    tail_hex: rec.tail_hex.clone(),
                });
            }
        }
    }

    let mut extents = Vec::new();
    for (id, (hdr_idx, segments)) in extents_map {
        let total_sectors: u32 = segments.iter().map(|s| s.sector_count as u32).sum();
        let total_bytes = total_sectors as u64 * 512;
        extents.push(MeihdfsExtentSummary {
            extent_id: id,
            extent_id_hex: format!("0x{:04X}", id),
            header_record_index: hdr_idx,
            total_sectors,
            total_bytes,
            segments_count: segments.len(),
            segments,
        });
    }

    // Process Journal if requested and full image available
    let mut journal_blocks = Vec::new();
    if include_journal && is_full_image {
        for b_idx in 0..JOURNAL_BLOCK_COUNT {
            let b_start = JOURNAL_AREA_OFFSET as usize + b_idx * JOURNAL_BLOCK_SIZE;
            if file_len >= b_start + JOURNAL_BLOCK_SIZE {
                let blk = &mmap[b_start..b_start + JOURNAL_BLOCK_SIZE];
                let is_active = blk[0] == MEIHDFS_TAG_EXTENT_HEADER;
                let hdr_hex = hex::encode_upper(&blk[..16]);
                let crc_hex = hex::encode_upper(
                    &blk[JOURNAL_TRANSACTION_TRAILER_OFFSET
                        ..JOURNAL_TRANSACTION_TRAILER_OFFSET + 4],
                );
                let pool_size = LittleEndian::read_u32(
                    &blk[JOURNAL_TRANSACTION_TRAILER_OFFSET + 4
                        ..JOURNAL_TRANSACTION_TRAILER_OFFSET + 8],
                );
                let seq_id = LittleEndian::read_u32(
                    &blk[JOURNAL_TRANSACTION_SEQ_OFFSET..JOURNAL_TRANSACTION_SEQ_OFFSET + 4],
                );
                let lba = (b_start / LBA_SECTOR_SIZE_BYTES) as u64;

                journal_blocks.push(JournalBlockDecoded {
                    block_index: b_idx,
                    start_lba: lba,
                    start_offset_hex: format!("0x{:08X}", b_start),
                    is_active_header: is_active,
                    header_hex_16: hdr_hex,
                    transaction_seq: seq_id,
                    pool_size,
                    checksum_crc: crc_hex,
                    trailer_offset_hex: format!("0x{:05X}", JOURNAL_TRANSACTION_TRAILER_OFFSET),
                    active_payload_sectors: (JOURNAL_TRANSACTION_SEQ_OFFSET + 4)
                        / LBA_SECTOR_SIZE_BYTES
                        + 1,
                });
            }
        }
    }

    // Print Rich Console Telemetry
    log_println!("================================================================================");
    log_println!(" PANASONIC MEIHDFS ALLOCATION DESCRIPTORS & EXTENT FORENSIC DECODER             ");
    log_println!("================================================================================");
    log_println!(" Arquivo Analisado : {}", resolved.display());
    log_println!(
        " Modo de Origem    : {}",
        if is_full_image {
            "Imagem Completa de HDD (100 MB)"
        } else {
            "Tabela Extraida de Descritores (4,096 bytes)"
        }
    );
    if is_full_image {
        log_println!(
            " Validacao Espelho : LBA 24..31 vs LBA 57368..57375 -> {}",
            if mirror_verified {
                "100% IDENTICO BIT-A-BIT (Integridade Confirmada)"
            } else {
                "DIVERGENCIA DETECTADA"
            }
        );
    }
    log_println!(" SHA-256 Descritor : {}", descriptors_sha256);
    log_println!(
        " Total de Registros: {} registros (16 bytes cada, decodificados via NOT/XOR 0xFF)",
        records.len()
    );
    log_println!("--------------------------------------------------------------------------------");
    log_println!(" DISTRIBUICAO DE TAGS DECRIPTO-FORENSES (ONE'S COMPLEMENT):");
    for (tag_name, count) in &tag_distribution {
        let pct = (*count as f64 / records.len() as f64) * 100.0;
        log_println!("   * {:<24} : {:>3} registros ({:5.2}%)", tag_name, count, pct);
    }
    log_println!("--------------------------------------------------------------------------------");
    log_println!(" MAPEAMENTO CONSOLIDADO DE EXTENTS & ARMAZENAMENTO MEIHDFS:");
    log_println!("   ID Extent | Segmentos | Total Setores | Tamanho Util | Status / Descricao");
    log_println!("   ----------+-----------+---------------+--------------+-------------------------------");
    for ext in extents
        .iter()
        .filter(|e| e.segments_count > 0 || e.header_record_index.is_some())
    {
        log_println!(
            "   {:<9} | {:>9} | {:>13} | {:>10} B | {}",
            ext.extent_id_hex,
            ext.segments_count,
            ext.total_sectors,
            ext.total_bytes,
            if let Some(h) = ext.header_record_index {
                format!("Header em Rec [{:3}] | {} frag(s)", h, ext.segments_count)
            } else {
                format!("Fragmentos orfaos: {} frag(s)", ext.segments_count)
            }
        );
    }

    if !journal_blocks.is_empty() {
        log_println!("--------------------------------------------------------------------------------");
        log_println!(" AUDITORIA DO POOL DE JOURNAL TRANSACIONAL DE 10 BLOCOS (LBA 16640..19199):");
        log_println!(
            "   Bloco | LBA Base | Offset     | Header Ativo | Seq ID | Pool | Checksum CRC | Payload"
        );
        log_println!(
            "   ------+----------+------------+--------------+--------+------+--------------+---------"
        );
        for jb in &journal_blocks {
            log_println!(
                "   {:>5} | {:>8} | {} | {:>12} | {:>6} | {:>4} | {:>12} | ~{} set",
                jb.block_index,
                jb.start_lba,
                jb.start_offset_hex,
                if jb.is_active_header {
                    "5D FF FF..."
                } else {
                    "Inativo"
                },
                jb.transaction_seq,
                jb.pool_size,
                jb.checksum_crc,
                jb.active_payload_sectors
            );
        }
    }

    let report = MeihdfsAllocationReport {
        source_path: resolved.to_string_lossy().to_string(),
        is_full_hdd_image: is_full_image,
        is_mirror_verified: mirror_verified,
        descriptors_sha256,
        total_records: records.len(),
        tag_distribution,
        records,
        extents,
        journal_blocks,
    };

    if let Some(json_path) = json_export_path {
        export_report_to_json(&report, json_path, "MEIHDFS allocation report")?;
    }

    log_println!("================================================================================");
    log_println!(" DECODIFICACAO DE METADADOS MEIHDFS CONCLUIDA COM TOTAL FIDELIDADE              ");
    log_println!("================================================================================");

    Ok(report)
}

/// Backward compatibility wrapper for `decode_meihdfs_descriptors`.
pub fn decode_descriptors(image_path: &Path) -> Result<()> {
    decode_meihdfs_descriptors(image_path, true, None)?;
    Ok(())
}
