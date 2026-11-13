//! Workspace path resolution and forensic input helpers shared across MCP handlers.

use anyhow::Result;
pub use panadiga_core::constants::resolve_workspace_root;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

pub fn resolve_workspace_path(root: &Path, raw: &str) -> PathBuf {
    let candidate = Path::new(raw);
    if candidate.is_absolute() {
        candidate.to_path_buf()
    } else {
        root.join(candidate)
    }
}

pub fn paths_refer_to_same_file(a: &Path, b: &Path) -> bool {
    if a == b {
        return true;
    }
    if let (Ok(ca), Ok(cb)) = (fs::canonicalize(a), fs::canonicalize(b)) {
        return ca == cb;
    }
    if let (Ok(ca), Some(b_parent), Some(b_name)) = (fs::canonicalize(a), b.parent(), b.file_name())
        && let Ok(cp) = fs::canonicalize(b_parent)
    {
        return ca == cp.join(b_name);
    }
    false
}

pub fn parse_u64_hex_or_dec(raw: &str) -> std::result::Result<u64, String> {
    let s = raw.trim();
    if let Some(hex_body) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        u64::from_str_radix(hex_body, 16)
            .map_err(|_| format!("Invalid hexadecimal value format: {}", raw))
    } else {
        s.parse::<u64>()
            .map_err(|_| format!("Invalid integer/hex format: {}", raw))
    }
}

pub fn require_non_empty_str<'a>(args: &'a Value, key: &str) -> Result<&'a str> {
    args.get(key)
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| anyhow::anyhow!("Missing required argument: {}", key))
}

pub fn optional_non_empty_str<'a>(args: &'a Value, key: &str) -> Option<&'a str> {
    args.get(key)
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
}
