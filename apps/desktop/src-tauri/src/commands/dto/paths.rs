//! Workspace file path resolution helpers for desktop IPC commands.

use panadiga_core::constants::{find_workspace_subdir, resolve_workspace_root};
use panadiga_core::ssot::is_valid_nor_flash_size;
use std::path::{Path, PathBuf};

/// Resolves the first existing file path from a candidate list, or returns `fallback`.
/// Probes CWD, parent dirs, workspace root, and executable-relative dirs.
pub fn resolve_candidate(candidates: &[&str], fallback: &str) -> String {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()));
    let ws_root = resolve_workspace_root();

    for &c in candidates {
        let p = Path::new(c);
        if p.exists() {
            return c.to_string();
        }

        let ws_rel = ws_root.join(c);
        if ws_rel.exists() {
            return ws_rel.to_string_lossy().to_string();
        }

        // Try one and two levels up (e.g. running from dist/ or apps/desktop/)
        let p1 = Path::new("..").join(c);
        if p1.exists() {
            return p1.to_string_lossy().to_string();
        }
        let p2 = Path::new("../..").join(c);
        if p2.exists() {
            return p2.to_string_lossy().to_string();
        }
        let p3 = Path::new("../../..").join(c);
        if p3.exists() {
            return p3.to_string_lossy().to_string();
        }

        if let Some(ref ed) = exe_dir {
            let exe_rel = ed.join(c);
            if exe_rel.exists() {
                return exe_rel.to_string_lossy().to_string();
            }
            if let Some(parent) = ed.parent() {
                let parent_rel = parent.join(c);
                if parent_rel.exists() {
                    return parent_rel.to_string_lossy().to_string();
                }
            }
        }
    }
    fallback.to_string()
}

fn find_nor_flash_in_dir(dir: &Path, depth: u8) -> Option<PathBuf> {
    let entries = std::fs::read_dir(dir).ok()?;
    let mut subdirs = Vec::new();
    let mut secondary_match = None;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if depth == 0 {
                let is_ignored = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| {
                        matches!(
                            n.to_ascii_lowercase().as_str(),
                            "bkp" | "extracted_partitions"
                        )
                    })
                    .unwrap_or(false);
                if !is_ignored {
                    subdirs.push(path);
                }
            }
            continue;
        }
        if !path.is_file() {
            continue;
        }
        let ext_ok = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| matches!(e.to_ascii_lowercase().as_str(), "bin" | "rom" | "dump"))
            .unwrap_or(false);
        if !ext_ok {
            continue;
        }
        let Ok(meta) = entry.metadata() else {
            continue;
        };
        if !is_valid_nor_flash_size(meta.len()) {
            continue;
        }
        let name_lower = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        if name_lower.contains("clean") || name_lower.contains("recovered") {
            if secondary_match.is_none() {
                secondary_match = Some(path);
            }
        } else {
            return Some(path);
        }
    }

    if let Some(found) = secondary_match {
        return Some(found);
    }

    subdirs.sort();
    for sub in subdirs {
        if let Some(found) = find_nor_flash_in_dir(&sub, depth + 1) {
            return Some(found);
        }
    }
    None
}

/// Automatically scans workspace `firmware_and_images` directories (and model subdirectories,
/// excluding `bkp/`) for any valid NOR Flash dump (2MB, 4MB, 8MB, or 16MB), falling back to
/// the canonical expected physical Spansion target path `firmware_and_images/IC5001.bin`.
pub fn resolve_default_flash_path(input_path: Option<String>) -> String {
    if let Some(custom) = input_path {
        let trimmed = custom.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }

    let mut firmware_roots = vec![
        PathBuf::from("firmware_and_images"),
        PathBuf::from("../firmware_and_images"),
        PathBuf::from("../../firmware_and_images"),
        PathBuf::from("../../../firmware_and_images"),
    ];
    if let Some(found_fw_dir) = find_workspace_subdir("firmware_and_images") {
        firmware_roots.insert(0, found_fw_dir);
    }

    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            firmware_roots.push(exe_dir.join("firmware_and_images"));
            if let Some(parent) = exe_dir.parent() {
                firmware_roots.push(parent.join("firmware_and_images"));
            }
        }
    }

    for root in firmware_roots {
        if let Some(found) = find_nor_flash_in_dir(&root, 0) {
            return found.to_string_lossy().to_string();
        }
    }

    "firmware_and_images/IC5001.bin".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn find_nor_flash_ignores_bkp_and_prefers_primary_dump() {
        let tmp = std::env::temp_dir().join("panadiga_paths_bkp_test");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(tmp.join("bkp")).unwrap();
        fs::create_dir_all(tmp.join("dmr-test")).unwrap();

        let mock_bkp = vec![0xFFu8; 8 * 1024 * 1024];
        fs::write(tmp.join("bkp").join("mock_dump.bin"), &mock_bkp).unwrap();

        assert!(find_nor_flash_in_dir(&tmp, 0).is_none());

        let real_4mb = vec![0xAAu8; 4 * 1024 * 1024];
        let expected = tmp.join("dmr-test").join("real_dump_4mb.bin");
        fs::write(
            tmp.join("dmr-test").join("real_dump_recovered.bin"),
            &real_4mb,
        )
        .unwrap();
        fs::write(&expected, &real_4mb).unwrap();

        assert_eq!(find_nor_flash_in_dir(&tmp, 0), Some(expected));
        let _ = fs::remove_dir_all(&tmp);
    }
}
