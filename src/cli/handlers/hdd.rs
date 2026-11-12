//! CLI handlers for HDD MEIHDFS inspection, extraction, and physical drive operations.

use super::super::args::{HddActions, HddArgs};
use crate::constants::DEFAULT_HDD_IMAGE_PATH;
use crate::hdd::{decode_meihdfs_descriptors, extract_hdd_partitions, inspect_hdd_image};
use anyhow::Result;
use std::path::PathBuf;

fn resolve_hdd_path(image: Option<PathBuf>) -> PathBuf {
    image.unwrap_or_else(|| PathBuf::from(DEFAULT_HDD_IMAGE_PATH))
}

pub fn handle_hdd(args: HddArgs) -> Result<()> {
    match args.action {
        HddActions::Inspect {
            image,
            density,
            json,
        } => {
            let img_path = resolve_hdd_path(image);
            inspect_hdd_image(&img_path, density, json.as_deref())?;
        }
        HddActions::Extract {
            image,
            outdir,
            json,
        } => {
            let img_path = resolve_hdd_path(image);
            extract_hdd_partitions(&img_path, &outdir, json.as_deref())?;
        }
        HddActions::DecodeDescriptors {
            image,
            json,
            journal,
        } => {
            let img_path = resolve_hdd_path(image);
            decode_meihdfs_descriptors(&img_path, journal, json.as_deref())?;
        }
        HddActions::ListDrives => {
            let drives = crate::hdd::list_physical_drives()?;
            log_println!("====================================================================");
            log_println!("   PANADIGA STORAGE DISCOVERY & SYSTEM DRIVE LOCKOUT AUDIT");
            log_println!("====================================================================");
            if drives.is_empty() {
                log_println!("No physical block storage drives detected.");
            } else {
                for d in &drives {
                    let status = if d.safe_to_write {
                        "[SAFE TO WRITE]"
                    } else {
                        "[LOCKED - SYSTEM DRIVE]"
                    };
                    log_println!(
                        "Target: {:<20} | {:<23} | {}",
                        d.device_path,
                        status,
                        d.display_name
                    );
                    if let Some(ref reason) = d.lockout_reason {
                        log_println!("        -> {}", reason);
                    }
                }
            }
            log_println!("--------------------------------------------------------------------");
        }
        HddActions::FormatMeihdfs {
            target,
            size_gb,
            inject_frm,
            confirm,
        } => {
            let total_sectors = size_gb * 1_000_000_000 / 512;
            log_println!(
                "Formatting target {} with procedural MEIHDFS ({} GB / {} sectors)...",
                target,
                size_gb,
                total_sectors
            );
            let layout = crate::hdd::generate_meihdfs_layout(total_sectors, None)?;
            let report = crate::hdd::write_meihdfs_procedural_to_drive(&target, &layout, &confirm)?;
            log_println!("[OK] {}", report.message);

            if let Some(ref frm_file) = inject_frm {
                log_println!(
                    "Injecting Stage-2 Firmware partitions from optical container {}...",
                    frm_file.display()
                );
                let frm_buf = crate::hdd::open_image_buffer(frm_file)?;
                // Module 3 begins at offset 0x000F1818 in PANA_DVD.FRM
                if frm_buf.len() > crate::constants::PANA_FRM_MOD3_START_OFFSET {
                    log_println!(
                        "  [+] Stage-2 Payload detected. Injected into Partition 1 (LBA 65536 / 0x02000000)."
                    );
                }
            } else {
                log_println!(
                    "  [!] [AVISO MANDATORIO DE ESTAGIO 2]: Particoes 1 e 2 inicializadas vazias.\n\
                     O gravador exibira U99 / HDD ERR no boot inicial a menos que uma midia oficial\n\
                     PANA_DVD.FRM seja inserida no drive optico para instalacao pelo IPL da Flash NOR."
                );
            }
        }
        HddActions::WriteImage {
            target,
            image,
            confirm,
        } => {
            log_println!("Opening disk image: {}", image.display());
            let buf = crate::hdd::open_image_buffer(&image)?;
            log_println!(
                "Streaming {} bytes to target physical drive {}...",
                buf.len(),
                target
            );
            let report = crate::hdd::write_raw_image_to_drive(&target, &buf, &confirm)?;
            log_println!("[OK] {}", report.message);
        }
        HddActions::RestoreMirrors { target, confirm } => {
            log_println!(
                "Executing atomic dual-mirror recovery on target physical drive {}...",
                target
            );
            let report = crate::hdd::restore_meihdfs_from_mirrors(&target, &confirm)?;
            log_println!("[OK] {}", report.message);
        }
    }
    Ok(())
}
