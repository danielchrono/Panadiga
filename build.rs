use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::SystemTime;

fn main() {
    ensure_frontend_built();

    println!("cargo:rerun-if-changed=apps/desktop/src");
    println!("cargo:rerun-if-changed=apps/desktop/package.json");
    println!("cargo:rerun-if-changed=tauri.conf.json");

    #[cfg(windows)]
    {
        let attrs = tauri_build::Attributes::new()
            .windows_attributes(tauri_build::WindowsAttributes::new_without_app_manifest());
        tauri_build::try_build(attrs).expect("failed to run tauri-build");
    }
    #[cfg(not(windows))]
    {
        tauri_build::build();
    }
}

/// Automatically compiles the React / TypeScript frontend before packaging the native binary
/// if apps/desktop/dist is missing or if sources in apps/desktop/src have been modified.
fn ensure_frontend_built() {
    let dist_index = Path::new("apps/desktop/dist/index.html");
    let mut needs_build = !dist_index.exists();

    if !needs_build {
        if let Ok(meta) = dist_index.metadata() {
            if let Ok(dist_time) = meta.modified() {
                if is_src_newer(dist_time) {
                    needs_build = true;
                }
            }
        }
    }

    if needs_build {
        println!("cargo:warning=[PANADIGA] Building React/TypeScript frontend (apps/desktop)...");
        let npm_cmd = if cfg!(windows) { "npm.cmd" } else { "npm" };
        let status = Command::new(npm_cmd)
            .args(["--prefix", "apps/desktop", "run", "build"])
            .status()
            .expect("Failed to execute npm run build in apps/desktop");

        if !status.success() {
            panic!(
                "[PANADIGA] Frontend build failed with exit code: {}",
                status
            );
        }
    }
}

fn is_src_newer(dist_time: SystemTime) -> bool {
    let src_dir = Path::new("apps/desktop/src");
    if !src_dir.exists() {
        return false;
    }
    check_dir_newer(src_dir, dist_time)
}

fn check_dir_newer(dir: &Path, threshold: SystemTime) -> bool {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if check_dir_newer(&path, threshold) {
                    return true;
                }
            } else if let Ok(meta) = path.metadata() {
                if let Ok(modified) = meta.modified() {
                    if modified > threshold {
                        return true;
                    }
                }
            }
        }
    }
    false
}
