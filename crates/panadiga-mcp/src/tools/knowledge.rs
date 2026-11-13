//! Knowledge retrieval MCP tools: PDF manual page search, Zero-Error RAG queries, codebase search, and Ghidra symbols.

use anyhow::Result;
use panadiga_core::constants::{AM34_SYSTEM_MEMORY_REGIONS, DEFAULT_MANUALS_DIR};
use panadiga_core::ghidra::{generate_ghidra_memory_map, generate_ghidra_symbols};
use panadiga_core::manuals::list_pdf_files;
use panadiga_rag::ChunkSourceFilter;
use serde_json::{Value, json};
use std::path::Path;

use crate::workspace::{optional_non_empty_str, require_non_empty_str};

pub fn handle_search_manuals(args: &Value, workspace_root: &Path) -> Result<Value> {
    let query_raw = require_non_empty_str(args, "query")?;
    let query_lower = query_raw.to_lowercase();
    let max_hits = args
        .get("max_results")
        .and_then(|v| v.as_u64())
        .unwrap_or(8)
        .clamp(1, 25) as usize;

    let manuals_dir = workspace_root.join(DEFAULT_MANUALS_DIR);
    let all_pdfs = list_pdf_files(&manuals_dir, None).unwrap_or_default();
    let total = all_pdfs.len();

    let norm_query = query_lower.replace('_', "-");
    let filename_matches: Vec<Value> = all_pdfs
        .into_iter()
        .filter(|p| {
            let name = p
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_lowercase();
            name.contains(&query_lower) || name.replace('_', "-").contains(&norm_query)
        })
        .map(|p| {
            let rel = p
                .strip_prefix(workspace_root)
                .map(|r| r.to_string_lossy().to_string())
                .unwrap_or_else(|_| p.to_string_lossy().to_string());
            let fname = p
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string();
            json!({
                "file": fname,
                "path": rel,
                "match_type": "filename"
            })
        })
        .collect();

    let pdf_hits: Vec<Value> =
        panadiga_rag::retrieve_filtered(query_raw, ChunkSourceFilter::PdfManuals, max_hits)
            .into_iter()
            .map(|res| {
                let excerpt: String = res.chunk.text.chars().take(420).collect();
                json!({
                    "file": res.chunk.metadata.doc_title,
                    "path": res.chunk.metadata.file_path,
                    "page": res.chunk.metadata.line_start,
                    "section": res.chunk.metadata.section_title,
                    "score": (res.score * 1000.0).round() / 1000.0,
                    "excerpt": excerpt,
                })
            })
            .collect();

    Ok(json!({
        "query": query_lower,
        "direct_matches": filename_matches,
        "pdf_page_matches": pdf_hits,
        "total_manuals_in_catalog": total,
    }))
}

pub fn handle_rag_query(args: &Value) -> Result<Value> {
    let query_str = require_non_empty_str(args, "query")?;
    let include_stats = args
        .get("include_corpus_stats")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let dto = panadiga_rag::query(query_str).to_query_result();
    let stats = if include_stats {
        serde_json::to_value(panadiga_rag::corpus_stats()).unwrap_or(Value::Null)
    } else {
        Value::Null
    };

    Ok(json!({
        "query": dto.query,
        "answer": dto.answer,
        "confidence": dto.confidence,
        "groundedness_score": dto.groundedness_score,
        "safety_passed": dto.safety.is_safe,
        "is_abstained": dto.is_abstained,
        "citations": dto.citations,
        "corpus_stats": stats,
    }))
}

pub fn handle_search_code(args: &Value) -> Result<Value> {
    let query_str = require_non_empty_str(args, "query")?;
    let scope = optional_non_empty_str(args, "scope").unwrap_or("code");
    let limit = args
        .get("limit")
        .and_then(|v| v.as_u64())
        .unwrap_or(8)
        .clamp(1, 25) as usize;

    let filter = match scope.to_lowercase().as_str() {
        "ghidra" => ChunkSourceFilter::Ghidra,
        "docs" => ChunkSourceFilter::Documentation,
        "all" => ChunkSourceFilter::All,
        _ => ChunkSourceFilter::SourceCode,
    };

    let matches: Vec<Value> = panadiga_rag::retrieve_filtered(query_str, filter, limit)
        .into_iter()
        .map(|res| {
            let excerpt: String = res.chunk.text.chars().take(480).collect();
            json!({
                "file": res.chunk.metadata.file_path,
                "symbol": res.chunk.metadata.section_title,
                "lines": format!("{}-{}", res.chunk.metadata.line_start, res.chunk.metadata.line_end),
                "subsystem": format!("{:?}", res.chunk.metadata.subsystem),
                "score": (res.score * 1000.0).round() / 1000.0,
                "excerpt": excerpt,
            })
        })
        .collect();

    Ok(json!({
        "query": query_str,
        "scope": scope,
        "matches": matches,
    }))
}

pub fn handle_ghidra_symbols(args: &Value) -> Value {
    let filter_lower = optional_non_empty_str(args, "filter").map(|s| s.to_lowercase());
    let symbols_script = generate_ghidra_symbols();
    let map_script = generate_ghidra_memory_map();

    let symbol_lines: Vec<&str> = symbols_script
        .lines()
        .filter(|l| l.contains("createLabel") || l.contains("0x"))
        .filter(|l| match &filter_lower {
            Some(f) => l.to_lowercase().contains(f),
            None => true,
        })
        .collect();

    let blocks: Vec<Value> = AM34_SYSTEM_MEMORY_REGIONS
        .iter()
        .filter(|r| match &filter_lower {
            Some(f) => r.name.to_lowercase().contains(f) || r.perms.to_lowercase().contains(f),
            None => true,
        })
        .map(|r| r.to_json_value())
        .collect();

    json!({
        "architecture": "Matsushita AM34 / MN103S (UniPhier)",
        "memory_blocks": blocks,
        "ghidra_symbol_directives": symbol_lines,
        "memory_map_script_lines": map_script.lines().count(),
        "symbols_script_lines": symbols_script.lines().count(),
    })
}
