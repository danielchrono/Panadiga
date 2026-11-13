//! CLI handlers for FRM cryptanalysis, binary header inspection, and string scanning.

use super::super::args::{FrmActions, FrmArgs, HeadersActions, HeadersArgs, ScanArgs};
use anyhow::Result;
use panadiga_core::constants::DEFAULT_FRM_CONTAINER_PATH;
use panadiga_core::crypto::{analyze_frm_container, dump_toc, print_codebook};
use panadiga_core::headers::{dump_file_hex, inspect_firmware_headers};
use panadiga_core::scan::{run_deep_scan, scan_all_default};
use std::path::PathBuf;

fn resolve_frm_path(file: Option<PathBuf>) -> PathBuf {
    file.unwrap_or_else(|| PathBuf::from(DEFAULT_FRM_CONTAINER_PATH))
}

pub fn handle_frm(args: FrmArgs) -> Result<()> {
    match args.action {
        FrmActions::Analyze {
            file,
            hdd,
            top,
            json,
        } => {
            let frm_path = resolve_frm_path(file);
            analyze_frm_container(&frm_path, hdd.as_deref(), top, json.as_deref())?;
        }
        FrmActions::DumpToc { file, json } => {
            let frm_path = resolve_frm_path(file);
            dump_toc(&frm_path, json.as_deref())?;
        }
        FrmActions::Codebook { file, hdd, json } => {
            let frm_path = resolve_frm_path(file);
            print_codebook(&frm_path, hdd.as_deref(), json.as_deref())?;
        }
    }
    Ok(())
}

pub fn handle_headers(args: HeadersArgs) -> Result<()> {
    match args.action {
        Some(HeadersActions::Inspect { hdd, frm, length }) => {
            inspect_firmware_headers(hdd.as_deref(), frm.as_deref(), length)?;
        }
        Some(HeadersActions::Dump {
            file,
            offset,
            length,
        }) => {
            dump_file_hex(&file, offset, length)?;
        }
        None => {
            inspect_firmware_headers(None, None, args.length)?;
        }
    }
    Ok(())
}

pub fn handle_scan(args: ScanArgs) -> Result<()> {
    let custom_kws = if args.keywords.is_empty() {
        None
    } else {
        Some(args.keywords)
    };

    match args.file {
        Some(f) => {
            run_deep_scan(
                &f,
                args.min_len,
                custom_kws,
                args.limit,
                args.json.as_deref(),
            )?;
        }
        None => {
            scan_all_default(args.min_len, args.limit)?;
        }
    }
    Ok(())
}
