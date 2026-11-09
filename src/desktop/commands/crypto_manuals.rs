//! Panasonic DMR-EH55 DIGA Suite - Cryptanalysis & Manuals IPC Commands
//!
//! Single Responsibility: Tauri IPC command dispatchers for PANA_DVD.FRM 64-bit ECB
//! cryptanalysis, TOC inspection, KPA codebook generation, and PDF manual search.

use std::path::Path;

use super::dto::{
    CodebookReport, FrmEntropyReport, FrmTocDumpReport, ManualSearchResult, resolve_candidate,
};

/// Performs Shannon entropy calculation, cipher frequency analysis, and ECB leak detection.
#[tauri::command]
pub fn analyze_frm_command(frm_path: String) -> Result<FrmEntropyReport, String> {
    let hdd_candidates: Vec<&Path> = crate::constants::CANDIDATE_HDD_RAW_BACKUP_PATHS
        .iter()
        .map(Path::new)
        .collect();
    let hdd_path = hdd_candidates.iter().find(|p| p.exists()).copied();

    crate::crypto::analyze_frm_container(Path::new(&frm_path), hdd_path, 25, None)
        .map_err(|e| e.to_string())
}

/// Dumps and validates the Table of Contents (TOC) descriptors in PANA_DVD.FRM.
#[tauri::command]
pub fn dump_frm_toc_command(frm_path: String) -> Result<FrmTocDumpReport, String> {
    crate::crypto::dump_toc(Path::new(&frm_path), None).map_err(|e| e.to_string())
}

/// Computes the 64-bit ECB Known-Plaintext Attack (KPA) codebook dictionary.
#[tauri::command]
pub fn get_frm_codebook_command(
    frm_path: String,
    hdd_path: Option<String>,
) -> Result<CodebookReport, String> {
    let hdd_p = hdd_path.as_deref().map(Path::new);
    crate::crypto::print_codebook(Path::new(&frm_path), hdd_p, None).map_err(|e| e.to_string())
}

/// Searches official Panasonic DMR-EH55 service manuals and schematics for technical terms.
#[tauri::command]
pub fn search_manuals_command(term: String) -> Result<Vec<ManualSearchResult>, String> {
    let candidate_dirs: Vec<&Path> = crate::constants::CANDIDATE_MANUALS_DIRS
        .iter()
        .map(Path::new)
        .collect();
    let manuals_dir = candidate_dirs
        .iter()
        .find(|p| p.exists())
        .copied()
        .ok_or_else(|| "Manuals directory 'manuals_and_schematics' not found".to_string())?;

    crate::manuals::search_manuals(manuals_dir, Some(vec![term]), None).map_err(|e| e.to_string())
}

/// Frontend `analyze_frm` command.
#[tauri::command]
pub fn analyze_frm(frm_path: Option<String>) -> Result<FrmEntropyReport, String> {
    let path = frm_path.unwrap_or_else(|| {
        resolve_candidate(
            &[
                crate::constants::DEFAULT_FRM_CONTAINER_PATH,
                "../../firmware_and_images/PANA_DVD.FRM",
            ],
            crate::constants::DEFAULT_FRM_CONTAINER_PATH,
        )
    });
    analyze_frm_command(path)
}

/// Frontend `dump_frm_toc` command.
#[tauri::command]
pub fn dump_frm_toc(frm_path: Option<String>) -> Result<FrmTocDumpReport, String> {
    let path = frm_path.unwrap_or_else(|| {
        resolve_candidate(
            &[
                crate::constants::DEFAULT_FRM_CONTAINER_PATH,
                "../../firmware_and_images/PANA_DVD.FRM",
            ],
            crate::constants::DEFAULT_FRM_CONTAINER_PATH,
        )
    });
    dump_frm_toc_command(path)
}

/// Frontend `get_codebook` command.
#[tauri::command]
pub fn get_codebook(
    frm_path: Option<String>,
    hdd_path: Option<String>,
) -> Result<CodebookReport, String> {
    let path = frm_path.unwrap_or_else(|| {
        resolve_candidate(
            &[
                crate::constants::DEFAULT_FRM_CONTAINER_PATH,
                "../../firmware_and_images/PANA_DVD.FRM",
            ],
            crate::constants::DEFAULT_FRM_CONTAINER_PATH,
        )
    });
    get_frm_codebook_command(path, hdd_path)
}

/// Frontend `search_manuals` command accepting either `query` or `term`.
#[tauri::command]
pub fn search_manuals(
    query: Option<String>,
    term: Option<String>,
) -> Result<Vec<ManualSearchResult>, String> {
    let q = query.or(term).unwrap_or_else(|| "U99".to_string());
    search_manuals_command(q)
}
