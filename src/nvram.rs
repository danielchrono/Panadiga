//! Panasonic DMR-EH55 NVRAM / Flash Management Subsystem
//!
//! Handles surgical cleaning, auditing, and verification of Spansion S29JL064H NOR Flash dumps.

use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use md5::{Digest as _, Md5};
use sha2::Sha256;

use crate::constants::*;

/// Audit metrics for an individual 8 KB micro-sector.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MicroSectorAudit {
    pub sector_name: String,
    pub offset: usize,
    pub active_bytes: usize,
    pub total_bytes: usize,
    pub header_hex: String,
}

/// Comprehensive audit summary of the flash cleaning operation.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NvramCleanReport {
    pub input_file: String,
    pub output_file: String,
    pub backup_file: String,
    pub original_md5: String,
    pub original_sha256: String,
    pub clean_md5: String,
    pub clean_sha256: String,
    pub total_flash_bytes: usize,
    pub nvram_offset_start: usize,
    pub nvram_offset_end: usize,
    pub nvram_non_ff_bytes: usize,
    pub nvram_non_zero_bytes: usize,
    pub microsectors: Vec<MicroSectorAudit>,
    pub bootloader_immutable: bool,
}

/// Computes MD5 and SHA-256 hashes of a byte slice.
pub fn calculate_hashes(data: &[u8]) -> (String, String) {
    let mut md5_hasher = Md5::new();
    md5_hasher.update(data);
    let md5_str = hex::encode(md5_hasher.finalize());

    let mut sha_hasher = Sha256::new();
    sha_hasher.update(data);
    let sha256_str = hex::encode(sha_hasher.finalize());

    (md5_str, sha256_str)
}

/// Intelligently resolves an input file path across working directory and known folders.
pub fn resolve_input_path(input: &Path) -> Result<PathBuf> {
    if input.exists() {
        return Ok(input.to_path_buf());
    }

    // Try relative to firmware_and_images directory
    let candidate = Path::new("firmware_and_images").join(input);
    if candidate.exists() {
        return Ok(candidate);
    }

    // Try relative to parent directories if run from within subdirectories
    let candidate_parent = Path::new("../../firmware_and_images").join(input);
    if candidate_parent.exists() {
        return Ok(candidate_parent);
    }

    bail!("Flash binary input file not found: {}", input.display())
}

/// Executes the complete NVRAM clean, backup, audit, and verification workflow.
pub fn clean_flash_nvram(input_path: &Path, output_path: Option<&Path>) -> Result<NvramCleanReport> {
    let resolved_input = resolve_input_path(input_path)?;

    // Determine output file path
    let resolved_output = match output_path {
        Some(p) => p.to_path_buf(),
        None => {
            let parent = resolved_input.parent().unwrap_or_else(|| Path::new("."));
            let stem = resolved_input
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("flash_dump");
            let ext = resolved_input
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("bin");
            parent.join(format!("{stem}_CLEAN.{ext}"))
        }
    };

    let backup_dir = resolved_output
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let backup_nvram_path = backup_dir.join("nvram_corrupt_backup.bin");

    println!("======================================================================");
    println!("   PANASONIC DMR-EH55 NVRAM CLEANER & FLASH INTEGRITY ARCHITECT      ");
    println!("            Spansion S29JL064H NOR Flash (8 MB / 64 Mbit)             ");
    println!("======================================================================");
    println!("  Input File   : {}", resolved_input.display());
    println!("  Output File  : {}", resolved_output.display());
    println!("  Backup File  : {}", backup_nvram_path.display());
    println!("----------------------------------------------------------------------");

    // Read input binary
    let mut flash_bytes = Vec::with_capacity(SPANSION_FLASH_SIZE_BYTES);
    let mut file = File::open(&resolved_input)
        .with_context(|| format!("Failed to open flash binary: {}", resolved_input.display()))?;
    file.read_to_end(&mut flash_bytes)
        .with_context(|| "Failed reading flash binary content")?;

    // Strict validation of Spansion S29JL064H 8MB capacity
    if flash_bytes.len() != SPANSION_FLASH_SIZE_BYTES {
        bail!(
            "Invalid flash dump size: {} bytes. Expected Spansion S29JL064H capacity: {} bytes (8 MB).",
            flash_bytes.len(),
            SPANSION_FLASH_SIZE_BYTES
        );
    }

    let (orig_md5, orig_sha256) = calculate_hashes(&flash_bytes);
    println!("  Dump Size    : {} bytes (8.00 MB / 64 Mbit)", flash_bytes.len());
    println!("  Original MD5 : {}", orig_md5);
    println!("  Orig SHA-256 : {}", orig_sha256);
    println!("----------------------------------------------------------------------");

    // Extract raw NVRAM block (0x007F_0000 .. 0x0080_0000)
    let nvram_slice = &flash_bytes[SPANSION_NVRAM_BASE_OFFSET..SPANSION_NVRAM_END_OFFSET];
    fs::write(&backup_nvram_path, nvram_slice).with_context(|| {
        format!(
            "Failed writing corrupt NVRAM backup to {}",
            backup_nvram_path.display()
        )
    })?;

    println!(
        "[OK] Corrupted NVRAM sector (0x{:06X} - 0x{:06X}) extracted to:",
        SPANSION_NVRAM_BASE_OFFSET,
        SPANSION_NVRAM_END_OFFSET - 1
    );
    println!("     -> {} ({} bytes)", backup_nvram_path.display(), nvram_slice.len());

    // Perform deep content audit on the NVRAM block
    let non_ff = nvram_slice.iter().filter(|&&b| b != FLASH_ERASED_BYTE).count();
    let non_zero = nvram_slice.iter().filter(|&&b| b != 0x00).count();
    let pct_non_ff = (non_ff as f64 / SPANSION_NVRAM_SIZE_BYTES as f64) * 100.0;
    let pct_non_zero = (non_zero as f64 / SPANSION_NVRAM_SIZE_BYTES as f64) * 100.0;

    println!("\n[+] ANALISE ESTRUTURAL DA NVRAM CORROMPIDA:");
    println!("    * Bytes nao-FF (dados gravados) : {:6} / {} ({:.2}%)", non_ff, SPANSION_NVRAM_SIZE_BYTES, pct_non_ff);
    println!("    * Bytes nao-zero                : {:6} / {} ({:.2}%)", non_zero, SPANSION_NVRAM_SIZE_BYTES, pct_non_zero);
    println!("\n    * Auditoria Detalhada dos 8 Micro-Setores (SA134 a SA141 - 8 KB cada):");
    println!("    +-------+----------+--------------------+-------------------------+");
    println!("    | Setor |  Offset  | Ativos (Nao-0xFF)  | Header (Primeiros 8B)   |");
    println!("    +-------+----------+--------------------+-------------------------+");

    let mut microsectors = Vec::with_capacity(SPANSION_MICROSECTOR_COUNT);
    for idx in 0..SPANSION_MICROSECTOR_COUNT {
        let sec_offset = idx * SPANSION_MICROSECTOR_SIZE;
        let sec_slice = &nvram_slice[sec_offset..sec_offset + SPANSION_MICROSECTOR_SIZE];
        let used = sec_slice.iter().filter(|&&b| b != FLASH_ERASED_BYTE).count();
        let header_hex = hex::encode_upper(&sec_slice[..8]);
        let sec_name = format!("SA{}", SPANSION_MICROSECTOR_BASE_INDEX + idx);
        let abs_offset = SPANSION_NVRAM_BASE_OFFSET + sec_offset;

        println!(
            "    | {:<5} | 0x{:06X} | {:4} / {} ({:5.1}%) | {:<23} |",
            sec_name,
            abs_offset,
            used,
            SPANSION_MICROSECTOR_SIZE,
            (used as f64 / SPANSION_MICROSECTOR_SIZE as f64) * 100.0,
            header_hex
        );

        microsectors.push(MicroSectorAudit {
            sector_name: sec_name,
            offset: abs_offset,
            active_bytes: used,
            total_bytes: SPANSION_MICROSECTOR_SIZE,
            header_hex,
        });
    }
    println!("    +-------+----------+--------------------+-------------------------+");

    // Preserve original OS and Bootloader byte sequence for strict immutability check
    let original_os_bytes = flash_bytes[..SPANSION_BOOTLOADER_SIZE_BYTES].to_vec();

    // Erase NVRAM to factory virgin state (0xFF)
    for byte in &mut flash_bytes[SPANSION_NVRAM_BASE_OFFSET..SPANSION_NVRAM_END_OFFSET] {
        *byte = FLASH_ERASED_BYTE;
    }

    // Verify 100% bit-for-bit immutability of OS and Bootloader area
    if flash_bytes[..SPANSION_BOOTLOADER_SIZE_BYTES] != original_os_bytes[..] {
        bail!("[CRITICAL ERROR] Bootloader/OS memory region (0x00000000 - 0x007EFFFF) was modified during clean! Aborting.");
    }
    println!("\n[OK] Verificacao de Integridade: Area do Bootloader/OS (primeiros {} bytes)", SPANSION_BOOTLOADER_SIZE_BYTES);
    println!("     Bit-a-bit 100% IDENTICA E IMUTAVEL.");

    // Write clean binary
    fs::write(&resolved_output, &flash_bytes)
        .with_context(|| format!("Failed writing clean binary to {}", resolved_output.display()))?;

    let (clean_md5, clean_sha256) = calculate_hashes(&flash_bytes);
    println!("----------------------------------------------------------------------");
    println!("[SUCESSO] Imagem limpa gerada com perfeicao:");
    println!("  Arquivo Limpo: {}", resolved_output.display());
    println!("  Clean MD5    : {}", clean_md5);
    println!("  Clean SHA-256: {}", clean_sha256);
    println!("======================================================================");
    println!("PRONTO PARA REGRAVACAO NA MEMORIA TSOP-48 VIA XGECU T48 / TL866II / SVOD 4!");
    println!("Ao inicializar, a CPU Panasonic AM34 recriara a NVRAM limpa de fabrica.");
    println!("======================================================================");

    Ok(NvramCleanReport {
        input_file: resolved_input.to_string_lossy().to_string(),
        output_file: resolved_output.to_string_lossy().to_string(),
        backup_file: backup_nvram_path.to_string_lossy().to_string(),
        original_md5: orig_md5,
        original_sha256: orig_sha256,
        clean_md5,
        clean_sha256,
        total_flash_bytes: flash_bytes.len(),
        nvram_offset_start: SPANSION_NVRAM_BASE_OFFSET,
        nvram_offset_end: SPANSION_NVRAM_END_OFFSET,
        nvram_non_ff_bytes: non_ff,
        nvram_non_zero_bytes: non_zero,
        microsectors,
        bootloader_immutable: true,
    })
}
