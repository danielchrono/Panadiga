use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{Result, bail};
use byteorder::{ByteOrder, LittleEndian};

use crate::constants::*;
use crate::nvram::{calculate_hashes, resolve_input_path};

use super::types::{
    JournalBlockDecoded, MeihdfsAllocationRecord, MeihdfsAllocationReport, MeihdfsExtentSegment,
    MeihdfsExtentSummary,
};
use super::utils::{export_report_to_json, open_image_buffer};

/// Pure Functional Core: Decodes a single 16-byte bit-inverted MEIHDFS allocation record.
pub fn decode_single_meihdfs_record(raw_chunk: &[u8], index: usize) -> MeihdfsAllocationRecord {
    let offset = index * MEIHDFS_RECORD_SIZE;
    let inv_chunk: [u8; MEIHDFS_RECORD_SIZE] = std::array::from_fn(|j| raw_chunk[j] ^ 0xFF);

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

    MeihdfsAllocationRecord {
        index,
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
    }
}

/// Pure Functional Core: Groups decoded records into consolidated extent summaries.
pub fn aggregate_meihdfs_extents(records: &[MeihdfsAllocationRecord]) -> Vec<MeihdfsExtentSummary> {
    let mut extents_map: BTreeMap<u16, (Option<usize>, Vec<MeihdfsExtentSegment>)> =
        BTreeMap::new();
    for rec in records {
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

    extents_map
        .into_iter()
        .map(|(id, (hdr_idx, segments))| {
            let total_sectors: u32 = segments.iter().map(|s| s.sector_count as u32).sum();
            let total_bytes = total_sectors as u64 * 512;
            MeihdfsExtentSummary {
                extent_id: id,
                extent_id_hex: format!("0x{:04X}", id),
                header_record_index: hdr_idx,
                total_sectors,
                total_bytes,
                segments_count: segments.len(),
                segments,
            }
        })
        .collect()
}

/// Pure Functional Core: Decodes the 10 transactional journal blocks from a byte slice.
pub fn decode_journal_blocks_slice(data: &[u8]) -> Vec<JournalBlockDecoded> {
    let mut journal_blocks = Vec::with_capacity(JOURNAL_BLOCK_COUNT);
    for b_idx in 0..JOURNAL_BLOCK_COUNT {
        let b_start = JOURNAL_AREA_OFFSET as usize + b_idx * JOURNAL_BLOCK_SIZE;
        if data.len() >= b_start + JOURNAL_BLOCK_SIZE {
            let blk = &data[b_start..b_start + JOURNAL_BLOCK_SIZE];
            let is_active = blk[0] == MEIHDFS_TAG_EXTENT_HEADER;
            let hdr_hex = hex::encode_upper(&blk[..16]);
            let crc_hex = hex::encode_upper(
                &blk[JOURNAL_TRANSACTION_TRAILER_OFFSET..JOURNAL_TRANSACTION_TRAILER_OFFSET + 4],
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
    journal_blocks
}

/// Pure Functional Core: Decodes MEIHDFS descriptors and extent structures in-memory with zero I/O.
pub fn decode_meihdfs_slice(
    data: &[u8],
    include_journal: bool,
    source_label: &str,
) -> Result<MeihdfsAllocationReport> {
    let file_len = data.len();
    let is_full_image = file_len >= (LBA_MIRROR_MBR_OFFSET as usize + ALLOCATION_DESCRIPTORS_SIZE);

    let (alloc_slice, mirror_verified) = if is_full_image {
        let p_start = ALLOCATION_DESCRIPTORS_OFFSET as usize;
        let p_end = p_start + ALLOCATION_DESCRIPTORS_SIZE;
        if file_len < p_end {
            bail!(
                "Image slice too small to contain allocation descriptors table at 0x{:08X}",
                p_start
            );
        }
        let desc = &data[p_start..p_end];

        let m_start = ALLOCATION_DESCRIPTORS_MIRROR_OFFSET as usize;
        let m_end = m_start + ALLOCATION_DESCRIPTORS_SIZE;
        let mirror_ok = if file_len >= m_end {
            desc == &data[m_start..m_end]
        } else {
            false
        };
        (desc, mirror_ok)
    } else {
        if file_len < ALLOCATION_DESCRIPTORS_SIZE {
            bail!(
                "Input descriptors buffer too small (expected at least {} bytes, found {})",
                ALLOCATION_DESCRIPTORS_SIZE,
                file_len
            );
        }
        (&data[..ALLOCATION_DESCRIPTORS_SIZE], false)
    };

    let (_, descriptors_sha256) = calculate_hashes(alloc_slice);

    // Functional Stream: Decode all 256 records via pure mapping
    let records: Vec<MeihdfsAllocationRecord> = (0..MEIHDFS_DESCRIPTOR_RECORDS_COUNT)
        .map(|i| {
            let offset = i * MEIHDFS_RECORD_SIZE;
            decode_single_meihdfs_record(&alloc_slice[offset..offset + MEIHDFS_RECORD_SIZE], i)
        })
        .collect();

    // Functional Fold: Compute tag frequency distribution
    let tag_distribution: BTreeMap<String, usize> =
        records.iter().fold(BTreeMap::new(), |mut acc, rec| {
            *acc.entry(rec.tag_name.clone()).or_insert(0) += 1;
            acc
        });

    // Functional Extents Grouping
    let extents = aggregate_meihdfs_extents(&records);

    // Functional Journal Decoding
    let journal_blocks = if include_journal && is_full_image {
        decode_journal_blocks_slice(data)
    } else {
        Vec::new()
    };

    Ok(MeihdfsAllocationReport {
        source_path: source_label.to_string(),
        is_full_hdd_image: is_full_image,
        is_mirror_verified: mirror_verified,
        descriptors_sha256,
        total_records: records.len(),
        tag_distribution,
        records,
        extents,
        journal_blocks,
    })
}

/// Imperative Shell: Resolves input file, coordinates pure decoding, and writes console logs & JSON export.
pub fn decode_meihdfs_descriptors(
    input_path: &Path,
    include_journal: bool,
    json_export_path: Option<&Path>,
) -> Result<MeihdfsAllocationReport> {
    let resolved = resolve_input_path(input_path)?;
    let mmap = open_image_buffer(&resolved)?;

    // 1. Pure Functional Transformation
    let report = decode_meihdfs_slice(&mmap, include_journal, &resolved.to_string_lossy())?;

    // 2. Imperative Shell: Rich Console Telemetry Output
    log_println!(
        "================================================================================"
    );
    log_println!(
        " PANASONIC MEIHDFS ALLOCATION DESCRIPTORS & EXTENT FORENSIC DECODER              "
    );
    log_println!(
        "================================================================================"
    );
    log_println!(" Arquivo Analisado : {}", resolved.display());
    log_println!(
        " Modo de Origem    : {}",
        if report.is_full_hdd_image {
            "Imagem Completa de HDD (100 MB)"
        } else {
            "Tabela Extraida de Descritores (4,096 bytes)"
        }
    );
    if report.is_full_hdd_image {
        log_println!(
            " Validacao Espelho : LBA 24..31 vs LBA 57368..57375 -> {}",
            if report.is_mirror_verified {
                "100% IDENTICO BIT-A-BIT (Integridade Confirmada)"
            } else {
                "DIVERGENCIA DETECTADA"
            }
        );
    }
    log_println!(" SHA-256 Descritor : {}", report.descriptors_sha256);
    log_println!(
        " Total de Registros: {} registros (16 bytes cada, decodificados via NOT/XOR 0xFF)",
        report.records.len()
    );
    log_println!(
        "--------------------------------------------------------------------------------"
    );
    log_println!(" DISTRIBUICAO DE TAGS DECRIPTO-FORENSES (ONE'S COMPLEMENT):");
    for (tag_name, count) in &report.tag_distribution {
        let pct = (*count as f64 / report.records.len() as f64) * 100.0;
        log_println!(
            "   * {:<24} : {:>3} registros ({:5.2}%)",
            tag_name,
            count,
            pct
        );
    }
    log_println!(
        "--------------------------------------------------------------------------------"
    );
    log_println!(" MAPEAMENTO CONSOLIDADO DE EXTENTS & ARMAZENAMENTO MEIHDFS:");
    log_println!("   ID Extent | Segmentos | Total Setores | Tamanho Util | Status / Descricao");
    log_println!(
        "   ----------+-----------+---------------+--------------+-------------------------------"
    );
    for ext in report
        .extents
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

    if !report.journal_blocks.is_empty() {
        log_println!(
            "--------------------------------------------------------------------------------"
        );
        log_println!(" AUDITORIA DO POOL DE JOURNAL TRANSACIONAL DE 10 BLOCOS (LBA 16640..19199):");
        log_println!(
            "   Bloco | LBA Base | Offset     | Header Ativo | Seq ID | Pool | Checksum CRC | Payload"
        );
        log_println!(
            "   ------+----------+------------+--------------+--------+------+--------------+---------"
        );
        for jb in &report.journal_blocks {
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

    // 3. Imperative Shell: JSON Export Persistence
    if let Some(json_path) = json_export_path {
        export_report_to_json(&report, json_path, "MEIHDFS allocation report")?;
    }

    log_println!(
        "================================================================================"
    );
    log_println!(
        " DECODIFICACAO DE METADADOS MEIHDFS CONCLUIDA COM TOTAL FIDELIDADE              "
    );
    log_println!(
        "================================================================================"
    );

    Ok(report)
}

/// Backward compatibility wrapper for `decode_meihdfs_descriptors`.
pub fn decode_descriptors(image_path: &Path) -> Result<()> {
    decode_meihdfs_descriptors(image_path, true, None)?;
    Ok(())
}

#[cfg(test)]
pub mod pure_meihdfs_tests {
    use super::*;

    #[test]
    fn test_decode_single_meihdfs_record_header() {
        // Raw chunk where inverted chunk has tag 0x5D (ExtentHeader)
        // 0x5D ^ 0xFF = 0xA2
        let mut raw = [0xFFu8; 16];
        raw[0] = 0x5D ^ 0xFF; // raw tag
        raw[1] = 0x01 ^ 0xFF; // subtag
        raw[2] = 0x34 ^ 0xFF; // extent ID low
        raw[3] = 0x12 ^ 0xFF; // extent ID high (0x1234)

        let record = decode_single_meihdfs_record(&raw, 42);
        assert_eq!(record.index, 42);
        assert_eq!(record.decoded_tag, MEIHDFS_TAG_EXTENT_HEADER);
        assert_eq!(record.tag_name, "0x5D (ExtentHeader)");
        assert_eq!(record.extent_id, Some(0x1234));
    }

    #[test]
    fn test_pure_meihdfs_slice_decoding() {
        let mut descriptor_table = vec![0xFFu8; ALLOCATION_DESCRIPTORS_SIZE];
        // Create one ExtentHeader record at index 0 (ID: 0x0001)
        descriptor_table[0] = 0x5D ^ 0xFF;
        descriptor_table[2] = 0x01 ^ 0xFF;
        descriptor_table[3] = 0xFF; // 0x00 inverted

        // Create one ExtentPointer record at index 1 (ID: 0x0001, 8 sectors, LBA 100)
        descriptor_table[16] = 0x55 ^ 0xFF;
        descriptor_table[18] = 0x01 ^ 0xFF;
        descriptor_table[19] = 0xFF; // 0x00 inverted
        descriptor_table[20] = 0x08 ^ 0xFF; // 8 sectors low
        descriptor_table[21] = 0xFF; // 0x00 inverted

        let report = decode_meihdfs_slice(&descriptor_table, false, "in-memory-test")
            .expect("pure decode should succeed");

        assert_eq!(report.total_records, 256);
        assert_eq!(
            report.tag_distribution.get("0x5D (ExtentHeader)").copied(),
            Some(1)
        );
        assert_eq!(
            report.tag_distribution.get("0x55 (ExtentPointer)").copied(),
            Some(1)
        );
        assert_eq!(report.extents.len(), 1);
        assert_eq!(report.extents[0].extent_id, 0x0001);
        assert_eq!(report.extents[0].total_sectors, 8);
        assert_eq!(report.extents[0].total_bytes, 8 * 512);
    }
}
