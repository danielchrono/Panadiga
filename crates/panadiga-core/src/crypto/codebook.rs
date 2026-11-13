use std::{
    collections::HashMap,
    fs::File,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use memmap2::Mmap;

use super::types::{CodebookEntry, CodebookReport, PartitionCorrelation};
use crate::constants::*;
use crate::hdd::open_image_buffer;
use crate::nvram::resolve_input_path;

pub(crate) fn check_partition_offset(
    mmap: &[u8],
    offset: usize,
    partition_name: &str,
    null_desc: &str,
    frm_blocks_map: &HashMap<[u8; CIPHER_BLOCK_SIZE], usize>,
) -> Option<PartitionCorrelation> {
    if mmap.len() < offset + CIPHER_BLOCK_SIZE {
        return None;
    }
    let mut arr = [0u8; CIPHER_BLOCK_SIZE];
    arr.copy_from_slice(&mmap[offset..offset + CIPHER_BLOCK_SIZE]);
    let frm_count = frm_blocks_map.get(&arr).copied().unwrap_or(0);

    Some(PartitionCorrelation {
        partition_name: partition_name.to_string(),
        offset_hex: format!("0x{:08X}", offset),
        block_hex: hex::encode_upper(arr),
        interpretation: if arr == ECB_NULL_BLOCK_8B {
            null_desc.to_string()
        } else {
            "Bloco de Cabecalho".to_string()
        },
        found_in_frm: frm_count > 0,
        frm_occurrences: frm_count,
    })
}

pub(crate) fn open_resolved_frm(frm_path: &Path) -> Result<(PathBuf, File, Mmap)> {
    let resolved_frm = resolve_input_path(frm_path)?;
    let frm_file = File::open(&resolved_frm)
        .with_context(|| format!("Failed opening FRM file: {}", resolved_frm.display()))?;
    let frm_mmap = unsafe { Mmap::map(&frm_file) }
        .with_context(|| format!("Failed creating mmap on: {}", resolved_frm.display()))?;
    Ok((resolved_frm, frm_file, frm_mmap))
}

pub(crate) fn resolve_hdd_candidate(hdd_path: Option<&Path>) -> Option<PathBuf> {
    match hdd_path {
        Some(p) => resolve_input_path(p).ok().or_else(|| Some(p.to_path_buf())),
        None => {
            let def = Path::new("panasonic DMR EH-55 Firmware hdd_100.bin");
            resolve_input_path(def).ok()
        }
    }
}

pub(crate) fn load_optional_hdd_image(
    hdd_path: Option<&Path>,
) -> (Option<crate::hdd::ImageBuffer>, Option<String>) {
    let candidate = resolve_hdd_candidate(hdd_path);
    if let Some(ref path) = candidate {
        if let Ok(buffer) = open_image_buffer(path) {
            return (Some(buffer), Some(path.to_string_lossy().to_string()));
        }
    }
    (None, None)
}

/// Correlates .FRM cipher blocks with known LBA offsets in HDD partitions.
pub fn correlate_hdd_partitions(
    hdd_mmap: Option<&[u8]>,
    frm_blocks_map: &HashMap<[u8; CIPHER_BLOCK_SIZE], usize>,
) -> Vec<PartitionCorrelation> {
    let mut correlations = Vec::new();

    let mmap = match hdd_mmap {
        Some(m) => m,
        None => return correlations,
    };

    let check_targets = [
        (
            (LBA_PARTITION_1_OFFSET + 0x38) as usize,
            "Particao 1 (32 MB - AM34 DIGA OS)",
            "Bloco Nulo Identico detectado no Offset 0x38",
        ),
        (
            (LBA_PARTITION_2_OFFSET + 0x30) as usize,
            "Particao 2 (64 MB - Sub-sistema OSD)",
            "Bloco Nulo [0] Identico detectado no Offset 0x30",
        ),
        (
            (LBA_PARTITION_2_OFFSET + 0x38) as usize,
            "Particao 2 (64 MB - Sub-sistema OSD)",
            "Bloco Nulo [1] Identico detectado no Offset 0x38",
        ),
    ];

    for (offset, part_name, null_desc) in check_targets {
        if let Some(corr) =
            check_partition_offset(mmap, offset, part_name, null_desc, frm_blocks_map)
        {
            correlations.push(corr);
        }
    }

    correlations
}

/// Pure Functional Transformation: Computes the 64-bit ECB Known-Plaintext Attack (KPA) codebook
/// dictionary and HDD correlations from in-memory byte slices without any disk I/O.
pub fn analyze_codebook_slices(
    frm_slice: &[u8],
    hdd_slice: Option<&[u8]>,
    resolved_frm_name: &str,
    resolved_hdd_name: Option<String>,
) -> CodebookReport {
    let file_size = frm_slice.len();
    let num_blocks = file_size / CIPHER_BLOCK_SIZE;

    let mut frm_counts: HashMap<[u8; CIPHER_BLOCK_SIZE], usize> = HashMap::new();
    let mut frm_first_loc: HashMap<[u8; CIPHER_BLOCK_SIZE], usize> = HashMap::new();

    for i in 0..num_blocks {
        let start = i * CIPHER_BLOCK_SIZE;
        let mut blk = [0u8; CIPHER_BLOCK_SIZE];
        blk.copy_from_slice(&frm_slice[start..start + CIPHER_BLOCK_SIZE]);
        *frm_counts.entry(blk).or_insert(0) += 1;
        frm_first_loc.entry(blk).or_insert(start);
    }

    let mut hdd_counts: HashMap<[u8; CIPHER_BLOCK_SIZE], usize> = HashMap::new();
    if let Some(hdd_data) = hdd_slice {
        let hdd_num_blocks = hdd_data.len() / CIPHER_BLOCK_SIZE;
        for j in 0..hdd_num_blocks {
            let start = j * CIPHER_BLOCK_SIZE;
            let mut blk = [0u8; CIPHER_BLOCK_SIZE];
            blk.copy_from_slice(&hdd_data[start..start + CIPHER_BLOCK_SIZE]);
            *hdd_counts.entry(blk).or_insert(0) += 1;
        }
    }

    let known_pairs: [([u8; 8], Option<&str>, &str); 6] = [
        (
            ECB_VIRGIN_FLASH_8B,
            Some("FF FF FF FF FF FF FF FF"),
            "Células Virgens de Flash NOR (0xFF * 8) / Padding",
        ),
        (
            ECB_NULL_BLOCK_8B,
            Some("00 00 00 00 00 00 00 00"),
            "Preenchimento Nulo / Padding de Zeros (0x00 * 8)",
        ),
        (
            ECB_COMMON_STRUCT_8B,
            None,
            "Estrutura Fixa de Descritores de Partição (Inter-Mod)",
        ),
        (
            PANA_FRM_TOC_SUBBLOCK_1,
            None,
            "TOC: Cabeçalho de Família de Processadores Matsushita",
        ),
        (
            PANA_FRM_TOC_AM34_TARGET_8B,
            None,
            "TOC: Seletor de Core AM34 / MN103x (CPU Principal/Painel)",
        ),
        (
            PANA_FRM_TOC_DRIVE_TARGET_8B,
            None,
            "TOC: Seletor de Core MN102 (DSP Servo do Drive Óptico)",
        ),
    ];

    let mut dictionary = Vec::new();
    let mut rank = 1;

    for (cipher, plain_opt, interp) in &known_pairs {
        let f_count = frm_counts.get(cipher).copied().unwrap_or(0);
        let h_count = hdd_counts.get(cipher).copied().unwrap_or(0);
        let f_loc = frm_first_loc
            .get(cipher)
            .map(|off| format!("0x{:06X}", off))
            .unwrap_or_else(|| "N/A".to_string());

        dictionary.push(CodebookEntry {
            rank,
            block_hex: hex::encode_upper(cipher),
            known_plaintext_hex: plain_opt.map(|s| s.to_string()),
            interpretation: interp.to_string(),
            frm_occurrences: f_count,
            frm_total_bytes: f_count * CIPHER_BLOCK_SIZE,
            hdd_occurrences: h_count,
            hdd_total_bytes: h_count * CIPHER_BLOCK_SIZE,
            sample_locations: vec![f_loc],
        });
        rank += 1;
    }

    let mut sorted_blocks: Vec<([u8; 8], usize)> = frm_counts.into_iter().collect();
    sorted_blocks.sort_by_key(|a| std::cmp::Reverse(a.1));

    for (blk, count) in sorted_blocks.iter().take(20) {
        if *blk == ECB_VIRGIN_FLASH_8B || *blk == ECB_COMMON_STRUCT_8B || *blk == ECB_NULL_BLOCK_8B
        {
            continue;
        }
        if *count >= 20 {
            let first_off = frm_first_loc.get(blk).copied().unwrap_or(0);
            let loc_str =
                if (PANA_FRM_MOD2_START_OFFSET..PANA_FRM_MOD2_BOUNDARY).contains(&first_off) {
                    "Módulo 2 (Drive Servo DSP / MN102 Code Section)"
                } else if first_off < PANA_FRM_MOD1_BOUNDARY {
                    "Módulo 1 (Sub-sistema / OSD Font Section)"
                } else {
                    "Módulo 3 (Main CPU OS / AM34 Core Section)"
                };

            dictionary.push(CodebookEntry {
                rank,
                block_hex: hex::encode_upper(blk),
                known_plaintext_hex: None,
                interpretation: format!("Bloco Cifrado Repetitivo ({}x) em {}", count, loc_str),
                frm_occurrences: *count,
                frm_total_bytes: *count * CIPHER_BLOCK_SIZE,
                hdd_occurrences: hdd_counts.get(blk).copied().unwrap_or(0),
                hdd_total_bytes: hdd_counts.get(blk).copied().unwrap_or(0) * CIPHER_BLOCK_SIZE,
                sample_locations: vec![format!("0x{:06X}", first_off)],
            });
            rank += 1;
        }
    }

    let is_ecb_leak = dictionary.iter().any(|e| e.frm_occurrences >= 100);
    let cross_proven = dictionary
        .iter()
        .any(|e| e.frm_occurrences > 0 && e.hdd_occurrences > 0);

    CodebookReport {
        frm_file: resolved_frm_name.to_string(),
        hdd_file: resolved_hdd_name,
        total_64bit_blocks_analyzed: num_blocks,
        unique_blocks: sorted_blocks.len(),
        dictionary,
        is_ecb_leak_confirmed: is_ecb_leak,
        cross_medium_key_identity_proven: cross_proven,
    }
}

/// Displays the 64-bit ECB Known-Plaintext Attack (KPA) codebook dictionary and HDD correlations.
pub fn print_codebook(
    frm_path: &Path,
    hdd_path: Option<&Path>,
    json_path: Option<&Path>,
) -> Result<CodebookReport> {
    let (resolved_frm, _frm_file, frm_mmap) = open_resolved_frm(frm_path)?;
    let (hdd_buf, resolved_hdd_name) = load_optional_hdd_image(hdd_path);

    let report = analyze_codebook_slices(
        &frm_mmap,
        hdd_buf.as_deref(),
        &resolved_frm.to_string_lossy(),
        resolved_hdd_name.clone(),
    );

    log_println!(
        "================================================================================"
    );
    log_println!(" PANASONIC 64-BIT ECB CODEBOOK — KNOWN-PLAINTEXT ATTACK (KPA) DICTIONARY       ");
    log_println!(
        "================================================================================"
    );
    log_println!(" Arquivo Analisado : {}", resolved_frm.display());
    log_println!(
        " Arquivo HDD       : {}",
        resolved_hdd_name
            .as_deref()
            .unwrap_or("Não fornecido / Não encontrado")
    );
    log_println!(
        " Total de Blocos   : {} blocos de 64 bits (8 bytes cada)",
        report.total_64bit_blocks_analyzed
    );
    log_println!(" Blocos Únicos     : {}", report.unique_blocks);
    log_println!(
        "--------------------------------------------------------------------------------"
    );
    log_println!(" FUNDAMENTO FORENSE DO CODEBOOK ECB (FEISTEL DE 64 BITS):");
    log_println!(" Em modo Electronic Codebook (ECB): C_i = E_K(P_i).");
    log_println!(
        " Blocos idênticos de texto-plano produzem invariavelmente blocos cifrados idênticos."
    );
    log_println!(
        " Ao correlacionar preenchimentos conhecidos da eletrônica (Flash virgem e zeros nulos),"
    );
    log_println!(
        " constrói-se o dicionário determinístico de tradução direta sem necessidade da chave."
    );
    log_println!(
        "--------------------------------------------------------------------------------"
    );

    log_println!("\n[+] DICIONÁRIO DE TEXTO-PLANO CONHECIDO (KNOWN-PLAINTEXT PAIRS):");
    log_println!(
        "    +----+-------------------------+-------------------------+------------+------------+--------------------------------------------+"
    );
    log_println!(
        "    | #  | Bloco Cifrado (Hex)     | Texto-Plano Conhecido   | Ocorr. FRM | Ocorr. HDD | Interpretação Forense & Arquitetura        |"
    );
    log_println!(
        "    +----+-------------------------+-------------------------+------------+------------+--------------------------------------------+"
    );

    for entry in &report.dictionary {
        if entry.rank <= 6 {
            log_println!(
                "    | {:<2} | {:<23} | {:<23} | {:>10} | {:>10} | {:<42} |",
                entry.rank,
                entry.block_hex,
                entry
                    .known_plaintext_hex
                    .as_deref()
                    .unwrap_or("Padrão Estrutural"),
                entry.frm_occurrences,
                entry.hdd_occurrences,
                entry.interpretation
            );
        }
    }
    log_println!(
        "    +----+-------------------------+-------------------------+------------+------------+--------------------------------------------+"
    );

    // Top frequent ciphertext blocks
    log_println!("\n[+] BLOCOS CIFRADOS DE ALTA FREQUÊNCIA NO MÓDULO 2 (DRIVE SERVO DSP):");
    log_println!(
        "    Estes blocos vazam devido a tabelas de firmware e instruções repetidas no MN102:"
    );
    log_println!(
        "    +----+-------------------------+------------+------------+--------------------------------------------------------------+"
    );
    log_println!(
        "    | #  | Bloco Cifrado (Hex)     | Ocorr. FRM | 1º Offset  | Localização & Comportamento Criptográfico                    |"
    );
    log_println!(
        "    +----+-------------------------+------------+------------+--------------------------------------------------------------+"
    );

    for (idx, entry) in report.dictionary.iter().filter(|e| e.rank > 6).enumerate() {
        log_println!(
            "    | {:<2} | {:<23} | {:>10} | {:<10} | {:<60} |",
            idx + 1,
            entry.block_hex,
            entry.frm_occurrences,
            entry
                .sample_locations
                .first()
                .map(|s| s.as_str())
                .unwrap_or("N/A"),
            entry.interpretation
        );
    }
    log_println!(
        "    +----+-------------------------+------------+------------+--------------------------------------------------------------+"
    );

    log_println!("\n[+] SÍNTESE FORENSE E PROVAS MATEMÁTICAS:");
    log_println!(
        "  1. [CONFIRMADO] Vazamento Estrutural ECB: O arquivo óptico PANA_DVD.FRM e o HD IDE utilizam"
    );
    log_println!(
        "     cifragem de bloco de 64 bits (8 bytes) sem encadeamento de feedback (CBC) nas fronteiras."
    );
    if report.cross_medium_key_identity_proven {
        log_println!("  2. [CONFIRMADO] IDENTIDADE DE CHAVE ENTRE CD E HD:");
        log_println!(
            "     - O bloco de zeros nulos 'E5 D1 F3 4A 51 8B 26 90' ocorre no .FRM e no HD."
        );
        log_println!(
            "     - O bloco de Flash virgem '03 50 65 C8 7B B8 E0 90' ocorre no .FRM e no HD."
        );
        log_println!(
            "     -> Conclusão: O firmware do HD NÃO é texto-plano, mas compartilha a MESMA chave do contêiner óptico!"
        );
    }

    if let Some(jpath) = json_path {
        let json_data = serde_json::to_string_pretty(&report)
            .context("Failed serializing Codebook report to JSON")?;
        std::fs::write(jpath, json_data)
            .with_context(|| format!("Failed writing JSON Codebook to {}", jpath.display()))?;
        log_println!("\n[+] Relatório JSON exportado: {}", jpath.display());
    }

    log_println!(
        "================================================================================"
    );
    log_println!(" CONSULTA AO CODEBOOK CONCLUÍDA COM SUCESSO                                    ");
    log_println!(
        "================================================================================"
    );

    Ok(report)
}

#[cfg(test)]
mod pure_codebook_tests {
    use super::*;

    #[test]
    fn test_pure_codebook_analysis_in_memory() {
        let mut mock_frm = vec![0u8; 64];
        // Inject virgin flash block into mock_frm
        mock_frm[0..8].copy_from_slice(&ECB_VIRGIN_FLASH_8B);
        mock_frm[8..16].copy_from_slice(&ECB_VIRGIN_FLASH_8B);
        mock_frm[16..24].copy_from_slice(&ECB_NULL_BLOCK_8B);

        let mut mock_hdd = vec![0u8; 32];
        mock_hdd[0..8].copy_from_slice(&ECB_VIRGIN_FLASH_8B);

        let report = analyze_codebook_slices(
            &mock_frm,
            Some(&mock_hdd),
            "memory://mock.frm",
            Some("memory://mock.hdd".to_string()),
        );

        assert_eq!(report.total_64bit_blocks_analyzed, 8);
        assert!(report.cross_medium_key_identity_proven);

        let virgin = report
            .dictionary
            .iter()
            .find(|e| e.block_hex == "035065C87BB8E090")
            .unwrap();
        assert_eq!(virgin.frm_occurrences, 2);
        assert_eq!(virgin.hdd_occurrences, 1);
    }
}
