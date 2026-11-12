//! CLI handlers for RP2040/RP2350 Pico hardware probing, flashing, and GDB RSP bridge.

use super::super::args::{GdbConfigArgs, PicoActions, PicoArgs};
use crate::pico::{list_pico_devices, ping_pico, run_gdb_bridge};
use anyhow::Result;
use std::path::PathBuf;

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
        PicoActions::Flash { port, file } => {
            log_println!("====================================================================");
            log_println!("   PANADIGA AUTOMATED PICO FIRMWARE FLASHER (DWire32A OCD)");
            log_println!("====================================================================");
            if let Some(ref p) = port {
                log_println!("Initiating 1200-baud touch reset on port {}...", p);
                let _ = crate::pico::trigger_1200_baud_bootsel(p);
            }
            let bootsel_drives = crate::pico::detect_bootsel_drives();
            if bootsel_drives.is_empty() {
                anyhow::bail!(
                    "No Raspberry Pi BOOTSEL drives (RPI-RP2 / RP2350) detected! Hold BOOTSEL button while plugging USB."
                );
            }
            let target_drive = &bootsel_drives[0];
            log_println!("Found active BOOTSEL drive: {}", target_drive.display());

            let uf2_data = if let Some(ref custom_path) = file {
                std::fs::read(custom_path)?
            } else {
                let candidates = [
                    PathBuf::from("dist/panasonic_dwire32a_unified.uf2"),
                    PathBuf::from("dist/firmware/panasonic_dwire32a_unified.uf2"),
                    PathBuf::from("src/pico/firmware_bins/panasonic_dwire32a_unified.uf2"),
                    PathBuf::from("dwire32a_jtag_research/panasonic_dwire32a_unified.uf2"),
                    PathBuf::from("dist/panasonic_dwire32a_rp2040.uf2"),
                    PathBuf::from("dist/firmware/panasonic_dwire32a_rp2040.uf2"),
                    PathBuf::from("src/pico/firmware_bins/panasonic_dwire32a_rp2040.uf2"),
                    PathBuf::from("dwire32a_jtag_research/panasonic_dwire32a_rp2040.uf2"),
                    PathBuf::from("dist_bin/firmware/panasonic_dwire32a_rp2040.uf2"),
                ];
                let mut found = None;
                for c in &candidates {
                    if c.exists() {
                        if let Ok(data) = std::fs::read(c) {
                            found = Some(data);
                            break;
                        }
                    }
                }
                match found {
                    Some(data) => data,
                    None => crate::pico::flasher::EMBEDDED_UNIFIED_UF2.to_vec(),
                }
            };

            let report = crate::pico::flash_uf2_to_drive(target_drive, &uf2_data)?;
            log_println!("[SUCCESS] {}", report.message);
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
