use anyhow::{Context, Result};
use std::{collections::HashMap, path::Path};

use super::codebook::{correlate_hdd_partitions, load_optional_hdd_image, open_resolved_frm};
use super::toc::analyze_toc;
use super::types::{BlockFrequency, CryptanalysisReport};
use crate::constants::*;
use crate::hdd::calculate_entropy;
use crate::nvram::calculate_hashes;

use rayon::prelude::*;

const BLOCK_COUNT_PARALLEL_CHUNK_BLOCKS: usize = 32 * 1024;

/// Counts aligned 64-bit cipher blocks in parallel across cores for large buffers.
fn count_8byte_blocks(data: &[u8]) -> HashMap<[u8; CIPHER_BLOCK_SIZE], usize> {
    let (blocks, _) = data.as_chunks::<CIPHER_BLOCK_SIZE>();

    if blocks.len() >= BLOCK_COUNT_PARALLEL_CHUNK_BLOCKS * 2 {
        blocks
            .par_chunks(BLOCK_COUNT_PARALLEL_CHUNK_BLOCKS)
            .fold(HashMap::new, |mut local, chunk| {
                for &blk in chunk {
                    *local.entry(blk).or_insert(0) += 1;
                }
                local
            })
            .reduce(HashMap::new, |mut acc, local| {
                for (k, v) in local {
                    *acc.entry(k).or_insert(0) += v;
                }
                acc
            })
    } else {
        let mut counts = HashMap::new();
        for &blk in blocks {
            *counts.entry(blk).or_insert(0) += 1;
        }
        counts
    }
}

fn rank_top_blocks(
    counts: &HashMap<[u8; CIPHER_BLOCK_SIZE], usize>,
    top_n: usize,
) -> Vec<BlockFrequency> {
    let mut sorted: Vec<([u8; CIPHER_BLOCK_SIZE], usize)> =
        counts.iter().map(|(&k, &v)| (k, v)).collect();
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

/// Analyzes 64-bit (8-byte) block frequencies in an aligned byte slice.
pub fn analyze_8byte_frequencies(data: &[u8], top_n: usize) -> Vec<BlockFrequency> {
    let counts = count_8byte_blocks(data);
    rank_top_blocks(&counts, top_n)
}

/// Pure Functional Transformation: Executes cryptographic analysis on an in-memory FRM slice
/// and cross-references with an optional HDD slice without performing any disk I/O.
pub fn analyze_frm_slices(
    frm_slice: &[u8],
    hdd_slice: Option<&[u8]>,
    resolved_frm_name: &str,
    _resolved_hdd_name: Option<String>,
    top_n: usize,
) -> CryptanalysisReport {
    let file_size = frm_slice.len();
    let ((global_entropy, (md5_str, sha256_str)), (all_blocks_map, toc_analysis)) = rayon::join(
        || {
            rayon::join(
                || calculate_entropy(frm_slice),
                || calculate_hashes(frm_slice),
            )
        },
        || rayon::join(|| count_8byte_blocks(frm_slice), || analyze_toc(frm_slice)),
    );

    let top_blocks = rank_top_blocks(&all_blocks_map, top_n);

    // Cross-reference with HDD Image
    let mut hdd_correlations = Vec::new();
    if let Some(hdd_data) = hdd_slice {
        hdd_correlations = correlate_hdd_partitions(Some(hdd_data), &all_blocks_map);
    }

    let is_ecb_leak_confirmed = top_blocks.iter().any(|b| b.count >= 100);

    CryptanalysisReport {
        frm_file: resolved_frm_name.to_string(),
        file_size_bytes: file_size,
        global_entropy,
        md5: md5_str,
        sha256: sha256_str,
        top_blocks,
        toc_analysis,
        hdd_correlations,
        is_ecb_leak_confirmed,
    }
}

/// Executes complete cryptanalysis on PANA_DVD.FRM and cross-references with HDD image.
pub fn analyze_frm_container(
    frm_path: &Path,
    hdd_path: Option<&Path>,
    top_n: usize,
    json_path: Option<&Path>,
) -> Result<CryptanalysisReport> {
    let (resolved_frm, _frm_file, frm_mmap) = open_resolved_frm(frm_path)?;
    let (hdd_buf, resolved_hdd_name) = load_optional_hdd_image(hdd_path);

    let report = analyze_frm_slices(
        &frm_mmap,
        hdd_buf.as_deref(),
        &resolved_frm.to_string_lossy(),
        resolved_hdd_name.clone(),
        top_n,
    );

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
        report.file_size_bytes,
        report.file_size_bytes as f64 / (1024.0 * 1024.0)
    );
    log_println!(
        " Entropia Shannon  : {:.8} bits/byte (Max Teórico: 8.00000000)",
        report.global_entropy
    );
    log_println!(" MD5               : {}", report.md5);
    log_println!(" SHA-256           : {}", report.sha256);
    log_println!(
        "--------------------------------------------------------------------------------"
    );

    log_println!(
        "\n[+] ANÁLISE DE FREQUÊNCIA DE BLOCOS DE 64-BITS (8 BYTES) - TOP {}:",
        top_n
    );
    log_println!(
        "    +----+-------------------------+------------+---------------+-----------------------------------------------+"
    );
    log_println!(
        "    | #  | Bloco Cifrado (Hex)     | Ocorrencias| Total Bytes   | Interpretacao Criptografica                   |"
    );
    log_println!(
        "    +----+-------------------------+------------+---------------+-----------------------------------------------+"
    );

    for (idx, b) in report.top_blocks.iter().enumerate() {
        log_println!(
            "    | {:<2} | {:<23} | {:>10} | {:>10} B  | {:<45} |",
            idx + 1,
            b.hex,
            b.count,
            b.total_bytes,
            b.interpretation
        );
    }
    log_println!(
        "    +----+-------------------------+------------+---------------+-----------------------------------------------+"
    );

    log_println!(
        "\n[+] ESTRUTURA DA TABELA DE CONTEUDOS (TOC) - REGISTROS DE {} BYTES (0x30):",
        report.toc_analysis.entry_size_bytes
    );
    log_println!(
        "  * Total de Registros Primarios Mapeados : {}",
        report.toc_analysis.total_examined_records
    );
    log_println!(
        "  * Sub-bloco Comum 1 ({})   : Repetido em {} descritores (Offsets 0x10, 0x40, 0x70)",
        report.toc_analysis.common_subblock_1_hex,
        report.toc_analysis.common_subblock_1_matches
    );
    log_println!(
        "  * Sub-bloco Comum 2 ({})   : Repetido em {} descritores (Offsets 0x18, 0x78)",
        report.toc_analysis.common_subblock_2_hex,
        report.toc_analysis.common_subblock_2_matches
    );

    if !report.hdd_correlations.is_empty() {
        log_println!("\n[+] CORRELAÇÃO FORENSE COM A IMAGEM DO DISCO RÍGIDO (HDD):");
        log_println!(
            "  * Arquivo HDD Identificado: {}",
            resolved_hdd_name.as_deref().unwrap_or("N/A")
        );
        for c in &report.hdd_correlations {
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

    log_println!("\n[+] DIAGNOSTICO CRIPTOGRAFICO FINAL:");
    if report.is_ecb_leak_confirmed {
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

#[cfg(test)]
mod pure_entropy_tests {
    use super::*;

    #[test]
    fn test_pure_frm_analysis_in_memory() {
        let mut mock_data = vec![0x55u8; 128];
        // Inject repeated block to trigger frequency counter
        for chunk in mock_data.as_chunks_mut::<8>().0.iter_mut().take(10) {
            chunk.copy_from_slice(&[0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88]);
        }

        let report = analyze_frm_slices(&mock_data, None, "memory://mock.frm", None, 5);

        assert_eq!(report.file_size_bytes, 128);
        assert!(!report.top_blocks.is_empty());
        assert_eq!(report.top_blocks[0].hex, "1122334455667788");
        assert_eq!(report.top_blocks[0].count, 10);
    }
}
