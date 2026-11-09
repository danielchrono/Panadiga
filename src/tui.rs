//! Panasonic DIGA Engineering Suite - Interactive Terminal Workbench (TUI)
//!
//! Provides a structured, cadenced terminal interface guiding the engineer through
//! hardware link discovery, NVRAM recovery, MEIHDFS forensics, firmware container analysis,
//! memory map telemetry, and zero-error safety queries.

use std::io::{self, Write};
use std::path::PathBuf;

use anyhow::Result;
use sha2::{Digest, Sha256};

use crate::constants::{
    DEFAULT_BAUD_RATE, DEFAULT_FRM_CONTAINER_PATH, DEFAULT_GDB_LISTEN_SOCKET,
    DEFAULT_HDD_EXTRACT_DIR, DEFAULT_HDD_IMAGE_PATH, DEFAULT_MANUALS_DIR, DEFAULT_SERIAL_PORT,
    DEFAULT_TOOLS_DIR, format_memory_map_table,
};
use crate::crypto::{analyze_frm_container, dump_toc, print_codebook};
use crate::ghidra::export_all_ghidra_scripts;
use crate::hdd::{decode_meihdfs_descriptors, extract_hdd_partitions, inspect_hdd_image};
use crate::manuals::search_manuals;
use crate::nvram::clean_flash_nvram;
use crate::pico::{list_pico_devices, ping_pico, run_gdb_bridge};
use crate::rag::query as rag_query;

/// Prompts the user with a formatted string and captures a single trimmed line of input.
pub fn prompt_line(prompt: &str) -> String {
    log_raw!("{}", prompt);
    let _ = io::stdout().flush();
    let mut buffer = String::new();
    let _ = io::stdin().read_line(&mut buffer);
    buffer.trim().to_string()
}

/// Runs the interactive terminal user interface (TUI) loop.
pub fn run_interactive_pipeline() -> Result<()> {
    loop {
        log_println!();
        log_println!(
            "\x1b[1;36m====================================================================\x1b[0m"
        );
        log_println!(
            "\x1b[1;36m      PANADIGA ENGINEERING SUITE - INTERACTIVE WORKBENCH (TUI)\x1b[0m"
        );
        log_println!(
            "\x1b[1;33m      Matsushita AM34 / UniPhier MN2DS0016 / Spansion S29JL064H\x1b[0m"
        );
        log_println!(
            "\x1b[1;36m====================================================================\x1b[0m"
        );
        log_println!(
            "  \x1b[1;32m[1]\x1b[0m Hardware Host Discovery & Ping (Pico RP2040 / RP2350)"
        );
        log_println!(
            "  \x1b[1;32m[2]\x1b[0m NVRAM & NOR Flash Sanitization (Surgical Microsector U99 Fix)"
        );
        log_println!(
            "  \x1b[1;32m[3]\x1b[0m HDD MEIHDFS Storage Forensics (Inspect & Extract Partitions)"
        );
        log_println!(
            "  \x1b[1;32m[4]\x1b[0m Firmware & Optical Container (.FRM Cryptanalysis & Codebook)"
        );
        log_println!("  \x1b[1;32m[5]\x1b[0m Matsushita AM34 Official Memory Map Architecture");
        log_println!("  \x1b[1;32m[6]\x1b[0m Export Ghidra Reverse Engineering Scripts (SSOT)");
        log_println!("  \x1b[1;32m[7]\x1b[0m Service Manuals & Schematic PDF Indexer & Search");
        log_println!(
            "  \x1b[1;32m[8]\x1b[0m Zero-Error Deterministic Safety RAG Engine (Ask Engineer)"
        );
        log_println!("  \x1b[1;32m[9]\x1b[0m Cryptographic Artifact Hash Verification (SHA-256)");
        log_println!("  \x1b[1;35m[D]\x1b[0m Launch Panadiga Native Desktop GUI (Tauri / React)");
        log_println!("  \x1b[1;31m[0]\x1b[0m Exit Workbench");
        log_println!(
            "\x1b[1;36m--------------------------------------------------------------------\x1b[0m"
        );

        let choice = prompt_line("\x1b[1;37mSelect an option [1-9, D, 0]: \x1b[0m");
        log_println!();

        match choice.to_uppercase().as_str() {
            "1" => handle_pico_option()?,
            "2" => handle_nvram_option()?,
            "3" => handle_hdd_option()?,
            "4" => handle_frm_option()?,
            "5" => print_memory_map_architecture(),
            "6" => handle_ghidra_export()?,
            "7" => handle_manuals_search()?,
            "8" => handle_rag_query(),
            "9" => handle_verify_artifact()?,
            "D" => handle_launch_desktop(),
            "0" | "Q" | "EXIT" => {
                log_println!("\x1b[1;32mExiting Panadiga Workbench. Have a great session!\x1b[0m");
                break;
            }
            _ => {
                log_println!("\x1b[1;31m[!] Invalid option. Please choose 1-9, D, or 0.\x1b[0m");
            }
        }

        log_println!();
        prompt_line("\x1b[1;30mPress [Enter] to continue to workbench menu...\x1b[0m");
    }

    Ok(())
}

fn handle_pico_option() -> Result<()> {
    log_println!("\x1b[1;36m>> Scanning for USB serial devices & Pico controllers...\x1b[0m");
    let devices = list_pico_devices()?;
    if devices.is_empty() {
        log_println!(
            "\x1b[1;33m[!] No USB serial devices detected. Connect Raspberry Pi Pico via USB.\x1b[0m"
        );
    } else {
        for dev in &devices {
            log_println!(
                "  • Port: {:<8} | {:<28} | VID:{:04X} PID:{:04X} | {}",
                dev.port_name,
                dev.architecture.to_string(),
                dev.vid,
                dev.pid,
                dev.description
            );
        }
        let sub =
            prompt_line("\x1b[1;37mAction ([P]ing / [G]DB Bridge / [Enter] to return): \x1b[0m");
        if sub.eq_ignore_ascii_case("P") {
            let prompt_msg = format!("Enter COM port [default: {}]: ", DEFAULT_SERIAL_PORT);
            let port = prompt_line(&prompt_msg);
            let p = if port.is_empty() {
                DEFAULT_SERIAL_PORT
            } else {
                &port
            };
            match ping_pico(p, DEFAULT_BAUD_RATE) {
                Ok(res) => log_println!("\x1b[1;32m{}\x1b[0m", res),
                Err(e) => log_println!("\x1b[1;31m[ERROR] Failed to ping {}: {}\x1b[0m", p, e),
            }
        } else if sub.eq_ignore_ascii_case("G") {
            let prompt_msg = format!("Enter COM port [default: {}]: ", DEFAULT_SERIAL_PORT);
            let port = prompt_line(&prompt_msg);
            let p = if port.is_empty() {
                DEFAULT_SERIAL_PORT
            } else {
                &port
            };
            log_println!(
                "\x1b[1;32m[INFO] Starting GDB RSP bridge on port {} -> {}...\x1b[0m",
                p,
                DEFAULT_GDB_LISTEN_SOCKET
            );
            let _ = run_gdb_bridge(p, DEFAULT_BAUD_RATE, DEFAULT_GDB_LISTEN_SOCKET);
        }
    }
    Ok(())
}

fn handle_nvram_option() -> Result<()> {
    log_println!("\x1b[1;36m>> NVRAM & NOR Flash Surgical Recovery (Spansion S29JL064H)\x1b[0m");
    let input = prompt_line(&format!(
        "Enter path to Flash/NVRAM dump [default: {}]: ",
        DEFAULT_HDD_IMAGE_PATH
    ));
    let dump_path = if input.is_empty() {
        PathBuf::from(DEFAULT_HDD_IMAGE_PATH)
    } else {
        PathBuf::from(input)
    };
    if !dump_path.exists() {
        log_println!("\x1b[1;31m[ERROR] File not found: {:?}\x1b[0m", dump_path);
    } else {
        clean_flash_nvram(&dump_path, None)?;
    }
    Ok(())
}

fn handle_hdd_option() -> Result<()> {
    log_println!("\x1b[1;36m>> HDD MEIHDFS Storage Forensics\x1b[0m");
    let input = prompt_line(&format!(
        "Enter path to HDD binary image [default: {}]: ",
        DEFAULT_HDD_IMAGE_PATH
    ));
    let img_path = if input.is_empty() {
        PathBuf::from(DEFAULT_HDD_IMAGE_PATH)
    } else {
        PathBuf::from(input)
    };
    if !img_path.exists() {
        log_println!("\x1b[1;31m[ERROR] File not found: {:?}\x1b[0m", img_path);
    } else {
        let act = prompt_line(
            "Action ([I]nspect partitions / [E]xtract partitions / [D]ecode descriptors): ",
        );
        if act.eq_ignore_ascii_case("E") {
            let outdir = prompt_line(&format!(
                "Destination directory [default: {}]: ",
                DEFAULT_HDD_EXTRACT_DIR
            ));
            let dest = if outdir.is_empty() {
                PathBuf::from(DEFAULT_HDD_EXTRACT_DIR)
            } else {
                PathBuf::from(outdir)
            };
            extract_hdd_partitions(&img_path, &dest, None)?;
        } else if act.eq_ignore_ascii_case("D") {
            decode_meihdfs_descriptors(&img_path, false, None)?;
        } else {
            inspect_hdd_image(&img_path, false, None)?;
        }
    }
    Ok(())
}

fn handle_frm_option() -> Result<()> {
    log_println!("\x1b[1;36m>> Firmware & Optical Container (.FRM Cryptanalysis)\x1b[0m");
    let input = prompt_line(&format!(
        "Enter path to .FRM file [default: {}]: ",
        DEFAULT_FRM_CONTAINER_PATH
    ));
    let frm_path = if input.is_empty() {
        PathBuf::from(DEFAULT_FRM_CONTAINER_PATH)
    } else {
        PathBuf::from(input)
    };
    if !frm_path.exists() {
        log_println!("\x1b[1;31m[ERROR] File not found: {:?}\x1b[0m", frm_path);
    } else {
        let act =
            prompt_line("Action ([A]nalyze container / [T]OC dump / [C]odebook comparison): ");
        if act.eq_ignore_ascii_case("T") {
            dump_toc(&frm_path, None)?;
        } else if act.eq_ignore_ascii_case("C") {
            print_codebook(&frm_path, None, None)?;
        } else {
            analyze_frm_container(&frm_path, None, 10, None)?;
        }
    }
    Ok(())
}

fn print_memory_map_architecture() {
    log_println!(
        "\x1b[1;36m>> Matsushita AM34 / UniPhier MN2DS0016 Memory Map Architecture\x1b[0m"
    );
    log_raw!("{}", format_memory_map_table());
}

fn handle_ghidra_export() -> Result<()> {
    log_println!("\x1b[1;36m>> Exporting Ghidra Scripts from Rust SSOT...\x1b[0m");
    let out = prompt_line(&format!(
        "Destination directory [default: {}]: ",
        DEFAULT_TOOLS_DIR
    ));
    let dest = if out.is_empty() {
        PathBuf::from(DEFAULT_TOOLS_DIR)
    } else {
        PathBuf::from(out)
    };
    export_all_ghidra_scripts(&dest)?;
    log_println!(
        "\x1b[1;32m[OK] Ghidra scripts exported successfully into {:?}\x1b[0m",
        dest
    );
    Ok(())
}

fn handle_manuals_search() -> Result<()> {
    log_println!("\x1b[1;36m>> Service Manuals & Schematic PDF Indexer & Search\x1b[0m");
    let term = prompt_line("Search term (e.g. CN55001, S29JL064H, VEP79138): ");
    if term.is_empty() {
        log_println!("\x1b[1;33m[!] No search term entered.\x1b[0m");
    } else {
        search_manuals(&PathBuf::from(DEFAULT_MANUALS_DIR), Some(vec![term]), None)?;
    }
    Ok(())
}

fn handle_rag_query() {
    log_println!("\x1b[1;36m>> Zero-Error Deterministic Safety RAG Engine\x1b[0m");
    let q = prompt_line("Enter engineering or electrical safety question: ");
    if !q.is_empty() {
        let ans = rag_query(&q);
        log_println!(
            "\x1b[1;36m--------------------------------------------------------------------\x1b[0m"
        );
        log_println!("Groundedness Score: {:.2}%", ans.groundedness_score * 100.0);
        log_println!("Confidence:         {:.2}%", ans.confidence * 100.0);
        log_println!(
            "Status:             {}",
            if ans.is_abstained {
                "\x1b[1;31mABSTAINED\x1b[0m"
            } else {
                "\x1b[1;32mANSWERED\x1b[0m"
            }
        );
        if let Some(r) = ans.abstention_reason {
            log_println!("Abstention Reason:  {}", r);
        }
        log_println!();
        log_println!("Answer:\n{}", ans.answer_text);
        log_println!(
            "\x1b[1;36m--------------------------------------------------------------------\x1b[0m"
        );
    }
}

fn handle_verify_artifact() -> Result<()> {
    log_println!("\x1b[1;36m>> Cryptographic Artifact Hash Verification (SHA-256)\x1b[0m");
    let path_str = prompt_line("Path to file: ");
    let p = PathBuf::from(&path_str);
    if !p.exists() {
        log_println!("\x1b[1;31m[ERROR] File not found: {:?}\x1b[0m", p);
    } else {
        let exp = prompt_line("Expected SHA-256 (optional, press Enter to calculate): ");
        let data = std::fs::read(&p)?;
        let mut hasher = Sha256::new();
        hasher.update(&data);
        let computed = hex::encode(hasher.finalize());
        log_println!(
            "File:     {}",
            p.file_name().and_then(|n| n.to_str()).unwrap_or("")
        );
        log_println!(
            "Size:     {} bytes ({:.2} KB)",
            data.len(),
            data.len() as f64 / 1024.0
        );
        log_println!("SHA-256:  \x1b[1;33m{}\x1b[0m", computed);
        if !exp.is_empty() {
            if computed.eq_ignore_ascii_case(&exp) {
                log_println!("\x1b[1;32m✔ HASH CONFIRMED SUCCESSFULLY!\x1b[0m");
            } else {
                log_println!("\x1b[1;31m✖ HASH MISMATCH! Expected: {}\x1b[0m", exp);
            }
        }
    }
    Ok(())
}

fn handle_launch_desktop() {
    log_println!("\x1b[1;35m>> Launching Panadiga Native Desktop GUI...\x1b[0m");
    if let Ok(exe_path) = std::env::current_exe() {
        log_println!(
            "\x1b[1;32m[INFO] Spawning GUI instance: {:?} desktop\x1b[0m",
            exe_path
        );
        let _ = std::process::Command::new(exe_path).arg("desktop").spawn();
    } else {
        log_println!("\x1b[1;31m[ERROR] Unable to locate current executable path.\x1b[0m");
    }
}
