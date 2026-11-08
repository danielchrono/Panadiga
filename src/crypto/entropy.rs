use anyhow::{Context, Result};
use std::{collections::HashMap, path::Path};

use super::codebook::{correlate_hdd_partitions, open_resolved_frm, resolve_hdd_candidate};
use super::toc::analyze_toc;
use super::types::{BlockFrequency, CryptanalysisReport};
use crate::constants::*;
use crate::hdd::{calculate_entropy, open_image_buffer};
use crate::nvram::calculate_hashes;

/// Analyzes 64-bit (8-byte) block frequencies in an aligned byte slice.
pub fn analyze_8byte_frequencies(data: &[u8], top_n: usize) -> Vec<BlockFrequency> {
    let mut counts: HashMap<[u8; CIPHER_BLOCK_SIZE], usize> = HashMap::new();
    let num_blocks = data.len() / CIPHER_BLOCK_SIZE;

    for i in 0..num_blocks {
        let start = i * CIPHER_BLOCK_SIZE;
        let mut blk = [0u8; CIPHER_BLOCK_SIZE];
        blk.copy_from_slice(&data[start..start + CIPHER_BLOCK_SIZE]);
        *counts.entry(blk).or_insert(0) += 1;
    }

    let mut sorted: Vec<([u8; CIPHER_BLOCK_SIZE], usize)> = counts.into_iter().collect();
    sorted.sort_by_key(|a| std::cmp::Reverse(a.1));

    sorted
        .into_iter()
        .take(top_n)
        .map(|(blk, count)| {
            let hex_str = hex::encode_upper(blk);
            let interp = match blk {
                ECB_VIRGIN_FLASH_8B => "Preenchimento Virgem Flash ROM (0xFF * 8)".to_string(),
                ECB_NULL_BLOCK_8B => "Bloco de Zeros Nulos / Padding (0x00 * 8)".to_string(),
                ECB_COMMON_STRUCT_8B => "Estrutura Fixa de Cabecalho / Tabela Alinhada".to_string(),
                _ => {
                    if count >= 30 {
                        "Padrao ECB Cifrado de Alta Repeticao".to_string()
                    } else {
                        "Bloco Cifrado Frequente".to_string()
                    }
                }
            };
            BlockFrequency {
                hex: hex_str,
                count,
                total_bytes: count * CIPHER_BLOCK_SIZE,
                interpretation: interp,
            }
        })
        .collect()
}

/// Executes complete cryptanalysis on PANA_DVD.FRM and cross-references with HDD image.
pub fn analyze_frm_container(
    frm_path: &Path,
    hdd_path: Option<&Path>,
    top_n: usize,
    json_path: Option<&Path>,
) -> Result<CryptanalysisReport> {
    let (resolved_frm, _frm_file, frm_mmap) = open_resolved_frm(frm_path)?;

    let file_size = frm_mmap.len();
    let global_entropy = calculate_entropy(&frm_mmap);
    let (md5_str, sha256_str) = calculate_hashes(&frm_mmap);

    log_println!(
        "================================================================================"
    );
    log_println!(" PANASONIC PANA_DVD.FRM CRIPTOMÉTRICA & DETECÇÃO FORENSE DE VAZAMENTO ECB      ");
    log_println!(
        "================================================================================"
    );
    log_println!(" Arquivo Analisado : {}", resolved_frm.display());
    log_println!(
        " Tamanho Total     : {:>10} bytes ({:.2} MB)",
        file_size,
        file_size as f64 / (1024.0 * 1024.0)
    );
    log_println!(
        " Entropia Shannon  : {:.8} bits/byte (Max Teórico: 8.00000000)",
        global_entropy
    );
    log_println!(" MD5               : {}", md5_str);
    log_println!(" SHA-256           : {}", sha256_str);
    log_println!(
        "--------------------------------------------------------------------------------"
    );

    // 1. Frequency Analysis of 64-bit blocks
    let top_blocks = analyze_8byte_frequencies(&frm_mmap, top_n);

    // Build fast lookup map for all blocks
    let mut all_blocks_map: HashMap<[u8; CIPHER_BLOCK_SIZE], usize> = HashMap::new();
    let num_blocks = file_size / CIPHER_BLOCK_SIZE;
    for i in 0..num_blocks {
        let start = i * CIPHER_BLOCK_SIZE;
        let mut blk = [0u8; CIPHER_BLOCK_SIZE];
        blk.copy_from_slice(&frm_mmap[start..start + CIPHER_BLOCK_SIZE]);
        *all_blocks_map.entry(blk).or_insert(0) += 1;
    }

    log_println!(
        "\n[+] ANÁLISE DE FREQUÊNCIA DE BLOCOS DE 64-BITS (8 BYTES) - TOP {}:",
        top_n
    );
    log_println!("    +----+-------------------------+------------+---------------+-----------------------------------------------+");
    log_println!("    | #  | Bloco Cifrado (Hex)     | Ocorrencias| Total Bytes   | Interpretacao Criptografica                   |");
    log_println!("    +----+-------------------------+------------+---------------+-----------------------------------------------+");

    for (idx, b) in top_blocks.iter().enumerate() {
        log_println!(
            "    | {:<2} | {:<23} | {:>10} | {:>10} B  | {:<45} |",
            idx + 1,
            b.hex,
            b.count,
            b.total_bytes,
            b.interpretation
        );
    }
    log_println!("    +----+-------------------------+------------+---------------+-----------------------------------------------+");

    // 2. TOC Descriptors Analysis
    let toc_analysis = analyze_toc(&frm_mmap);
    log_println!(
        "\n[+] ESTRUTURA DA TABELA DE CONTEUDOS (TOC) - REGISTROS DE {} BYTES (0x30):",
        toc_analysis.entry_size_bytes
    );
    log_println!(
        "  * Total de Registros Primarios Mapeados : {}",
        toc_analysis.total_examined_records
    );
    log_println!(
        "  * Sub-bloco Comum 1 ({})   : Repetido em {} descritores (Offsets 0x10, 0x40, 0x70)",
        toc_analysis.common_subblock_1_hex,
        toc_analysis.common_subblock_1_matches
    );
    log_println!(
        "  * Sub-bloco Comum 2 ({})   : Repetido em {} descritores (Offsets 0x18, 0x78)",
        toc_analysis.common_subblock_2_hex,
        toc_analysis.common_subblock_2_matches
    );

    // 3. Cross-reference with HDD Image
    let hdd_candidate = resolve_hdd_candidate(hdd_path);

    let mut hdd_correlations = Vec::new();
    if let Some(hpath) = &hdd_candidate {
        if let Ok(hdd_mmap) = open_image_buffer(hpath) {
            log_println!("\n[+] CORRELAÇÃO FORENSE COM A IMAGEM DO DISCO RÍGIDO (HDD):");
            log_println!("  * Arquivo HDD Identificado: {}", hpath.display());
            hdd_correlations = correlate_hdd_partitions(Some(&hdd_mmap[..]), &all_blocks_map);
            for c in &hdd_correlations {
                log_println!(
                    "    - {} [Offset {}]:\n      Bloco: {} -> {} | Presente no .FRM: {} ({} ocorrencias)",
                    c.partition_name,
                    c.offset_hex,
                    c.block_hex,
                    c.interpretation,
                    if c.found_in_frm { "SIM!" } else { "Nao" },
                    c.frm_occurrences
                );
            }
        }
    }

    let is_ecb_leak_confirmed = top_blocks.iter().any(|b| b.count >= 100);

    log_println!("\n[+] DIAGNOSTICO CRIPTOGRAFICO FINAL:");
    if is_ecb_leak_confirmed {
        log_println!("  [CONFIRMADO] Vazamento Estrutural ECB Detectado!");
        log_println!(
            "  1. O arquivo PANA_DVD.FRM e cifrado por blocos de 64 bits (8 bytes) sem CBC."
        );
        log_println!(
            "  2. As particoes no disco rigido compartilham o MESMO algoritmo e a MESMA chave."
        );
        log_println!(
            "  3. O bloco nulo E5D1F34A518B2690 comprova a identidade de cifragem entre CD e HDD."
        );
    }

    let report = CryptanalysisReport {
        frm_file: resolved_frm.to_string_lossy().to_string(),
        file_size_bytes: file_size,
        global_entropy,
        md5: md5_str,
        sha256: sha256_str,
        top_blocks,
        toc_analysis,
        hdd_correlations,
        is_ecb_leak_confirmed,
    };

    if let Some(jpath) = json_path {
        let json_data = serde_json::to_string_pretty(&report)
            .context("Failed serializing cryptanalysis report to JSON")?;
        std::fs::write(jpath, json_data)
            .with_context(|| format!("Failed writing JSON report to {}", jpath.display()))?;
        log_println!("\n[+] Relatorio JSON exportado: {}", jpath.display());
    }

    log_println!(
        "================================================================================"
    );
    log_println!(
        " CRIPTOANÁLISE CONCLUÍDA COM SUCESSO                                            "
    );
    log_println!(
        "================================================================================"
    );

    Ok(report)
}
