//! Multi-language source code, RP2040 C11/PIO firmware, and Ghidra Python indexer.

use std::fs;
use std::path::Path;

use super::super::guardrails::QueryGuard;
use super::super::models::{Chunk, SubsystemType};
use super::markdown::flush_lines_chunk;

const MAX_SOURCE_BYTES: u64 = 256 * 1024;

pub fn is_supported_source_file(path: &Path) -> bool {
    let fname = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
    let path_lower = path.to_string_lossy().to_lowercase();
    if fname == "tests.rs"
        || fname.ends_with("_test.rs")
        || fname.ends_with(".test.ts")
        || fname.ends_with(".test.tsx")
        || fname.ends_with(".spec.ts")
        || fname.starts_with("mock")
        || path_lower.contains("\\tests\\")
        || path_lower.contains("/tests/")
        || path_lower.contains("\\test\\")
        || path_lower.contains("/test/")
    {
        return false;
    }

    matches!(
        path.extension().and_then(|s| s.to_str()),
        Some("rs" | "c" | "h" | "pio" | "ts" | "tsx" | "py")
    )
}

fn detect_symbol_section(trimmed: &str) -> Option<String> {
    let prefixes = [
        "pub fn ",
        "fn ",
        "pub struct ",
        "struct ",
        "pub enum ",
        "pub const ",
        "#[tauri::command]",
        "export function ",
        "export async function ",
        "def ",
        "class ",
        ".program ",
        "void ",
        "uint",
        "int ",
        "bool ",
    ];
    if prefixes.iter().any(|p| trimmed.starts_with(p)) {
        let summary: String = trimmed.chars().take(72).collect();
        return Some(summary);
    }
    None
}

pub fn chunk_source_file(
    guard: &QueryGuard,
    subsystem: SubsystemType,
    file_path: &Path,
) -> Vec<Chunk> {
    if let Ok(meta) = fs::metadata(file_path)
        && meta.len() > MAX_SOURCE_BYTES
    {
        return Vec::new();
    }

    let Ok(content) = fs::read_to_string(file_path) else {
        return Vec::new();
    };

    let lines: Vec<&str> = content.lines().collect();
    let doc_title = file_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("source")
        .to_string();

    let mut chunks = Vec::new();
    let mut current_section = "Module Overview".to_string();
    let mut current_hierarchy = vec![doc_title.clone(), current_section.clone()];
    let mut current_lines: Vec<(usize, String)> = Vec::new();

    for (idx_zero, &line) in lines.iter().enumerate() {
        let line_idx = idx_zero + 1;
        let trimmed = line.trim();

        if let Some(new_sec) = detect_symbol_section(trimmed) {
            if current_lines.len() >= 12 {
                flush_lines_chunk(
                    &mut chunks,
                    guard,
                    file_path,
                    &doc_title,
                    subsystem,
                    &current_section,
                    &current_hierarchy,
                    &mut current_lines,
                    line_idx - 1,
                );
            }
            current_section = new_sec.clone();
            current_hierarchy = vec![doc_title.clone(), new_sec];
        }

        current_lines.push((line_idx, line.to_string()));
        if current_lines.len() >= 60 {
            flush_lines_chunk(
                &mut chunks,
                guard,
                file_path,
                &doc_title,
                subsystem,
                &current_section,
                &current_hierarchy,
                &mut current_lines,
                line_idx,
            );
        }
    }

    if !current_lines.is_empty() {
        flush_lines_chunk(
            &mut chunks,
            guard,
            file_path,
            &doc_title,
            subsystem,
            &current_section,
            &current_hierarchy,
            &mut current_lines,
            lines.len(),
        );
    }

    chunks
}
