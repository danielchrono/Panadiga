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
    "mn2ds",
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

/// Scans a byte slice for ASCII strings and filters against target keywords.
pub fn scan_data_strings(
    data: &[u8],
    min_length: usize,
    keywords: &[String],
    limit: usize,
) -> (usize, Vec<StringMatch>) {
    let mut matches = Vec::new();
    let mut total_found = 0;
    let mut current_start = None;

    let kw_lower: Vec<String> = keywords.iter().map(|k| k.to_lowercase()).collect();

    let evaluate_candidate = |start: usize, end: usize, total: &mut usize, acc: &mut Vec<StringMatch>| {
        let len = end - start;
        if len >= min_length {
            *total += 1;
            let slice = &data[start..end];
            if let Ok(text) = std::str::from_utf8(slice) {
                let text_lower = text.to_lowercase();
                let matched_kw = kw_lower.iter().find(|&kw| text_lower.contains(kw));

                if !kw_lower.is_empty() {
                    if let Some(kw) = matched_kw {
                        if limit == 0 || acc.len() < limit {
                            acc.push(StringMatch {
                                offset: start,
                                offset_hex: format!("0x{:08X}", start),
                                text: text.to_string(),
                                matched_keyword: Some(kw.clone()),
                            });
                        }
                    }
                } else if limit == 0 || acc.len() < limit {
                    acc.push(StringMatch {
                        offset: start,
                        offset_hex: format!("0x{:08X}", start),
                        text: text.to_string(),
                        matched_keyword: None,
                    });
                }
            }
        }
    };

    for (i, &b) in data.iter().enumerate() {
        let is_valid_char = b.is_ascii_alphanumeric()
            || b == b'_'
            || b == b'-'
            || b == b'.'
            || b == b'/'
            || b == b' '
            || b == b':';

        if is_valid_char {
            if current_start.is_none() {
                current_start = Some(i);
            }
        } else if let Some(start) = current_start {
            evaluate_candidate(start, i, &mut total_found, &mut matches);
            current_start = None;
        }
    }

    // Check tail
    if let Some(start) = current_start {
        evaluate_candidate(start, data.len(), &mut total_found, &mut matches);
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

    log_println!("================================================================================");
    log_println!(" PANASONIC FORENSIC ZERO-COPY BINARY SCANNER");
    log_println!("================================================================================");
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
    log_println!("--------------------------------------------------------------------------------");

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
        log_println!("  ... [Output limited to {} matches. Use --limit 0 for all] ...", limit);
    }

    log_println!("================================================================================");

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
