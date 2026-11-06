//! Panasonic DMR-EH55 HDD Image Inspection, Partition Extraction & MEIHDFS Analysis
//!
//! Ultra-high performance zero-copy analysis utilizing `memmap2::Mmap`.

use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

use anyhow::{bail, Context, Result};
use byteorder::{ByteOrder, LittleEndian};
use memmap2::Mmap;
use serde::{Deserialize, Serialize};

use crate::constants::*;
use crate::nvram::{calculate_hashes, resolve_input_path};

/// Shannon entropy calculator (0.0000 to 8.0000 bits/byte).
pub fn calculate_entropy(data: &[u8]) -> f64 {
    if data.is_empty() {
        return 0.0;
    }
    let mut counts = [0usize; 256];
    for &b in data {
        counts[b as usize] += 1;
    }
    let total = data.len() as f64;
    let mut entropy = 0.0;
    for &c in &counts {
        if c > 0 {
            let p = c as f64 / total;
            entropy -= p * p.log2();
        }
    }
    entropy
}

/// MBR structure inspection report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MbrTableReport {
    pub lba0_offset: u64,
    pub lba0_hex_signature: String,
    pub format_version: u16,
    pub magic_ascii: String,
    pub disk_volume_guid: String,
    pub has_standard_mbr_55aa: bool,
    pub zeros_after_header_clean: bool,
    pub lba0_md5: String,
    pub lba0_sha256: String,
    pub mirror_lba: u64,
    pub mirror_offset: u64,
    pub mirror_is_bit_identical: bool,
    pub mirror_md5: String,
    pub mirror_sha256: String,
}

/// Metadata and transactional journal inspection report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationJournalReport {
    pub descriptors_offset: u64,
    pub descriptors_size: usize,
    pub descriptors_mirror_identical: bool,
    pub descriptors_sha256: String,
    pub journal_catalog_offset: u64,
    pub journal_total_blocks: u32,
    pub journal_max_index: u32,
    pub journal_block_trailers: Vec<String>,
    pub state_marker_offset: u64,
    pub state_marker_hex: String,
}

/// Embedded firmware partition descriptor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartitionReport {
    pub partition_id: u32,
    pub name: String,
    pub offset: u64,
    pub offset_hex: String,
    pub lba_start: u64,
    pub exact_payload_size: usize,
    pub sector_aligned_size: usize,
    pub sectors_count: usize,
    pub signature_hex: String,
    pub entropy: f64,
    pub exact_md5: String,
    pub exact_sha256: String,
    pub aligned_md5: Option<String>,
    pub aligned_sha256: Option<String>,
    pub header_hex_64: String,
}

/// Extracted partition/descriptor metadata report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedPartitionReport {
    pub name: String,
    pub filename: String,
    pub path: String,
    pub size_bytes: usize,
    pub sha256: String,
}

/// Chunk density metric for visual disk layout representation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkDensityReport {
    pub mb_index: usize,
    pub offset: u64,
    pub non_zero_bytes: usize,
    pub percentage: f64,
}

/// Global HDD image analysis report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HddAnalysisReport {
    pub image_path: String,
    pub file_size_bytes: u64,
    pub file_size_mb: f64,
    pub total_sectors: u64,
    pub global_md5: String,
    pub global_sha256: String,
    pub mbr_tables: MbrTableReport,
    pub allocation_and_journal: AllocationJournalReport,
    pub partitions: Vec<PartitionReport>,
    pub density_map: Vec<ChunkDensityReport>,
}

/// Single decoded 16-byte MEIHDFS allocation record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeihdfsAllocationRecord {
    pub index: usize,
    pub offset_in_table: usize,
    pub raw_hex: String,
    pub inverted_hex: String,
    pub raw_tag: u8,
    pub decoded_tag: u8,
    pub tag_name: String,
    pub extent_id: Option<u16>,
    pub sector_count: Option<u32>,
    pub lba_offset: Option<u32>,
    pub flags: Option<u16>,
    pub tail_hex: String,
    pub description: String,
}

/// Extent segment detail.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeihdfsExtentSegment {
    pub record_index: usize,
    pub subtag: u8,
    pub sector_count: u16,
    pub lba_offset_hint: u16,
    pub flags: u16,
    pub tail_hex: String,
}

/// Consolidated extent summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeihdfsExtentSummary {
    pub extent_id: u16,
    pub extent_id_hex: String,
    pub header_record_index: Option<usize>,
    pub total_sectors: u32,
    pub total_bytes: u64,
    pub segments_count: usize,
    pub segments: Vec<MeihdfsExtentSegment>,
}

/// Decoded Journal Block Report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalBlockDecoded {
    pub block_index: usize,
    pub start_lba: u64,
    pub start_offset_hex: String,
    pub is_active_header: bool,
    pub header_hex_16: String,
    pub transaction_seq: u32,
    pub pool_size: u32,
    pub checksum_crc: String,
    pub trailer_offset_hex: String,
    pub active_payload_sectors: usize,
}

/// Complete report of decoded allocation descriptors & journal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeihdfsAllocationReport {
    pub source_path: String,
    pub is_full_hdd_image: bool,
    pub is_mirror_verified: bool,
    pub descriptors_sha256: String,
    pub total_records: usize,
    pub tag_distribution: BTreeMap<String, usize>,
    pub records: Vec<MeihdfsAllocationRecord>,
    pub extents: Vec<MeihdfsExtentSummary>,
    pub journal_blocks: Vec<JournalBlockDecoded>,
}

/// Opens a disk image with zero-copy memory mapping.
pub fn open_mmap(path: &Path) -> Result<(File, Mmap)> {
    let resolved = resolve_input_path(path)?;
    let file = File::open(&resolved)
        .with_context(|| format!("Failed opening HDD image: {}", resolved.display()))?;
    let mmap = unsafe { Mmap::map(&file) }
        .with_context(|| format!("Failed creating mmap on: {}", resolved.display()))?;
    Ok((file, mmap))
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

    let (global_md5, global_sha256) = calculate_hashes(&mmap);

    println!("================================================================================");
    println!(" PANASONIC DMR-EH55 HDD FORENSIC PARSER & STORAGE ARCHITECTURE AUDITOR          ");
    println!("================================================================================");
    println!(" Arquivo da Imagem : {}", resolved_path.display());
    println!(
        " Tamanho Total     : {:>11} bytes ({:.2} MB)",
        file_size,
        file_size as f64 / (1024.0 * 1024.0)
    );
    println!(
        " Total de Setores  : {:>11} LBAs ({} bytes/setor)",
        total_sectors, LBA_SECTOR_SIZE_BYTES
    );
    println!(" MD5 Global        : {}", global_md5);
    println!(" SHA-256 Global    : {}", global_sha256);
    println!("--------------------------------------------------------------------------------");

    // 1. Inspect MBR (LBA 0)
    if mmap.len() < LBA_SECTOR_SIZE_BYTES {
        bail!("Image file is too small to contain LBA 0");
    }
    let lba0 = &mmap[..LBA_SECTOR_SIZE_BYTES];
    let (lba0_md5, lba0_sha256) = calculate_hashes(lba0);

    let format_version = LittleEndian::read_u16(&lba0[4..6]);
    let magic_ascii = String::from_utf8_lossy(&lba0[6..8]).to_string();
    let volume_guid = hex::encode_upper(&lba0[8..16]);
    let has_55aa = &lba0[510..512] == STANDARD_MBR_BOOT_SIGNATURE;
    let zeros_clean = lba0[16..512].iter().all(|&b| b == 0);

    // 2. Inspect Mirror MBR (LBA 57344)
    let mirror_offset = LBA_MIRROR_MBR_OFFSET as usize;
    let mirror_end = mirror_offset + LBA_SECTOR_SIZE_BYTES;
    if mmap.len() < mirror_end {
        bail!("Image too small to contain Mirror MBR at offset 0x{:08X}", mirror_offset);
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

    println!("\n[+] ESTRUTURA DO MASTER BOOT RECORD & TABELA ESPELHO (LBA 0 & LBA 57344)");
    println!("  * LBA 0 (Offset 0x00000000):");
    println!("    - Assinatura Bruta  : {}", hex::encode_upper(&lba0[..16]));
    println!("    - Versao de Formato : Rev {}.0 (0x{:04X})", format_version, format_version);
    println!("    - Magic Tag ASCII   : '{}' (System Storage Header)", magic_ascii);
    println!("    - Disk Volume GUID  : {} (Identificador de Formatacao)", volume_guid);
    println!(
        "    - MBR Padrao 0x55AA : {}",
        if has_55aa { "SIM" } else { "NAO (Assinatura Proprietaria Panasonic MEIHDFS RAW)" }
    );
    println!(
        "    - Zeros em 16..511  : {}",
        if zeros_clean { "100% Zeros limpos" } else { "Contem dados adicionais" }
    );
    println!("    - SHA-256 LBA 0     : {}", lba0_sha256);

    println!(
        "  * LBA {} (Offset 0x{:08X} = 28.00 MB):",
        LBA_MIRROR_MBR_SECTOR, LBA_MIRROR_MBR_OFFSET
    );
    println!(
        "    - Status do Espelho : {}",
        if mirror_identical {
            "IDENTICO BIT-A-BIT AO LBA 0 (Integridade Confirmada)"
        } else {
            "DIVERGENCIA DETECTADA"
        }
    );
    println!("    - SHA-256 LBA Mirror: {}", mirror_sha256);

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
    println!("\n[+] METADADOS DE ALOCACAO & JOURNAL TRANSACIONAL");
    println!(
        "  * Descritores Setores 24..31 (0x{:08X}, {} bytes):",
        alloc_start, ALLOCATION_DESCRIPTORS_SIZE
    );
    println!(
        "    - Espelho em LBA 57368 (0x{:08X}): {}",
        alloc_mirror_start,
        if alloc_mirror_identical {
            "100% IDENTICO (Integridade Confirmada)"
        } else {
            "Divergente"
        }
    );
    println!("    - SHA-256 Descritores  : {}", alloc_sha256);

    println!(
        "  * LBA 16384 (Offset 0x{:08X} = 8.00 MB):",
        journal_cat_start
    );
    println!("    - Catalogo Journal     : {} blocos (Max Index: {})", j_total_blocks, j_max_index);

    println!(
        "  * Regiao de Journal (0x{:08X} a 0x{:08X} = {} bytes):",
        JOURNAL_AREA_OFFSET, JOURNAL_AREA_END_OFFSET, JOURNAL_AREA_SIZE
    );
    for i in 0..JOURNAL_BLOCK_COUNT {
        let blk_start = JOURNAL_AREA_OFFSET as usize + i * JOURNAL_BLOCK_SIZE;
        let blk_slice = &mmap[blk_start..blk_start + JOURNAL_BLOCK_SIZE];
        let last_nz = blk_slice
            .iter()
            .rposition(|&b| b != 0)
            .unwrap_or(0);
        let trailer_hex = if last_nz >= 7 {
            hex::encode_upper(&blk_slice[last_nz - 7..=last_nz])
        } else {
            "0000000000000000".to_string()
        };
        let start_lba = blk_start / LBA_SECTOR_SIZE_BYTES;
        println!(
            "    - Bloco {:2} | LBA {:5} (0x{:08X}) | Trailer: {}",
            i, start_lba, blk_start, trailer_hex
        );
        journal_trailers.push(trailer_hex);
    }

    let state_marker_offset = LBA_STATE_TRANSITION_MARKER_OFFSET as usize;
    let state_marker = &mmap[state_marker_offset..state_marker_offset + 24];
    let state_marker_hex = hex::encode_upper(state_marker);
    println!("  * LBA 49152 (Offset 0x{:08X} = 24.00 MB):", state_marker_offset);
    println!("    - Marcador de Transicao: {}", state_marker_hex);

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
        && &mmap[p1_offset..p1_offset + 8] == PANASONIC_HDD_PARTITION_MAGIC
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
            signature_hex: hex::encode_upper(&PANASONIC_HDD_PARTITION_MAGIC),
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
        && &mmap[p2_offset..p2_offset + 8] == PANASONIC_HDD_PARTITION_MAGIC
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
            signature_hex: hex::encode_upper(&PANASONIC_HDD_PARTITION_MAGIC),
            entropy: (ent * 10000.0).round() / 10000.0,
            exact_md5,
            exact_sha256,
            aligned_md5: None,
            aligned_sha256: None,
            header_hex_64: hdr_64,
        });
    }

    println!("\n[+] PARTICÕES DE FIRMWARE PROPRIETÁRIAS (Magic: 51 87 13 8D 5B 0A FB 22)");
    for p in &partitions {
        println!("  ------------------------------------------------------------------------");
        println!("  PARTICAO {}: {}", p.partition_id, p.name);
        println!("  ------------------------------------------------------------------------");
        println!("  Offset Inicial    : {} (LBA {})", p.offset_hex, p.lba_start);
        println!(
            "  Tamanho Util      : {:>10} bytes ({:.3} MB / 0x{:X})",
            p.exact_payload_size,
            p.exact_payload_size as f64 / (1024.0 * 1024.0),
            p.exact_payload_size
        );
        println!(
            "  Setores Ocupados  : {:>10} LBAs ({} bytes)",
            p.sectors_count,
            p.sectors_count * LBA_SECTOR_SIZE_BYTES
        );
        println!("  Assinatura Magic  : {}", p.signature_hex);
        println!(
            "  Entropia Shannon  : {:.4} bits/byte (Alta Densidade / Criptografia Feistel)",
            p.entropy
        );
        println!("  MD5 Payload Util  : {}", p.exact_md5);
        println!("  SHA-256 Util      : {}", p.exact_sha256);
        if let Some(sha_al) = &p.aligned_sha256 {
            println!("  SHA-256 Alinhado  : {}", sha_al);
        }
    }

    // 5. Block Density Mapping across 1 MB chunks
    let mut density_map = Vec::new();
    let chunk_size = 1024 * 1024;
    let total_chunks = (file_size as usize) / chunk_size;

    if show_density {
        println!("\n[+] MAPEAMENTO VISUAL DE DENSIDADE (Blocos de 1 MB):");
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
            println!(
                "  MB {:02} (0x{:08X}): {:>9} bytes [{:<20}] ({:5.1}%)",
                mb, start, non_zero, bar, pct
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
        let json_data = serde_json::to_string_pretty(&report)
            .context("Failed serializing HDD analysis report to JSON")?;
        fs::write(json_path, json_data)
            .with_context(|| format!("Failed writing JSON report to {}", json_path.display()))?;
        println!("\n[+] Relatorio JSON estruturado exportado: {}", json_path.display());
    }

    println!("================================================================================");
    println!(" ANALISE ESTRUTURAL DE HDD CONCLUIDA COM SUCESSO                                ");
    println!("================================================================================");

    Ok(report)
}

/// Surgical extraction of all firmware partitions and filesystem descriptors.
pub fn extract_hdd_partitions(
    image_path: &Path,
    output_dir: &Path,
    json_export_path: Option<&Path>,
) -> Result<Vec<ExtractedPartitionReport>> {
    let resolved_path = resolve_input_path(image_path)?;
    let (_file, mmap) = open_mmap(&resolved_path)?;

    fs::create_dir_all(output_dir)
        .with_context(|| format!("Failed creating output directory: {}", output_dir.display()))?;

    println!("================================================================================");
    println!(" PANASONIC DMR-EH55 PARTITION & DESCRIPTOR SURGICAL EXTRACTOR                   ");
    println!("================================================================================");
    println!(" Imagem Fonte : {}", resolved_path.display());
    println!(" Destino      : {}", output_dir.display());
    println!("--------------------------------------------------------------------------------");

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
    println!("  [+] Extraido MBR Master  : hdd_mbr_master_lba0.bin (512 bytes)");

    // 2. Mirror MBR (LBA 57344)
    let mirror_start = LBA_MIRROR_MBR_OFFSET as usize;
    let mirror_data = &mmap[mirror_start..mirror_start + LBA_SECTOR_SIZE_BYTES];
    let mirror_file = output_dir.join("hdd_mbr_mirror_lba57344.bin");
    fs::write(&mirror_file, mirror_data)?;
    let (_, mirror_sha256) = calculate_hashes(mirror_data);
    manifest_entries.push(("hdd_mbr_mirror_lba57344.bin".to_string(), mirror_sha256.clone()));
    extracted_reports.push(ExtractedPartitionReport {
        name: "Mirror MBR (LBA 57344)".to_string(),
        filename: "hdd_mbr_mirror_lba57344.bin".to_string(),
        path: mirror_file.to_string_lossy().to_string(),
        size_bytes: mirror_data.len(),
        sha256: mirror_sha256,
    });
    println!("  [+] Extraido MBR Mirror  : hdd_mbr_mirror_lba57344.bin (512 bytes)");

    // 3. Allocation Descriptors (0x00003000)
    let alloc_start = ALLOCATION_DESCRIPTORS_OFFSET as usize;
    let alloc_data = &mmap[alloc_start..alloc_start + ALLOCATION_DESCRIPTORS_SIZE];
    let alloc_file = output_dir.join("hdd_allocation_descriptors_0x00003000.bin");
    fs::write(&alloc_file, alloc_data)?;
    let (_, alloc_sha256) = calculate_hashes(alloc_data);
    manifest_entries.push(("hdd_allocation_descriptors_0x00003000.bin".to_string(), alloc_sha256.clone()));
    extracted_reports.push(ExtractedPartitionReport {
        name: "MEIHDFS Allocation Descriptors (0x3000)".to_string(),
        filename: "hdd_allocation_descriptors_0x00003000.bin".to_string(),
        path: alloc_file.to_string_lossy().to_string(),
        size_bytes: alloc_data.len(),
        sha256: alloc_sha256,
    });
    println!("  [+] Extraido Descritores : hdd_allocation_descriptors_0x00003000.bin (4,096 bytes)");

    // 4. Journal Area (0x00820000 - 0x00960000)
    let j_start = JOURNAL_AREA_OFFSET as usize;
    let j_data = &mmap[j_start..j_start + JOURNAL_AREA_SIZE];
    let j_file = output_dir.join("hdd_journal_blocks_0x00820000.bin");
    fs::write(&j_file, j_data)?;
    let (_, j_sha256) = calculate_hashes(j_data);
    manifest_entries.push(("hdd_journal_blocks_0x00820000.bin".to_string(), j_sha256.clone()));
    extracted_reports.push(ExtractedPartitionReport {
        name: "Transactional Journal Area (0x820000)".to_string(),
        filename: "hdd_journal_blocks_0x00820000.bin".to_string(),
        path: j_file.to_string_lossy().to_string(),
        size_bytes: j_data.len(),
        sha256: j_sha256,
    });
    println!("  [+] Extraido Journal     : hdd_journal_blocks_0x00820000.bin (1,310,720 bytes)");

    // 5. Firmware Partition 1 - Exact Payload (0x5E3250)
    let p1_start = LBA_PARTITION_1_OFFSET as usize;
    let p1_data = &mmap[p1_start..p1_start + LBA_PARTITION_1_PAYLOAD_SIZE];
    let p1_file = output_dir.join("hdd_partition_firmware_0x02000000.bin");
    fs::write(&p1_file, p1_data)?;
    let (_, p1_sha256) = calculate_hashes(p1_data);
    manifest_entries.push(("hdd_partition_firmware_0x02000000.bin".to_string(), p1_sha256.clone()));
    extracted_reports.push(ExtractedPartitionReport {
        name: "Firmware Partition 1 (Exact Payload)".to_string(),
        filename: "hdd_partition_firmware_0x02000000.bin".to_string(),
        path: p1_file.to_string_lossy().to_string(),
        size_bytes: p1_data.len(),
        sha256: p1_sha256,
    });
    println!("  [+] Extraida Particao 1  : hdd_partition_firmware_0x02000000.bin (6,173,264 bytes)");

    // 6. Firmware Partition 1 - Sector Aligned (0x5E3400)
    let p1_s_data = &mmap[p1_start..p1_start + LBA_PARTITION_1_ALIGNED_SIZE];
    let p1_s_file = output_dir.join("hdd_partition_firmware_0x02000000_sector_aligned.bin");
    fs::write(&p1_s_file, p1_s_data)?;
    let (_, p1_s_sha256) = calculate_hashes(p1_s_data);
    manifest_entries.push(("hdd_partition_firmware_0x02000000_sector_aligned.bin".to_string(), p1_s_sha256.clone()));
    extracted_reports.push(ExtractedPartitionReport {
        name: "Firmware Partition 1 (Sector Aligned)".to_string(),
        filename: "hdd_partition_firmware_0x02000000_sector_aligned.bin".to_string(),
        path: p1_s_file.to_string_lossy().to_string(),
        size_bytes: p1_s_data.len(),
        sha256: p1_s_sha256,
    });
    println!("  [+] Extraida P1 Alinhada : hdd_partition_firmware_0x02000000_sector_aligned.bin (6,173,696 bytes)");

    // 7. Firmware Partition 2 - Exact Payload (0xB0000)
    let p2_start = LBA_PARTITION_2_OFFSET as usize;
    let p2_data = &mmap[p2_start..p2_start + LBA_PARTITION_2_PAYLOAD_SIZE];
    let p2_file = output_dir.join("hdd_partition_firmware_0x04000000.bin");
    fs::write(&p2_file, p2_data)?;
    let (_, p2_sha256) = calculate_hashes(p2_data);
    manifest_entries.push(("hdd_partition_firmware_0x04000000.bin".to_string(), p2_sha256.clone()));
    extracted_reports.push(ExtractedPartitionReport {
        name: "Firmware Partition 2 (Exact Payload)".to_string(),
        filename: "hdd_partition_firmware_0x04000000.bin".to_string(),
        path: p2_file.to_string_lossy().to_string(),
        size_bytes: p2_data.len(),
        sha256: p2_sha256,
    });
    println!("  [+] Extraida Particao 2  : hdd_partition_firmware_0x04000000.bin (720,896 bytes)");

    // Write SHA256SUMS.txt
    let manifest_path = output_dir.join("SHA256SUMS.txt");
    let mut manifest_file = File::create(&manifest_path)?;
    for (filename, sha) in &manifest_entries {
        writeln!(manifest_file, "{}  {}", sha, filename)?;
    }
    println!("--------------------------------------------------------------------------------");
    println!("  [+] Manifesto Criptografico: {}", manifest_path.display());

    // Also run inspect to generate json report if requested
    if let Some(jpath) = json_export_path {
        let _ = inspect_hdd_image(&resolved_path, false, Some(jpath))?;
    }

    println!("================================================================================");
    println!(" EXTRAÇÃO CIRÚRGICA CONCLUÍDA COM TOTAL FIDELIDADE                             ");
    println!("================================================================================");

    Ok(extracted_reports)
}

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
            bail!("Image file too small to contain allocation descriptors table at 0x{:08X}", p_start);
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
            bail!("Input descriptors file is too small (expected at least {} bytes, found {})", ALLOCATION_DESCRIPTORS_SIZE, file_len);
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
                    format!("Extent ID: 0x{:04X} | Subtag: 0x{:02X} | Flags: 0x{:04X} | Segments: {} | Param: 0x{:08X}", id, subtag, fl, p1, p2)
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
                    format!("Extent ID: 0x{:04X} | Sectors: {} ({} bytes) | LBA Off: {} | Flags: 0x{:02X}", id, sectors, sectors * 512, lba, subtag)
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
                    format!("Continuation ID: 0x{:04X} | Subtag: 0x{:02X} | Payload: {}", id, subtag, hex::encode_upper(&inv_chunk[4..12]))
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
                    format!("Boundary ID: 0x{:04X} | Subtag: 0x{:02X} | Limiter: {}", id, subtag, hex::encode_upper(&inv_chunk[4..12]))
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
                    format!("VNode Type: 0x{:02X} | Node Index: 0x{:04X} | Metadata: {}", subtag, id, hex::encode_upper(&inv_chunk[4..12]))
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
                    format!("Directory Link | Flags: 0x{:02X} | Parent: 0x{:04X} -> Child: 0x{:04X}", subtag, parent, child)
                )
            }
            _ => (
                format!("0x{:02X} (Unknown)", tag),
                None,
                None,
                None,
                None,
                format!("Unrecognized Record Tag: 0x{:02X}", tag)
            )
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
                let crc_hex = hex::encode_upper(&blk[JOURNAL_TRANSACTION_TRAILER_OFFSET..JOURNAL_TRANSACTION_TRAILER_OFFSET + 4]);
                let pool_size = LittleEndian::read_u32(&blk[JOURNAL_TRANSACTION_TRAILER_OFFSET + 4..JOURNAL_TRANSACTION_TRAILER_OFFSET + 8]);
                let seq_id = LittleEndian::read_u32(&blk[JOURNAL_TRANSACTION_SEQ_OFFSET..JOURNAL_TRANSACTION_SEQ_OFFSET + 4]);
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
                    active_payload_sectors: (JOURNAL_TRANSACTION_SEQ_OFFSET + 4) / LBA_SECTOR_SIZE_BYTES + 1,
                });
            }
        }
    }

    // Print Rich Console Telemetry
    println!("================================================================================");
    println!(" PANASONIC MEIHDFS ALLOCATION DESCRIPTORS & EXTENT FORENSIC DECODER             ");
    println!("================================================================================");
    println!(" Arquivo Analisado : {}", resolved.display());
    println!(" Modo de Origem    : {}", if is_full_image { "Imagem Completa de HDD (100 MB)" } else { "Tabela Extraida de Descritores (4,096 bytes)" });
    if is_full_image {
        println!(" Validacao Espelho : LBA 24..31 vs LBA 57368..57375 -> {}", if mirror_verified { "100% IDENTICO BIT-A-BIT (Integridade Confirmada)" } else { "DIVERGENCIA DETECTADA" });
    }
    println!(" SHA-256 Descritor : {}", descriptors_sha256);
    println!(" Total de Registros: {} registros (16 bytes cada, decodificados via NOT/XOR 0xFF)", records.len());
    println!("--------------------------------------------------------------------------------");
    println!(" DISTRIBUICAO DE TAGS DECRIPTO-FORENSES (ONE'S COMPLEMENT):");
    for (tag_name, count) in &tag_distribution {
        let pct = (*count as f64 / records.len() as f64) * 100.0;
        println!("   * {:<24} : {:>3} registros ({:5.2}%)", tag_name, count, pct);
    }
    println!("--------------------------------------------------------------------------------");
    println!(" MAPEAMENTO CONSOLIDADO DE EXTENTS & ARMAZENAMENTO MEIHDFS:");
    println!("   ID Extent | Segmentos | Total Setores | Tamanho Util | Status / Descricao");
    println!("   ----------+-----------+---------------+--------------+-------------------------------");
    for ext in extents.iter().filter(|e| e.segments_count > 0 || e.header_record_index.is_some()) {
        println!(
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
        println!("--------------------------------------------------------------------------------");
        println!(" AUDITORIA DO POOL DE JOURNAL TRANSACIONAL DE 10 BLOCOS (LBA 16640..19199):");
        println!("   Bloco | LBA Base | Offset     | Header Ativo | Seq ID | Pool | Checksum CRC | Payload");
        println!("   ------+----------+------------+--------------+--------+------+--------------+---------");
        for jb in &journal_blocks {
            println!(
                "   {:>5} | {:>8} | {} | {:>12} | {:>6} | {:>4} | {:>12} | ~{} set",
                jb.block_index,
                jb.start_lba,
                jb.start_offset_hex,
                if jb.is_active_header { "5D FF FF..." } else { "Inativo" },
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
        let json_data = serde_json::to_string_pretty(&report)
            .context("Failed serializing MEIHDFS allocation report to JSON")?;
        fs::write(json_path, json_data)
            .with_context(|| format!("Failed writing JSON report to {}", json_path.display()))?;
        println!("\n[+] Relatorio JSON estruturado exportado: {}", json_path.display());
    }

    println!("================================================================================");
    println!(" DECODIFICACAO DE METADADOS MEIHDFS CONCLUIDA COM TOTAL FIDELIDADE              ");
    println!("================================================================================");

    Ok(report)
}

/// Backward compatibility wrapper for `decode_meihdfs_descriptors`.
pub fn decode_descriptors(image_path: &Path) -> Result<()> {
    decode_meihdfs_descriptors(image_path, true, None)?;
    Ok(())
}
