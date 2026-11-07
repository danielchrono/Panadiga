use std::fs::{self, File};
use std::path::Path;

use anyhow::{Context, Result};
use memmap2::Mmap;

use crate::nvram::resolve_input_path;

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

/// Opens a disk image with zero-copy memory mapping.
pub fn open_mmap(path: &Path) -> Result<(File, Mmap)> {
    let resolved = resolve_input_path(path)?;
    let file = File::open(&resolved)
        .with_context(|| format!("Failed opening HDD image: {}", resolved.display()))?;
    let file_len = file.metadata()?.len();
    if file_len == 0 {
        anyhow::bail!(
            "Cannot memory map empty file (0 bytes): {}",
            resolved.display()
        );
    }
    let mmap = unsafe { Mmap::map(&file) }
        .with_context(|| format!("Failed creating mmap on: {}", resolved.display()))?;
    Ok((file, mmap))
}

/// Helper function to serialize any report struct to formatted JSON and write to disk.
pub fn export_report_to_json<T: serde::Serialize>(
    report: &T,
    json_path: &Path,
    report_desc: &str,
) -> Result<()> {
    let json_data = serde_json::to_string_pretty(report)
        .with_context(|| format!("Failed serializing {report_desc} to JSON"))?;
    fs::write(json_path, json_data)
        .with_context(|| format!("Failed writing JSON report to {}", json_path.display()))?;
    log_println!(
        "\n[+] Structured JSON report exported: {}",
        json_path.display()
    );
    Ok(())
}
