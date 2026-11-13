//! Service manual & datasheet PDF page extractor and cache for the Panadiga RAG Engine.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{LazyLock, Mutex};
use std::time::UNIX_EPOCH;

use super::super::guardrails::QueryGuard;
use super::super::models::{Chunk, SubsystemType};
use super::markdown::flush_lines_chunk;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CachedPdfEntry {
    file_len: u64,
    modified_secs: u64,
    chunks: Vec<Chunk>,
}

static PDF_MEMORY_CACHE: LazyLock<Mutex<HashMap<String, CachedPdfEntry>>> =
    LazyLock::new(|| Mutex::new(load_disk_cache()));

/// Serializes concurrent PDF extraction across threads so each manual is parsed at most once.
static PDF_EXTRACT_LOCK: Mutex<()> = Mutex::new(());
static PDF_CACHE_DIRTY: AtomicBool = AtomicBool::new(false);

pub fn flush_disk_cache() {
    if PDF_CACHE_DIRTY.swap(false, Ordering::SeqCst) {
        let cache = PDF_MEMORY_CACHE.lock().unwrap_or_else(|e| e.into_inner());
        persist_disk_cache(&cache);
    }
}

fn cache_file_path() -> PathBuf {
    std::env::temp_dir().join("panadiga_rag_pdf_cache_v1.json")
}

fn load_disk_cache() -> HashMap<String, CachedPdfEntry> {
    let path = cache_file_path();
    let Ok(bytes) = fs::read(path) else {
        return HashMap::new();
    };
    serde_json::from_slice(&bytes).unwrap_or_default()
}

fn persist_disk_cache(cache: &HashMap<String, CachedPdfEntry>) {
    if let Ok(json) = serde_json::to_vec(cache) {
        let _ = fs::write(cache_file_path(), json);
    }
}

fn file_signature(path: &Path) -> (String, u64, u64) {
    let canon = fs::canonicalize(path)
        .unwrap_or_else(|_| path.to_path_buf())
        .to_string_lossy()
        .to_string();
    let meta = fs::metadata(path).ok();
    let len = meta.as_ref().map(|m| m.len()).unwrap_or(0);
    let mtime = meta
        .and_then(|m| m.modified().ok())
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or(0);
    (canon, len, mtime)
}

fn is_readable_technical_text(text: &str) -> bool {
    let mut alnum = 0usize;
    let mut printable = 0usize;
    let mut total = 0usize;

    for ch in text.chars() {
        total += 1;
        if ch.is_ascii_alphanumeric() {
            alnum += 1;
        }
        if !ch.is_control() || ch == '\n' || ch == '\r' || ch == '\t' {
            printable += 1;
        }
    }

    alnum >= 40 && total > 0 && (printable * 100 / total) >= 80
}

fn lookup_cached_chunks(key: &str, len: u64, mtime: u64) -> Option<Vec<Chunk>> {
    let cache = PDF_MEMORY_CACHE.lock().unwrap_or_else(|e| e.into_inner());
    cache
        .get(key)
        .filter(|entry| entry.file_len == len && entry.modified_secs == mtime)
        .map(|entry| entry.chunks.clone())
}

pub fn chunk_pdf_file(
    guard: &QueryGuard,
    subsystem: SubsystemType,
    file_path: &Path,
) -> Vec<Chunk> {
    let (key, len, mtime) = file_signature(file_path);

    if let Some(chunks) = lookup_cached_chunks(&key, len, mtime) {
        return chunks;
    }

    let _extract_guard = PDF_EXTRACT_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(chunks) = lookup_cached_chunks(&key, len, mtime) {
        return chunks;
    }

    let Ok(doc) = lopdf::Document::load(file_path) else {
        return Vec::new();
    };

    let doc_title = file_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Service Manual PDF")
        .replace('_', " ");

    let pages = doc.get_pages();
    let mut page_nums: Vec<u32> = pages.keys().copied().collect();
    page_nums.sort_unstable();

    let mut chunks = Vec::new();
    for page_num in page_nums {
        let Ok(raw_text) = doc.extract_text(&[page_num]) else {
            continue;
        };
        if !is_readable_technical_text(&raw_text) {
            continue;
        }

        let section = format!("Page {}", page_num);
        let hierarchy = vec![doc_title.clone(), section.clone()];
        let mut line_buf: Vec<(usize, String)> = Vec::new();
        let page_idx = page_num as usize;

        for raw_line in raw_text.lines() {
            let trimmed = raw_line.split_whitespace().collect::<Vec<&str>>().join(" ");
            if trimmed.len() < 3 {
                continue;
            }
            line_buf.push((page_idx, trimmed));
            if line_buf.len() >= 45 {
                flush_lines_chunk(
                    &mut chunks,
                    guard,
                    file_path,
                    &doc_title,
                    subsystem,
                    &section,
                    &hierarchy,
                    &mut line_buf,
                    page_idx,
                );
            }
        }

        if !line_buf.is_empty() {
            flush_lines_chunk(
                &mut chunks,
                guard,
                file_path,
                &doc_title,
                subsystem,
                &section,
                &hierarchy,
                &mut line_buf,
                page_idx,
            );
        }
    }

    let mut cache = PDF_MEMORY_CACHE.lock().unwrap_or_else(|e| e.into_inner());
    cache.insert(
        key,
        CachedPdfEntry {
            file_len: len,
            modified_secs: mtime,
            chunks: chunks.clone(),
        },
    );
    PDF_CACHE_DIRTY.store(true, Ordering::SeqCst);
    chunks
}
