//! CLI Command Handlers and Dispatch Engine
//!
//! Executes forensic analysis, cryptographic routines, hardware operations,
//! and reverse engineering queries invoked via the command-line interface.

use std::path::PathBuf;

use anyhow::Result;
use sha2::{Digest, Sha256};

use crate::constants::{
    format_memory_map_table, get_am34_memory_map_json, DEFAULT_FRM_CONTAINER_PATH,
    DEFAULT_HDD_IMAGE_PATH,
};
use crate::crypto::{analyze_frm_container, dump_toc, print_codebook};
use crate::hdd::{decode_meihdfs_descriptors, extract_hdd_partitions, inspect_hdd_image};
use crate::headers::{dump_file_hex, inspect_firmware_headers};
use crate::manuals::search_manuals;
use crate::nvram::clean_flash_nvram;
use crate::pico::{list_pico_devices, ping_pico, run_gdb_bridge};
use crate::scan::{run_deep_scan, scan_all_default};

use super::args::*;

/// Dispatches a parsed CLI command to its specialized handler.
pub fn dispatch(command: Commands) -> Result<()> {
    match command {
        Commands::Desktop | Commands::Gui => {
            crate::desktop::run_desktop_app();
            Ok(())
        }
        Commands::Interactive => crate::tui::run_interactive_pipeline(),
        Commands::Nvram(args) => handle_nvram(args),
        Commands::Hdd(args) => handle_hdd(args),
        Commands::Frm(args) => handle_frm(args),
        Commands::Headers(args) => handle_headers(args),
        Commands::Scan(args) => handle_scan(args),
        Commands::Manuals(args) => handle_manuals(args),
        Commands::Pico(args) => handle_pico(args),
        Commands::Ghidra(args) => handle_ghidra(args),
        Commands::Rag(args) => handle_rag(args),
        Commands::MemoryMap(args) => handle_memory_map(args),
        Commands::VerifyArtifact(args) => handle_verify_artifact(args),
        Commands::GdbConfig(args) => handle_gdb_config(args),
    }
}

pub fn handle_nvram(args: NvramArgs) -> Result<()> {
    match args.action {
        NvramActions::Clean { input, output } => {
            clean_flash_nvram(&input, output.as_deref())?;
        }
    }
    Ok(())
}

pub fn handle_hdd(args: HddArgs) -> Result<()> {
    match args.action {
        HddActions::Inspect {
            image,
            density,
            json,
        } => {
            let default_img = PathBuf::from(DEFAULT_HDD_IMAGE_PATH);
            let img_path = image.unwrap_or(default_img);
            inspect_hdd_image(&img_path, density, json.as_deref())?;
        }
        HddActions::Extract {
            image,
            outdir,
            json,
        } => {
            let default_img = PathBuf::from(DEFAULT_HDD_IMAGE_PATH);
            let img_path = image.unwrap_or(default_img);
            extract_hdd_partitions(&img_path, &outdir, json.as_deref())?;
        }
        HddActions::DecodeDescriptors {
            image,
            json,
            journal,
        } => {
            let default_img = PathBuf::from(DEFAULT_HDD_IMAGE_PATH);
            let img_path = image.unwrap_or(default_img);
            decode_meihdfs_descriptors(&img_path, journal, json.as_deref())?;
        }
    }
    Ok(())
}

pub fn handle_frm(args: FrmArgs) -> Result<()> {
    match args.action {
        FrmActions::Analyze {
            file,
            hdd,
            top,
            json,
        } => {
            let default_frm = PathBuf::from(DEFAULT_FRM_CONTAINER_PATH);
            let frm_path = file.unwrap_or(default_frm);
            analyze_frm_container(&frm_path, hdd.as_deref(), top, json.as_deref())?;
        }
        FrmActions::DumpToc { file, json } => {
            let default_frm = PathBuf::from(DEFAULT_FRM_CONTAINER_PATH);
            let frm_path = file.unwrap_or(default_frm);
            dump_toc(&frm_path, json.as_deref())?;
        }
        FrmActions::Codebook { file, hdd, json } => {
            let default_frm = PathBuf::from(DEFAULT_FRM_CONTAINER_PATH);
            let frm_path = file.unwrap_or(default_frm);
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

pub fn handle_manuals(args: ManualsArgs) -> Result<()> {
    let custom_terms = if args.terms.is_empty() {
        None
    } else {
        Some(args.terms)
    };
    search_manuals(&args.dir, custom_terms, args.filter.as_deref())?;
    Ok(())
}

pub fn handle_pico(args: PicoArgs) -> Result<()> {
    match args.action {
        PicoActions::List => {
            let devices = list_pico_devices()?;
            log_println!("====================================================================");
            log_println!("   PANADIGA HARDWARE DISCOVERY: SERIAL COM PORTS & PICO CONTROLLERS");
            log_println!("====================================================================");
            if devices.is_empty() {
                log_println!("No USB serial devices detected on system.");
            } else {
                for dev in &devices {
                    log_println!(
                        "Port: {:<10} | {:<32} | VID:{:04X} PID:{:04X} | {}",
                        dev.port_name,
                        dev.architecture.to_string(),
                        dev.vid,
                        dev.pid,
                        dev.description
                    );
                }
            }
            log_println!("--------------------------------------------------------------------");
        }
        PicoActions::Ping { port, baud } => {
            let report = ping_pico(&port, baud)?;
            log_println!("{}", report);
        }
        PicoActions::GdbBridge { port, baud, listen } => {
            run_gdb_bridge(&port, baud, &listen)?;
        }
    }
    Ok(())
}

pub fn handle_ghidra(args: GhidraArgs) -> Result<()> {
    match args.action {
        GhidraActions::Map { output } => {
            let script = crate::ghidra::generate_ghidra_memory_map();
            std::fs::write(&output, script)?;
            log_info!("[OK] Ghidra Memory Map script generated at: {:?}", output);
        }
        GhidraActions::Symbols { output } => {
            let script = crate::ghidra::generate_ghidra_symbols();
            std::fs::write(&output, script)?;
            log_info!("[OK] Ghidra Symbols script generated at: {:?}", output);
        }
        GhidraActions::ExportAll { dir } => {
            crate::ghidra::export_all_ghidra_scripts(&dir)?;
            log_info!(
                "[OK] All Ghidra scripts generated into directory: {:?}",
                dir
            );
        }
    }
    Ok(())
}

pub fn handle_rag(args: RagArgs) -> Result<()> {
    match args.action {
        RagActions::Query { query, json } => {
            let ans = crate::rag::query(&query);
            if json {
                let dto = ans.to_query_result();
                log_println!("{}", serde_json::to_string_pretty(&dto)?);
            } else {
                log_println!(
                    "===================================================================="
                );
                log_println!("         PANADIGA ZERO-ERROR RAG ENGINE (NATIVE RUST)");
                log_println!(
                    "===================================================================="
                );
                log_println!("Query:              {}", query);
                log_println!("Groundedness Score: {:.2}%", ans.groundedness_score * 100.0);
                log_println!("Confidence:         {:.2}%", ans.confidence * 100.0);
                if ans.is_abstained {
                    log_println!("Status:             ABSTAINED");
                    if let Some(reason) = &ans.abstention_reason {
                        log_println!("Reason:             {}", reason);
                    }
                } else {
                    log_println!("Status:             ANSWERED");
                }
                if let Some(s) = &ans.safety_report {
                    log_println!(
                        "Safety Check:       {}",
                        if s.is_safe {
                            "PASS (SAFE)"
                        } else {
                            "FAIL (HAZARD DETECTED)"
                        }
                    );
                    for v in &s.violations {
                        log_println!("  [!] VIOLATION: {}", v);
                    }
                    for a in &s.critical_alerts {
                        log_println!("  [*] ALERT: {}", a);
                    }
                }
                if let Some(c) = &ans.critic_report {
                    log_println!(
                        "Critic Faithful:    {}",
                        if c.is_faithful { "YES" } else { "NO" }
                    );
                    if !c.ungrounded_entities.is_empty() {
                        log_println!("  [?] Ungrounded:   {:?}", c.ungrounded_entities);
                    }
                }
                log_println!(
                    "--------------------------------------------------------------------"
                );
                log_println!("ANSWER:\n{}", ans.answer_text);
                if !ans.citations.is_empty() {
                    log_println!(
                        "--------------------------------------------------------------------"
                    );
                    log_println!("CITATIONS:");
                    for (idx, cit) in ans.citations.iter().enumerate() {
                        log_println!(
                            "  [{}] {} | Section: {} | Lines: {}",
                            idx + 1,
                            cit.doc_title,
                            cit.section_title,
                            cit.line_range
                        );
                        log_println!("      Excerpt: \"{}\"", cit.excerpt);
                    }
                }
                log_println!(
                    "===================================================================="
                );
            }
        }
    }
    Ok(())
}

pub fn handle_memory_map(args: MemoryMapArgs) -> Result<()> {
    if args.format.to_lowercase() == "json" {
        let regions_json = get_am34_memory_map_json();
        let data = serde_json::to_string_pretty(&regions_json)?;
        if let Some(out_p) = args.output {
            std::fs::write(&out_p, &data)?;
            log_println!("Memory map written to {:?}", out_p);
        } else {
            log_println!("{}", data);
        }
    } else {
        let out = format_memory_map_table();
        if let Some(out_p) = args.output {
            std::fs::write(&out_p, &out)?;
            log_println!("Memory map written to {:?}", out_p);
        } else {
            log_println!("{}", out);
        }
    }
    Ok(())
}

pub fn handle_verify_artifact(args: VerifyArtifactArgs) -> Result<()> {
    if !args.file.exists() {
        log_error!("ERROR: File not found: {:?}", args.file);
        std::process::exit(1);
    }
    let data = std::fs::read(&args.file)?;
    let size = data.len();
    let mut hasher = Sha256::new();
    hasher.update(&data);
    let computed_sha256 = hex::encode(hasher.finalize());

    log_println!("====================================================================");
    log_println!("            CRYPTOGRAPHIC ARTIFACT VERIFICATION");
    log_println!("====================================================================");
    log_println!(
        "File:     {}",
        args.file.file_name().and_then(|n| n.to_str()).unwrap_or("")
    );
    log_println!("Size:     {} bytes ({:.2} KB)", size, size as f64 / 1024.0);
    log_println!("SHA-256:  {}", computed_sha256);
    log_println!("--------------------------------------------------------------------");

    if let Some(expected) = args.sha256 {
        if computed_sha256.eq_ignore_ascii_case(&expected) {
            log_println!("✔ HASH CONFIRMED SUCCESSFULLY!");
        } else {
            log_error!("✖ HASH MISMATCH! Expected: {}", expected);
            std::process::exit(2);
        }
    }
    Ok(())
}

pub fn handle_gdb_config(args: GdbConfigArgs) -> Result<()> {
    log_println!("====================================================================");
    log_println!("          DWire32A GDB RSP Bridge Configuration");
    log_println!("====================================================================");
    log_println!("• Serial Port:   {}", args.port);
    log_println!("• Listen Socket: {}:{}", args.host, args.tcp_port);
    log_println!();
    log_println!("GDB Command to Connect:");
    log_println!("  (gdb) target remote {}:{}", args.host, args.tcp_port);
    log_println!("  (gdb) set architecture am34");
    log_println!("  (gdb) info registers");
    log_println!("====================================================================");
    Ok(())
}
