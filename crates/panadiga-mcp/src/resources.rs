//! MCP Resources (`resources/list`, `resources/read`) exposing Panadiga SSOT specs and documentation.

use anyhow::{Result, bail};
use panadiga_core::constants::{
    AM34_FLASH_BASE, AM34_FLASH_SIZE, AM34_NVRAM_BASE, AM34_SYSTEM_MEMORY_REGIONS,
    SPANSION_BOOTLOADER_SIZE_BYTES, SPANSION_NVRAM_SIZE_BYTES,
};
use panadiga_core::ssot::{build_hardware_ssot_spec, get_diga_model_catalog_dto};
use serde_json::{Value, json};
use std::fs;
use std::path::{Component, Path, PathBuf};

fn collect_markdown_docs(root: &Path, docs_dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(docs_dir) else {
        return;
    };
    let mut paths: Vec<PathBuf> = entries.flatten().map(|e| e.path()).collect();
    paths.sort();

    for p in paths {
        if p.is_dir() {
            collect_markdown_docs(root, &p, out);
        } else if p.is_file() && p.extension().and_then(|s| s.to_str()) == Some("md") {
            out.push(p);
        }
    }
}

pub fn list_resources(workspace_root: &Path) -> Vec<Value> {
    let mut resources = vec![
        json!({
            "uri": "panadiga://invariants",
            "name": "Physical & Forensic Safety Invariants",
            "description": "Voltage caps (3.3V max LVTTL / NSW5V GP3 Hi-Z), P7501 power rules, Spansion Flash/IPL lockouts, MEIHFS raw image immutability.",
            "mimeType": "application/json"
        }),
        json!({
            "uri": "panadiga://memory-map",
            "name": "Matsushita AM34 / MN103S Physical Memory Map",
            "description": "Authoritative bus regions for SRAM, SDRAM, Spansion NOR Flash, IPL bootloader, NVRAM U99, ASIC & PATA MMIO.",
            "mimeType": "application/json"
        }),
        json!({
            "uri": "panadiga://pinout-p7501-uart",
            "name": "P7501 (TO WRITE JIGU) Async UART 115200 8N1 & RP2040/RP2350 Wiring",
            "description": "Primary DMR-EH55 P7501 discrete buffer mapping (SW_UART, XRESET, UARTM2P_T/Q7514, GND, NSW5V Hi-Z ONLY, UARTP2M_T) plus voltage limits.",
            "mimeType": "application/json"
        }),
        json!({
            "uri": "panadiga://visual-catalog",
            "name": "DMR-EH55 Curated Visual Catalog (238 PNGs)",
            "description": "Structured JSON index (docs/assets/visual_manifest.json) of all 238 curated schematic, PCB foil, and vector CAD PNG crops.",
            "mimeType": "application/json"
        }),
        json!({
            "uri": "panadiga://models-catalog",
            "name": "Panasonic DIGA Hardware Catalog",
            "description": "Supported DIGA models, Flash chips, NVRAM offsets, and SoC profiles.",
            "mimeType": "application/json"
        }),
        json!({
            "uri": "panadiga://pico-commands",
            "name": "Pico P7501 Async UART Serial Protocol Command Catalog",
            "description": "Complete command set implemented by firmware/pico-p7501 (PING, STATUS, PINCHECK, BUSDIAG, BOOTCAP, JIGBOOT, UARTMON, UARTSEND, ISOLATE, LIVE_CAPTURE, GDB RSP).",
            "mimeType": "application/json"
        }),
    ];

    let docs_root = workspace_root.join("docs");
    let mut md_files = Vec::new();
    collect_markdown_docs(workspace_root, &docs_root, &mut md_files);

    for path in md_files.into_iter().take(250) {
        if let Ok(rel) = path.strip_prefix(workspace_root) {
            let rel_posix = rel.to_string_lossy().replace('\\', "/");
            let title = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("doc")
                .replace('_', " ");
            resources.push(json!({
                "uri": format!("panadiga://{}", rel_posix),
                "name": title,
                "description": format!("Technical dossier: {}", rel_posix),
                "mimeType": "text/markdown"
            }));
        }
    }

    resources
}

pub fn read_resource(uri: &str, workspace_root: &Path) -> Result<Value> {
    match uri {
        "panadiga://invariants" => {
            let spec = build_hardware_ssot_spec();
            let payload = json!({
                "max_nominal_voltage_v": spec.thresholds.nominal_vdd_volts,
                "max_safe_voltage_v": spec.thresholds.max_vdd_ceiling_volts,
                "min_safe_voltage_v": spec.thresholds.safe_operating_min_volts,
                "min_vdd_threshold_v": spec.thresholds.min_vdd_threshold_volts,
                "pin6_never_power_pico": true,
                "spansion_bus_base": format!("0x{:08X}", AM34_FLASH_BASE),
                "spansion_total_bytes": AM34_FLASH_SIZE,
                "ipl_read_only_bytes": SPANSION_BOOTLOADER_SIZE_BYTES,
                "nvram_mutable_bus_base": format!("0x{:08X}", AM34_NVRAM_BASE),
                "nvram_mutable_size_bytes": SPANSION_NVRAM_SIZE_BYTES,
                "forensic_raw_dumps_immutable": true,
            });
            Ok(json!({
                "contents": [{
                    "uri": uri,
                    "mimeType": "application/json",
                    "text": serde_json::to_string_pretty(&payload)?
                }]
            }))
        }
        "panadiga://memory-map" => {
            let regions: Vec<Value> = AM34_SYSTEM_MEMORY_REGIONS
                .iter()
                .map(|r| r.to_json_value())
                .collect();
            Ok(json!({
                "contents": [{
                    "uri": uri,
                    "mimeType": "application/json",
                    "text": serde_json::to_string_pretty(&regions)?
                }]
            }))
        }
        "panadiga://pinout-p7501-uart" => {
            let spec = build_hardware_ssot_spec();
            let payload = json!({
                "connector": "P7501 (JST B6B-ZR-3R4 TO JIGU) Async UART 115200 8N1",
                "pinout": spec.pinout,
                "thresholds": spec.thresholds,
                "breakin_modes": spec.breakin_modes,
            });
            Ok(json!({
                "contents": [{
                    "uri": uri,
                    "mimeType": "application/json",
                    "text": serde_json::to_string_pretty(&payload)?
                }]
            }))
        }
        "panadiga://visual-catalog" => {
            let raw = crate::tools::visuals::load_visual_manifest_raw(workspace_root)
                .map_err(|e| anyhow::anyhow!(e))?;
            Ok(json!({
                "contents": [{
                    "uri": uri,
                    "mimeType": "application/json",
                    "text": raw
                }]
            }))
        }
        "panadiga://models-catalog" => {
            let catalog = get_diga_model_catalog_dto();
            Ok(json!({
                "contents": [{
                    "uri": uri,
                    "mimeType": "application/json",
                    "text": serde_json::to_string_pretty(&catalog)?
                }]
            }))
        }
        "panadiga://pico-commands" => {
            let spec = build_hardware_ssot_spec();
            Ok(json!({
                "contents": [{
                    "uri": uri,
                    "mimeType": "application/json",
                    "text": serde_json::to_string_pretty(&spec.pico_commands)?
                }]
            }))
        }
        other if other.starts_with("panadiga://docs/") => {
            let rel_str = other.trim_start_matches("panadiga://");
            let rel_path = Path::new(rel_str);
            if rel_path.components().any(|c| {
                matches!(
                    c,
                    Component::ParentDir | Component::RootDir | Component::Prefix(_)
                )
            }) {
                bail!("Invalid resource path traversal attempt: {}", uri);
            }
            let full_path = workspace_root.join(rel_path);
            if !full_path.is_file() {
                bail!("Document resource not found: {}", uri);
            }
            let text = fs::read_to_string(&full_path)?;
            Ok(json!({
                "contents": [{
                    "uri": uri,
                    "mimeType": "text/markdown",
                    "text": text
                }]
            }))
        }
        _ => bail!("Unknown Panadiga resource URI: {}", uri),
    }
}
