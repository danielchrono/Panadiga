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

/// Searches all PDFs in a directory for specific terms.
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

    println!("================================================================================");
    println!(" PANASONIC DMR-EH55 SERVICE MANUALS & SCHEMATICS INDEXER");
    println!("================================================================================");
    println!(" Directory : {}", manuals_dir.display());
    println!(" Terms     : {}", terms.join(", "));
    println!("--------------------------------------------------------------------------------");

    if !manuals_dir.exists() {
        anyhow::bail!("Manuals directory not found: {}", manuals_dir.display());
    }

    let mut pdf_files = Vec::new();
    for entry in fs::read_dir(manuals_dir)? {
        let entry = entry?;
        let p = entry.path();
        if p.is_file() {
            if let Some(ext) = p.extension() {
                if ext.to_string_lossy().eq_ignore_ascii_case("pdf") {
                    if let Some(filt) = filter_filename {
                        if !p.file_name().unwrap().to_string_lossy().to_lowercase().contains(&filt.to_lowercase()) {
                            continue;
                        }
                    }
                    pdf_files.push(p);
                }
            }
        }
    }

    pdf_files.sort();

    if pdf_files.is_empty() {
        println!("[!] No PDF documents matched the criteria in {}", manuals_dir.display());
        return Ok(Vec::new());
    }

    let mut all_results = Vec::new();

    for pdf_path in &pdf_files {
        let file_name = pdf_path.file_name().unwrap().to_string_lossy().to_string();
        println!("\n[*] Indexing PDF: {}", file_name);

        let doc = match Document::load(pdf_path) {
            Ok(d) => d,
            Err(e) => {
                println!("    [!] Could not parse PDF: {}", e);
                continue;
            }
        };

        let pages = doc.get_pages();
        let total_pages = pages.len();
        println!("    Total Pages: {}", total_pages);

        let mut term_matches: BTreeMap<String, Vec<u32>> = BTreeMap::new();
        for t in &terms {
            term_matches.insert(t.clone(), Vec::new());
        }

        for (&page_num, _) in &pages {
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
                let preview: Vec<String> = pages_found.iter().take(10).map(|p| p.to_string()).collect();
                let more = if pages_found.len() > 10 { "..." } else { "" };
                println!(
                    "    [{}] {} occurrences on pages: {}{}",
                    t,
                    pages_found.len(),
                    preview.join(", "),
                    more
                );
            }
        }

        if found_count == 0 {
            println!("    (No target terms found in text stream)");
        }

        all_results.push(PdfSearchResult {
            file_name,
            file_path: pdf_path.to_string_lossy().to_string(),
            total_pages,
            matches: term_matches,
        });
    }

    println!("\n================================================================================");
    Ok(all_results)
}
