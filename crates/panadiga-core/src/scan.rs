//! Deep Binary Forensics & String Scanner
//!
//! High-performance zero-copy string extraction and keyword matching for
//! reverse engineering firmware, memory dumps, and storage containers.

use std::fs::File;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use memmap2::Mmap;
use serde::{Deserialize, Serialize};

use crate::nvram::resolve_input_path;

/// Default forensic keyword dictionary for Panasonic DIGA / Matsushita research.
pub const DEFAULT_FORENSIC_KEYWORDS: &[&str] = &[
    "panasonic",
    "diga",
    "firmware",
    "boot",
    "dvd",
    "version",
    "error",
    "system",
    "copyright",
    "spansion",
    "mei",
    "matsushita",
    "dmr",
    "mn103s",
    "p7501",
    "vep79138",
    "am34",
    "meihdfs",
    "u99",
];

/// A single matched string record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StringMatch {
    pub offset: usize,
    pub offset_hex: String,
    pub text: String,
    pub matched_keyword: Option<String>,
}

/// Structured report of binary scan results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanReport {
    pub file_path: String,
    pub file_size_bytes: usize,
    pub min_length: usize,
    pub total_strings_found: usize,
    pub total_keyword_matches: usize,
    pub matches: Vec<StringMatch>,
}

const SCAN_PARALLEL_CHUNK_BYTES: usize = 512 * 1024;

const fn build_ascii_lut() -> [bool; 256] {
    let mut lut = [false; 256];
    let mut i = 0usize;
    while i < 256 {
        let b = i as u8;
        lut[i] = b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'.' | b'/' | b' ' | b':');
        i += 1;
    }
    lut
}

const ASCII_STR_LUT: [bool; 256] = build_ascii_lut();

#[inline]
fn contains_ascii_ci(haystack: &[u8], needle_lower: &[u8]) -> bool {
    if needle_lower.is_empty() {
        return true;
    }
    if haystack.len() < needle_lower.len() {
        return false;
    }
    haystack
        .windows(needle_lower.len())
        .any(|win| win.eq_ignore_ascii_case(needle_lower))
}

fn scan_data_chunk(
    data: &[u8],
    base_offset: usize,
    min_length: usize,
    kw_lower: &[String],
    limit: usize,
) -> (usize, Vec<StringMatch>) {
    let mut matches = Vec::new();
    let mut total_found = 0usize;
    let mut current_start: Option<usize> = None;

    let evaluate_candidate =
        |start: usize, end: usize, total: &mut usize, acc: &mut Vec<StringMatch>| {
            if end - start < min_length {
                return;
            }
            let slice = &data[start..end];
            if let Ok(text) = std::str::from_utf8(slice) {
                *total += 1;
                if limit != 0 && acc.len() >= limit {
                    return;
                }
                if kw_lower.is_empty() {
                    let abs_off = base_offset + start;
                    acc.push(StringMatch {
                        offset: abs_off,
                        offset_hex: format!("0x{:08X}", abs_off),
                        text: text.to_string(),
                        matched_keyword: None,
                    });
                    return;
                }
                let matched_kw = kw_lower
                    .iter()
                    .find(|kw| kw.is_ascii() && contains_ascii_ci(slice, kw.as_bytes()));
                if let Some(kw) = matched_kw {
                    let abs_off = base_offset + start;
                    acc.push(StringMatch {
                        offset: abs_off,
                        offset_hex: format!("0x{:08X}", abs_off),
                        text: text.to_string(),
                        matched_keyword: Some(kw.clone()),
                    });
                }
            }
        };

    for (i, &b) in data.iter().enumerate() {
        if ASCII_STR_LUT[b as usize] {
            if current_start.is_none() {
                current_start = Some(i);
            }
        } else if let Some(start) = current_start {
            evaluate_candidate(start, i, &mut total_found, &mut matches);
            current_start = None;
        }
    }

    if let Some(start) = current_start {
        evaluate_candidate(start, data.len(), &mut total_found, &mut matches);
    }

    (total_found, matches)
}

/// Scans a byte slice for ASCII strings and filters against target keywords (parallelized above 512 KiB).
pub fn scan_data_strings(
    data: &[u8],
    min_length: usize,
    keywords: &[String],
    limit: usize,
) -> (usize, Vec<StringMatch>) {
    use rayon::prelude::*;

    let kw_lower: Vec<String> = keywords.iter().map(|k| k.to_lowercase()).collect();
    if data.len() < SCAN_PARALLEL_CHUNK_BYTES * 2 {
        return scan_data_chunk(data, 0, min_length, &kw_lower, limit);
    }

    // Partition cleanly at non-string delimiter bytes so runs are never split mid-string.
    let mut bounds = Vec::with_capacity(data.len() / SCAN_PARALLEL_CHUNK_BYTES + 2);
    bounds.push(0usize);
    let mut pos = SCAN_PARALLEL_CHUNK_BYTES;
    while pos < data.len() {
        while pos < data.len() && ASCII_STR_LUT[data[pos] as usize] {
            pos += 1;
        }
        if pos < data.len() {
            bounds.push(pos);
            pos += SCAN_PARALLEL_CHUNK_BYTES;
        }
    }
    bounds.push(data.len());

    let chunk_results: Vec<(usize, Vec<StringMatch>)> = bounds
        .windows(2)
        .collect::<Vec<_>>()
        .into_par_iter()
        .map(|w| scan_data_chunk(&data[w[0]..w[1]], w[0], min_length, &kw_lower, limit))
        .collect();

    let mut total_found = 0usize;
    let mut matches = Vec::new();
    for (sub_total, sub_matches) in chunk_results {
        total_found += sub_total;
        if limit == 0 {
            matches.extend(sub_matches);
        } else if matches.len() < limit {
            let remaining = limit - matches.len();
            matches.extend(sub_matches.into_iter().take(remaining));
        }
    }
    (total_found, matches)
}

/// Executes deep zero-copy string scanning on a binary file.
pub fn run_deep_scan(
    file_path: &Path,
    min_length: usize,
    custom_keywords: Option<Vec<String>>,
    limit: usize,
    json_path: Option<&Path>,
) -> Result<ScanReport> {
    let resolved = resolve_input_path(file_path)?;
    let file = File::open(&resolved)
        .with_context(|| format!("Failed to open file: {}", resolved.display()))?;
    let mmap = unsafe { Mmap::map(&file)? };

    let keywords: Vec<String> = match custom_keywords {
        Some(kws) if !kws.is_empty() => kws,
        _ => DEFAULT_FORENSIC_KEYWORDS
            .iter()
            .map(|s| s.to_string())
            .collect(),
    };

    log_println!(
        "================================================================================"
    );
    log_println!(" PANASONIC FORENSIC ZERO-COPY BINARY SCANNER");
    log_println!(
        "================================================================================"
    );
    log_println!(" Target File     : {}", resolved.display());
    log_println!(" File Size       : {} bytes", mmap.len());
    log_println!(" Min String Len  : {} chars", min_length);
    log_println!(
        " Target Keywords : {}",
        if keywords.is_empty() {
            "NONE (all strings)".to_string()
        } else {
            keywords.join(", ")
        }
    );
    log_println!(
        "--------------------------------------------------------------------------------"
    );

    let (total_strings, matches) = scan_data_strings(&mmap, min_length, &keywords, limit);

    log_println!(
        "Total valid strings extracted: {} | Matching keywords: {}",
        total_strings,
        matches.len()
    );

    for m in &matches {
        let kw_info = match &m.matched_keyword {
            Some(kw) => format!(" (Matched: '{}')", kw),
            None => String::new(),
        };
        log_println!("  [{}] {}{}", m.offset_hex, m.text, kw_info);
    }

    if limit > 0 && matches.len() >= limit {
        log_println!(
            "  ... [Output limited to {} matches. Use --limit 0 for all] ...",
            limit
        );
    }

    log_println!(
        "================================================================================"
    );

    let report = ScanReport {
        file_path: resolved.to_string_lossy().to_string(),
        file_size_bytes: mmap.len(),
        min_length,
        total_strings_found: total_strings,
        total_keyword_matches: matches.len(),
        matches,
    };

    if let Some(jpath) = json_path {
        let json_text = serde_json::to_string_pretty(&report)?;
        std::fs::write(jpath, json_text)?;
        log_println!("[+] Report written to JSON: {}", jpath.display());
    }

    Ok(report)
}

/// Convenience scanner that runs against both HDD and FRM if available.
pub fn scan_all_default(min_length: usize, limit: usize) -> Result<()> {
    let hdd_path = PathBuf::from("firmware_and_images/panasonic DMR EH-55 Firmware hdd_100.bin");
    let frm_path = PathBuf::from("firmware_and_images/PANA_DVD.FRM");

    if hdd_path.exists() {
        log_println!("\n>>> Scanning HDD Dump Image <<<");
        let _ = run_deep_scan(&hdd_path, min_length, None, limit, None);
    }

    if frm_path.exists() {
        log_println!("\n>>> Scanning Optical Container PANA_DVD.FRM <<<");
        let _ = run_deep_scan(&frm_path, min_length, None, limit, None);
    }

    Ok(())
}
