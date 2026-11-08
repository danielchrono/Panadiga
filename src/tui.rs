//! Panasonic DIGA Engineering Suite - Interactive Terminal Workbench (TUI)
//!
//! Provides a structured, cadenced terminal interface guiding the engineer through
//! hardware link discovery, NVRAM recovery, MEIHDFS forensics, firmware container analysis,
//! memory map telemetry, and zero-error safety queries.

use std::io::{self, Write};
use std::path::{Path, PathBuf};

use anyhow::Result;
use sha2::{Digest, Sha256};

use crate::constants::{
    DEFAULT_BAUD_RATE, DEFAULT_FRM_CONTAINER_PATH, DEFAULT_GDB_LISTEN_SOCKET,
    DEFAULT_HDD_EXTRACT_DIR, DEFAULT_HDD_IMAGE_PATH, DEFAULT_MANUALS_DIR, DEFAULT_SERIAL_PORT,
    DEFAULT_TOOLS_DIR,
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
    print!("{}", prompt);
    let _ = io::stdout().flush();
    let mut buffer = String::new();
    let _ = io::stdin().read_line(&mut buffer);
    buffer.trim().to_string()
}

/// Runs the interactive terminal user interface (TUI) loop.
pub fn run_interactive_pipeline() -> Result<()> {
    loop {
        println!();
        println!("\x1b[1;36m====================================================================\x1b[0m");
        println!("\x1b[1;36m      PANADIGA ENGINEERING SUITE - INTERACTIVE WORKBENCH (TUI)\x1b[0m");
        println!("\x1b[1;33m      Matsushita AM34 / UniPhier MN2DS0016 / Spansion S29JL064H\x1b[0m");
        println!("\x1b[1;36m====================================================================\x1b[0m");
        println!("  \x1b[1;32m[1]\x1b[0m Hardware Host Discovery & Ping (Pico RP2040 / RP2350)");
        println!("  \x1b[1;32m[2]\x1b[0m NVRAM & NOR Flash Sanitization (Surgical Microsector U99 Fix)");
        println!("  \x1b[1;32m[3]\x1b[0m HDD MEIHDFS Storage Forensics (Inspect & Extract Partitions)");
        println!("  \x1b[1;32m[4]\x1b[0m Firmware & Optical Container (.FRM Cryptanalysis & Codebook)");
        println!("  \x1b[1;32m[5]\x1b[0m Matsushita AM34 Official Memory Map Architecture");
        println!("  \x1b[1;32m[6]\x1b[0m Export Ghidra Reverse Engineering Scripts (SSOT)");
        println!("  \x1b[1;32m[7]\x1b[0m Service Manuals & Schematic PDF Indexer & Search");
        println!("  \x1b[1;32m[8]\x1b[0m Zero-Error Deterministic Safety RAG Engine (Ask Engineer)");
        println!("  \x1b[1;32m[9]\x1b[0m Cryptographic Artifact Hash Verification (SHA-256)");
        println!("  \x1b[1;35m[D]\x1b[0m Launch Panadiga Native Desktop GUI (Tauri / React)");
        println!("  \x1b[1;31m[0]\x1b[0m Exit Workbench");
        println!("\x1b[1;36m--------------------------------------------------------------------\x1b[0m");

        let choice = prompt_line("\x1b[1;37mSelect an option [1-9, D, 0]: \x1b[0m");
        println!();

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
                println!("\x1b[1;32mExiting Panadiga Workbench. Have a great session!\x1b[0m");
                break;
            }
            _ => {
                println!("\x1b[1;31m[!] Invalid option. Please choose 1-9, D, or 0.\x1b[0m");
            }
        }

        println!();
        prompt_line("\x1b[1;30mPress [Enter] to continue to workbench menu...\x1b[0m");
    }

    Ok(())
}

fn handle_pico_option() -> Result<()> {
    println!("\x1b[1;36m>> Scanning for USB serial devices & Pico controllers...\x1b[0m");
    let devices = list_pico_devices()?;
    if devices.is_empty() {
        println!("\x1b[1;33m[!] No USB serial devices detected. Connect Raspberry Pi Pico via USB.\x1b[0m");
    } else {
        for dev in &devices {
            println!(
                "  • Port: {:<8} | {:<28} | VID:{:04X} PID:{:04X} | {}",
                dev.port_name,
                dev.architecture.to_string(),
                dev.vid,
                dev.pid,
                dev.description
            );
        }
        let sub = prompt_line("\x1b[1;37mAction ([P]ing / [G]DB Bridge / [Enter] to return): \x1b[0m");
        if sub.eq_ignore_ascii_case("P") {
            let prompt_msg = format!("Enter COM port [default: {}]: ", DEFAULT_SERIAL_PORT);
            let port = prompt_line(&prompt_msg);
            let p = if port.is_empty() { DEFAULT_SERIAL_PORT } else { &port };
            match ping_pico(p, DEFAULT_BAUD_RATE) {
                Ok(res) => println!("\x1b[1;32m{}\x1b[0m", res),
                Err(e) => println!("\x1b[1;31m[ERROR] Failed to ping {}: {}\x1b[0m", p, e),
            }
        } else if sub.eq_ignore_ascii_case("G") {
            let prompt_msg = format!("Enter COM port [default: {}]: ", DEFAULT_SERIAL_PORT);
            let port = prompt_line(&prompt_msg);
            let p = if port.is_empty() { DEFAULT_SERIAL_PORT } else { &port };
            println!(
                "\x1b[1;32m[INFO] Starting GDB RSP bridge on port {} -> {}...\x1b[0m",
                p, DEFAULT_GDB_LISTEN_SOCKET
            );
            let _ = run_gdb_bridge(p, DEFAULT_BAUD_RATE, DEFAULT_GDB_LISTEN_SOCKET);
        }
    }
    Ok(())
}

fn handle_nvram_option() -> Result<()> {
    println!("\x1b[1;36m>> NVRAM & NOR Flash Surgical Recovery (Spansion S29JL064H)\x1b[0m");
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
        println!("\x1b[1;31m[ERROR] File not found: {:?}\x1b[0m", dump_path);
    } else {
        clean_flash_nvram(&dump_path, None)?;
    }
    Ok(())
}

fn handle_hdd_option() -> Result<()> {
    println!("\x1b[1;36m>> HDD MEIHDFS Storage Forensics\x1b[0m");
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
        println!("\x1b[1;31m[ERROR] File not found: {:?}\x1b[0m", img_path);
    } else {
        let act = prompt_line("Action ([I]nspect partitions / [E]xtract partitions / [D]ecode descriptors): ");
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
    println!("\x1b[1;36m>> Firmware & Optical Container (.FRM Cryptanalysis)\x1b[0m");
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
        println!("\x1b[1;31m[ERROR] File not found: {:?}\x1b[0m", frm_path);
    } else {
        let act = prompt_line("Action ([A]nalyze container / [T]OC dump / [C]odebook comparison): ");
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
    println!("\x1b[1;36m>> Matsushita AM34 / UniPhier MN2DS0016 Memory Map Architecture\x1b[0m");
    println!("+------------------+---------------+---------------+-----------+-------------+");
    println!("| Region           | Start Address | End Address   | Size (KB) | Permissions |");
    println!("+------------------+---------------+---------------+-----------+-------------+");
    println!("| INTERNAL_SRAM    | 0x00000000    | 0x0003FFFF    |       256 | rwx         |");
    println!("| CORE_MMIO        | 0x10000000    | 0x1000FFFF    |        64 | rw-         |");
    println!("| MAIN_SDRAM       | 0x40000000    | 0x43FFFFFF    |     65536 | rwx         |");
    println!("| FLASH_NOR_IPL    | 0x80000000    | 0x807EFFFF    |      8128 | r-x         |");
    println!("| NVRAM_U99        | 0x807F0000    | 0x807FFFFF    |        64 | rw-         |");
    println!("| SYSTEM_MMIO      | 0xC0000000    | 0xC0003FFF    |        16 | rw-         |");
    println!("| DWIRE32A_OCD     | 0xC0004000    | 0xC00040FF    |         1 | rw-         |");
    println!("| ATA_IDE_MMIO     | 0xD4000000    | 0xD4000FFF    |         4 | rw-         |");
    println!("+------------------+---------------+---------------+-----------+-------------+");
}

fn handle_ghidra_export() -> Result<()> {
    println!("\x1b[1;36m>> Exporting Ghidra Scripts from Rust SSOT...\x1b[0m");
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
    println!("\x1b[1;32m[OK] Ghidra scripts exported successfully into {:?}\x1b[0m", dest);
    Ok(())
}

fn handle_manuals_search() -> Result<()> {
    println!("\x1b[1;36m>> Service Manuals & Schematic PDF Indexer & Search\x1b[0m");
    let term = prompt_line("Search term (e.g. CN55001, S29JL064H, VEP79138): ");
    if term.is_empty() {
        println!("\x1b[1;33m[!] No search term entered.\x1b[0m");
    } else {
        search_manuals(&PathBuf::from(DEFAULT_MANUALS_DIR), Some(vec![term]), None)?;
    }
    Ok(())
}

fn handle_rag_query() {
    println!("\x1b[1;36m>> Zero-Error Deterministic Safety RAG Engine\x1b[0m");
    let q = prompt_line("Enter engineering or electrical safety question: ");
    if !q.is_empty() {
        let ans = rag_query(&q);
        println!("\x1b[1;36m--------------------------------------------------------------------\x1b[0m");
        println!("Groundedness Score: {:.2}%", ans.groundedness_score * 100.0);
        println!("Confidence:         {:.2}%", ans.confidence * 100.0);
        println!(
            "Status:             {}",
            if ans.is_abstained {
                "\x1b[1;31mABSTAINED\x1b[0m"
            } else {
                "\x1b[1;32mANSWERED\x1b[0m"
            }
        );
        if let Some(r) = ans.abstention_reason {
            println!("Abstention Reason:  {}", r);
        }
        println!();
        println!("Answer:\n{}", ans.answer_text);
        println!("\x1b[1;36m--------------------------------------------------------------------\x1b[0m");
    }
}

fn handle_verify_artifact() -> Result<()> {
    println!("\x1b[1;36m>> Cryptographic Artifact Hash Verification (SHA-256)\x1b[0m");
    let path_str = prompt_line("Path to file: ");
    let p = PathBuf::from(&path_str);
    if !p.exists() {
        println!("\x1b[1;31m[ERROR] File not found: {:?}\x1b[0m", p);
    } else {
        let exp = prompt_line("Expected SHA-256 (optional, press Enter to calculate): ");
        let data = std::fs::read(&p)?;
        let mut hasher = Sha256::new();
        hasher.update(&data);
        let computed = hex::encode(hasher.finalize());
        println!("File:     {}", p.file_name().and_then(|n| n.to_str()).unwrap_or(""));
        println!("Size:     {} bytes ({:.2} KB)", data.len(), data.len() as f64 / 1024.0);
        println!("SHA-256:  \x1b[1;33m{}\x1b[0m", computed);
        if !exp.is_empty() {
            if computed.eq_ignore_ascii_case(&exp) {
                println!("\x1b[1;32m✔ HASH CONFIRMED SUCCESSFULLY!\x1b[0m");
            } else {
                println!("\x1b[1;31m✖ HASH MISMATCH! Expected: {}\x1b[0m", exp);
            }
        }
    }
    Ok(())
}

fn handle_launch_desktop() {
    println!("\x1b[1;35m>> Launching Panadiga Native Desktop GUI...\x1b[0m");
    let candidates = [
        "dist/panadiga-desktop.exe",
        "dist_bin/panadiga-desktop.exe",
        "target/release/panasonic-diga-desktop.exe",
        "dist/panadiga-desktop",
        "target/release/panasonic-diga-desktop",
    ];
    let mut found = false;
    for c in &candidates {
        let path = Path::new(c);
        if path.exists() {
            println!("\x1b[1;32m[INFO] Spawning: {:?}\x1b[0m", path);
            let _ = std::process::Command::new(path).spawn();
            found = true;
            break;
        }
    }
    if !found {
        println!("\x1b[1;33m[!] Desktop binary not found. Build it with: build.cmd (or npm run tauri build)\x1b[0m");
    }
}
