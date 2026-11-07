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

    // Forensic Invariant #3: Reject in-place overwrite of original flash dump
    let same_path = resolved_input == resolved_output
        || (resolved_input.canonicalize().ok().is_some()
            && resolved_input.canonicalize().ok() == resolved_output.canonicalize().ok());
    if same_path {
        bail!(
            "[FORENSIC INVARIANT VIOLATION] Output path ({}) cannot overwrite input flash dump ({}) in-place.",
            resolved_output.display(),
            resolved_input.display()
        );
    }

    let backup_dir = resolved_output
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let backup_nvram_path = backup_dir.join("nvram_corrupt_backup.bin");

    log_println!("======================================================================");
    log_println!("   PANASONIC DMR-EH55 NVRAM CLEANER & FLASH INTEGRITY ARCHITECT      ");
    log_println!("            Spansion S29JL064H NOR Flash (8 MB / 64 Mbit)             ");
    log_println!("======================================================================");
    log_println!("  Input File   : {}", resolved_input.display());
    log_println!("  Output File  : {}", resolved_output.display());
    log_println!("  Backup File  : {}", backup_nvram_path.display());
    log_println!("----------------------------------------------------------------------");

    // Read input binary
    let mut flash_bytes = Vec::with_capacity(SPANSION_FLASH_SIZE_BYTES);
    let mut file = File::open(&resolved_input)
        .with_context(|| format!("Failed to open flash binary: {}", resolved_input.display()))?;
    file.read_to_end(&mut flash_bytes)
        .with_context(|| "Failed reading flash binary content")?;

    // Multi-model support: Standard Panasonic DIGA Top-Boot NOR Flash capacities
    // 2 MB (E50/E55), 4 MB (E85H/E95H), 8 MB (EH50/EH55/EH56/EH60/EH65), 16 MB (EX75/EX85)
    let flash_len = flash_bytes.len();
    let valid_capacities = [
        2 * 1024 * 1024,
        4 * 1024 * 1024,
        8 * 1024 * 1024,
        16 * 1024 * 1024,
    ];
    if !valid_capacities.contains(&flash_len) {
        bail!(
            "Invalid flash dump size: {} bytes. Expected standard Panasonic DIGA NOR Flash capacity (2 MB, 4 MB, 8 MB or 16 MB).",
            flash_len
        );
    }

    let nvram_base_offset = flash_len - SPANSION_NVRAM_SIZE_BYTES;
    let nvram_end_offset = flash_len;
    let bootloader_size = nvram_base_offset;
    let base_sector_idx = match flash_len {
        8388608 => 134, // S29JL064H Top-Boot SA134..SA141
        4194304 => 70,  // S29JL032H Top-Boot SA70..SA77
        2097152 => 38,  // S29AL016J Top-Boot SA38..SA45
        _ => 262,
    };

    let (orig_md5, orig_sha256) = calculate_hashes(&flash_bytes);
    let size_mb = flash_len as f64 / (1024.0 * 1024.0);
    log_println!("  Dump Size    : {} bytes ({:.2} MB / {} Mbit)", flash_len, size_mb, (flash_len * 8) / (1024 * 1024));
    log_println!("  Original MD5 : {}", orig_md5);
    log_println!("  Orig SHA-256 : {}", orig_sha256);
    log_println!("----------------------------------------------------------------------");

    // Extract raw NVRAM block
    let nvram_slice = &flash_bytes[nvram_base_offset..nvram_end_offset];
    fs::write(&backup_nvram_path, nvram_slice).with_context(|| {
        format!(
            "Failed writing corrupt NVRAM backup to {}",
            backup_nvram_path.display()
        )
    })?;

    log_println!(
        "[OK] Corrupted NVRAM sector (0x{:06X} - 0x{:06X}) extracted to:",
        nvram_base_offset,
        nvram_end_offset - 1
    );
    log_println!("     -> {} ({} bytes)", backup_nvram_path.display(), nvram_slice.len());

    // Perform deep content audit on the NVRAM block
    let non_ff = nvram_slice.iter().filter(|&&b| b != FLASH_ERASED_BYTE).count();
    let non_zero = nvram_slice.iter().filter(|&&b| b != 0x00).count();
    let pct_non_ff = (non_ff as f64 / SPANSION_NVRAM_SIZE_BYTES as f64) * 100.0;
    let pct_non_zero = (non_zero as f64 / SPANSION_NVRAM_SIZE_BYTES as f64) * 100.0;

    log_println!("\n[+] ANALISE ESTRUTURAL DA NVRAM CORROMPIDA:");
    log_println!("    * Bytes nao-FF (dados gravados) : {:6} / {} ({:.2}%)", non_ff, SPANSION_NVRAM_SIZE_BYTES, pct_non_ff);
    log_println!("    * Bytes nao-zero                : {:6} / {} ({:.2}%)", non_zero, SPANSION_NVRAM_SIZE_BYTES, pct_non_zero);
    log_println!("\n    * Auditoria Detalhada dos 8 Micro-Setores (SA{} a SA{} - 8 KB cada):", base_sector_idx, base_sector_idx + 7);
    log_println!("    +-------+----------+--------------------+-------------------------+");
    log_println!("    | Setor |  Offset  | Ativos (Nao-0xFF)  | Header (Primeiros 8B)   |");
    log_println!("    +-------+----------+--------------------+-------------------------+");

    let mut microsectors = Vec::with_capacity(SPANSION_MICROSECTOR_COUNT);
    for idx in 0..SPANSION_MICROSECTOR_COUNT {
        let sec_offset = idx * SPANSION_MICROSECTOR_SIZE;
        let sec_slice = &nvram_slice[sec_offset..sec_offset + SPANSION_MICROSECTOR_SIZE];
        let used = sec_slice.iter().filter(|&&b| b != FLASH_ERASED_BYTE).count();
        let header_hex = hex::encode_upper(&sec_slice[..8]);
        let sec_name = format!("SA{}", base_sector_idx + idx);
        let abs_offset = nvram_base_offset + sec_offset;

        log_println!(
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
    log_println!("    +-------+----------+--------------------+-------------------------+");

    // Preserve original OS and Bootloader byte sequence for strict immutability check
    let original_os_bytes = flash_bytes[..bootloader_size].to_vec();

    // Erase NVRAM to factory virgin state (0xFF)
    for byte in &mut flash_bytes[nvram_base_offset..nvram_end_offset] {
        *byte = FLASH_ERASED_BYTE;
    }

    // Verify 100% bit-for-bit immutability of OS and Bootloader area
    if flash_bytes[..bootloader_size] != original_os_bytes[..] {
        bail!("[CRITICAL ERROR] Bootloader/OS memory region (0x00000000 - 0x{:08X}) was modified during clean! Aborting.", bootloader_size);
    }
    log_println!("\n[OK] Verificacao de Integridade: Area do Bootloader/OS (primeiros {} bytes)", bootloader_size);
    log_println!("     Bit-a-bit 100% IDENTICA E IMUTAVEL.");

    // Write clean binary
    fs::write(&resolved_output, &flash_bytes)
        .with_context(|| format!("Failed writing clean binary to {}", resolved_output.display()))?;

    let (clean_md5, clean_sha256) = calculate_hashes(&flash_bytes);
    log_println!("----------------------------------------------------------------------");
    log_println!("[SUCESSO] Imagem limpa gerada com perfeicao:");
    log_println!("  Arquivo Limpo: {}", resolved_output.display());
    log_println!("  Clean MD5    : {}", clean_md5);
    log_println!("  Clean SHA-256: {}", clean_sha256);
    log_println!("======================================================================");
    log_println!("PRONTO PARA REGRAVACAO NA MEMORIA TSOP-48 VIA XGECU T48 / TL866II / SVOD 4!");
    log_println!("Ao inicializar, a CPU Panasonic AM34 recriara a NVRAM limpa de fabrica.");
    log_println!("======================================================================");

    Ok(NvramCleanReport {
        input_file: resolved_input.to_string_lossy().to_string(),
        output_file: resolved_output.to_string_lossy().to_string(),
        backup_file: backup_nvram_path.to_string_lossy().to_string(),
        original_md5: orig_md5,
        original_sha256: orig_sha256,
        clean_md5,
        clean_sha256,
        total_flash_bytes: flash_len,
        nvram_offset_start: nvram_base_offset,
        nvram_offset_end: nvram_end_offset,
        nvram_non_ff_bytes: non_ff,
        nvram_non_zero_bytes: non_zero,
        microsectors,
        bootloader_immutable: true,
    })
}