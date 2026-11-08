//! Panasonic Service Manual & Schematic PDF Search Engine
//!
//! Fast indexing and technical keyword lookup across all official Panasonic DMR-EH55
//! service manuals, operating guides, and schematics.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use anyhow::Result;
use lopdf::Document;

/// Default diagnostic keywords looked up in service manuals.
pub const DEFAULT_MANUAL_SEARCH_TERMS: &[&str] = &[
    "S29JL064",
    "MN2DS",
    "MN103",
    "CN55001",
    "IC55007",
    "IC5001",
    "IC5002",
    "FIRMWARE",
    "FLASH",
    "UPDATE",
    "SELF-CHECK",
    "U99",
    "HDD FORMAT",
    "SERVICE MODE",
    "RAM DISK",
    "EEPROM",
    "NVRAM",
    "VEP79138",
    "D_RESET",
];

/// Result of searching a single PDF document.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PdfSearchResult {
    pub file_name: String,
    pub file_path: String,
    pub total_pages: usize,
    pub matches: BTreeMap<String, Vec<u32>>,
}

fn collect_pdf_files(
    dir: &Path,
    filter: Option<&str>,
    files: &mut Vec<std::path::PathBuf>,
) -> Result<()> {
    if !dir.is_dir() {
        return Ok(());
    }
    let mut entries: Vec<_> = fs::read_dir(dir)?.filter_map(|e| e.ok()).collect();
    entries.sort_by_key(|e| e.path());

    for entry in entries {
        let p = entry.path();
        if p.is_dir() {
            collect_pdf_files(&p, filter, files)?;
        } else if p.is_file() {
            if let Some(ext) = p.extension() {
                if ext.to_string_lossy().eq_ignore_ascii_case("pdf") {
                    if let Some(filt) = filter {
                        let path_str = p.to_string_lossy().to_lowercase();
                        let norm_path = path_str.replace('_', "-");
                        let norm_filt = filt.to_lowercase().replace('_', "-");
                        if !path_str.contains(&filt.to_lowercase())
                            && !norm_path.contains(&norm_filt)
                        {
                            continue;
                        }
                    }
                    files.push(p);
                }
            }
        }
    }
    Ok(())
}

/// Searches all PDFs in a directory (recursively) for specific terms.
pub fn search_manuals(
    manuals_dir: &Path,
    custom_terms: Option<Vec<String>>,
    filter_filename: Option<&str>,
) -> Result<Vec<PdfSearchResult>> {
    let terms: Vec<String> = match custom_terms {
        Some(t) if !t.is_empty() => t,
        _ => DEFAULT_MANUAL_SEARCH_TERMS
            .iter()
            .map(|s| s.to_string())
            .collect(),
    };

    log_println!(
        "================================================================================"
    );
    log_println!(" PANASONIC DMR-EH55 SERVICE MANUALS & SCHEMATICS INDEXER");
    log_println!(
        "================================================================================"
    );
    log_println!(" Directory : {}", manuals_dir.display());
    log_println!(" Terms     : {}", terms.join(", "));
    log_println!(
        "--------------------------------------------------------------------------------"
    );

    if !manuals_dir.exists() {
        anyhow::bail!("Manuals directory not found: {}", manuals_dir.display());
    }

    let mut pdf_files = Vec::new();
    collect_pdf_files(manuals_dir, filter_filename, &mut pdf_files)?;
    pdf_files.sort();

    if pdf_files.is_empty() {
        log_println!(
            "[!] No PDF documents matched the criteria in {}",
            manuals_dir.display()
        );
        return Ok(Vec::new());
    }

    let mut all_results = Vec::new();

    for pdf_path in &pdf_files {
        let file_name = pdf_path.file_name().unwrap().to_string_lossy().to_string();
        log_println!("\n[*] Indexing PDF: {}", file_name);

        let doc = match Document::load(pdf_path) {
            Ok(d) => d,
            Err(e) => {
                log_println!("    [!] Could not parse PDF: {}", e);
                continue;
            }
        };

        let pages = doc.get_pages();
        let total_pages = pages.len();
        log_println!("    Total Pages: {}", total_pages);

        let mut term_matches: BTreeMap<String, Vec<u32>> = BTreeMap::new();
        for t in &terms {
            term_matches.insert(t.clone(), Vec::new());
        }

        for &page_num in pages.keys() {
            if let Ok(text) = doc.extract_text(&[page_num]) {
                let text_upper = text.to_uppercase();
                for t in &terms {
                    let term_upper = t.to_uppercase();
                    if text_upper.contains(&term_upper) {
                        term_matches.get_mut(t).unwrap().push(page_num);
                    }
                }
            }
        }

        let mut found_count = 0;
        for (t, pages_found) in &term_matches {
            if !pages_found.is_empty() {
                found_count += 1;
                let preview: Vec<String> =
                    pages_found.iter().take(10).map(|p| p.to_string()).collect();
                let more = if pages_found.len() > 10 { "..." } else { "" };
                log_println!(
                    "    [{}] {} occurrences on pages: {}{}",
                    t,
                    pages_found.len(),
                    preview.join(", "),
                    more
                );
            }
        }

        if found_count == 0 {
            log_println!("    (No target terms found in text stream)");
        }

        all_results.push(PdfSearchResult {
            file_name,
            file_path: pdf_path.to_string_lossy().to_string(),
            total_pages,
            matches: term_matches,
        });
    }

    log_println!(
        "\n================================================================================"
    );
    Ok(all_results)
}
