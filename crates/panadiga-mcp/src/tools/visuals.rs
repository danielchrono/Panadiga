//! Curated reverse-engineering visual catalog search (`docs/assets/visual_manifest.json`).

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::fs;
use std::path::Path;

use crate::workspace::optional_non_empty_str;

const VISUAL_MANIFEST_REL_PATH: &str = "docs/assets/visual_manifest.json";

#[derive(Debug, Deserialize)]
struct VisualManifest {
    unique_valid_pngs: usize,
    total_mb: f64,
    images: Vec<VisualEntry>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VisualEntry {
    pub id: String,
    pub filename: String,
    pub repo_path: String,
    pub source_rel: String,
    pub aliases: Vec<String>,
    pub sha256: String,
    pub bytes: u64,
    pub width: u32,
    pub height: u32,
    pub manual_edition: String,
    pub page_number: Option<u32>,
    pub category: String,
    pub tags: Vec<String>,
    pub caption: String,
}

fn entry_searchable_blob(entry: &VisualEntry) -> String {
    let page_str = entry
        .page_number
        .map(|p| format!("p{} pag {} page {}", p, p, p))
        .unwrap_or_default();
    format!(
        "{} {} {} {} {} {} {} {} {}",
        entry.id,
        entry.filename,
        entry.repo_path,
        entry.aliases.join(" "),
        entry.manual_edition,
        page_str,
        entry.category,
        entry.tags.join(" "),
        entry.caption
    )
    .to_lowercase()
}

pub fn load_visual_manifest_raw(workspace_root: &Path) -> Result<String, String> {
    let direct = workspace_root.join(VISUAL_MANIFEST_REL_PATH);
    let manifest_path = if direct.is_file() {
        direct
    } else {
        crate::workspace::resolve_workspace_root().join(VISUAL_MANIFEST_REL_PATH)
    };
    fs::read_to_string(&manifest_path)
        .map_err(|e| format!("Failed to read {}: {}", manifest_path.display(), e))
}

pub fn handle_search_visuals(args: &Value, workspace_root: &Path) -> Value {
    let raw = match load_visual_manifest_raw(workspace_root) {
        Ok(s) => s,
        Err(err) => return json!({ "error": err }),
    };
    let manifest: VisualManifest = match serde_json::from_str(&raw) {
        Ok(m) => m,
        Err(e) => {
            return json!({ "error": format!("Invalid visual_manifest.json: {}", e) });
        }
    };

    let query = optional_non_empty_str(args, "query").unwrap_or("");
    let edition_filter = optional_non_empty_str(args, "edition").map(str::to_lowercase);
    let category_filter = optional_non_empty_str(args, "category").map(str::to_lowercase);
    let page_filter = args.get("page").and_then(|v| v.as_u64()).map(|p| p as u32);
    let limit = args
        .get("limit")
        .and_then(|v| v.as_u64())
        .unwrap_or(15)
        .clamp(1, 50) as usize;

    let tokens: Vec<String> = query
        .split_whitespace()
        .map(str::to_lowercase)
        .filter(|t| !t.is_empty())
        .collect();

    let mut scored: Vec<(usize, &VisualEntry)> = manifest
        .images
        .iter()
        .filter(|entry| {
            if let Some(ref ed) = edition_filter
                && !entry.manual_edition.to_lowercase().contains(ed)
            {
                return false;
            }
            if let Some(ref cat) = category_filter
                && !entry.category.to_lowercase().contains(cat)
            {
                return false;
            }
            if let Some(pg) = page_filter
                && entry.page_number != Some(pg)
            {
                return false;
            }
            true
        })
        .filter_map(|entry| {
            if tokens.is_empty() {
                return Some((1usize, entry));
            }
            let blob = entry_searchable_blob(entry);
            let hits = tokens
                .iter()
                .filter(|tok| blob.contains(tok.as_str()))
                .count();
            if hits == 0 {
                return None;
            }
            // Prefer entries matching all query tokens, plus exact filename boost.
            let bonus = usize::from(hits == tokens.len()) * 10
                + usize::from(
                    entry
                        .filename
                        .to_lowercase()
                        .contains(&query.to_lowercase()),
                ) * 5;
            Some((hits + bonus, entry))
        })
        .collect();

    scored.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.filename.cmp(&b.1.filename)));
    let matched_total = scored.len();
    let results: Vec<&VisualEntry> = scored.into_iter().take(limit).map(|(_, e)| e).collect();

    json!({
        "manifest_path": VISUAL_MANIFEST_REL_PATH,
        "total_curated": manifest.unique_valid_pngs,
        "total_mb": manifest.total_mb,
        "matched": matched_total,
        "returned": results.len(),
        "results": results
    })
}
