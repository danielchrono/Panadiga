//! Hardware safety, memory map, Pico P7501 UART catalog, and repository health MCP tools.

use panadiga_core::constants::{
    AM34_FLASH_BASE, AM34_NVRAM_BASE, AM34_SYSTEM_MEMORY_REGIONS, DEFAULT_MANUALS_DIR,
    SPANSION_BOOTLOADER_SIZE_BYTES, SPANSION_FLASH_SIZE_BYTES, SPANSION_NVRAM_BASE_OFFSET,
};
use panadiga_core::manuals::count_pdf_files;
use panadiga_core::ssot::{build_hardware_ssot_spec, get_diga_model_catalog_dto};
use panadiga_rag::config::MAX_SAFE_VOLTAGE_VOLTS;
use serde_json::{Map, Value, json};
use std::path::Path;

use crate::workspace::{optional_non_empty_str, parse_u64_hex_or_dec};

pub fn handle_memory_map(args: &Value) -> Value {
    let filter_upper = optional_non_empty_str(args, "filter_name").map(|s| s.to_uppercase());

    let regions: Vec<Value> = AM34_SYSTEM_MEMORY_REGIONS
        .iter()
        .filter(|r| match &filter_upper {
            Some(f) => r.name.to_uppercase().contains(f),
            None => true,
        })
        .map(|r| r.to_json_value())
        .collect();

    let total = regions.len();
    json!({ "regions": regions, "total": total })
}

pub fn handle_safety_check(args: &Value) -> Value {
    let mut safe = true;
    let mut violations: Vec<String> = Vec::new();
    let mut result = Map::new();

    if let Some(v_val) = args.get("voltage").filter(|v| !v.is_null()) {
        match v_val.as_f64() {
            Some(v) => {
                if v > MAX_SAFE_VOLTAGE_VOLTS {
                    safe = false;
                    violations.push(format!(
                        "VOLTAGE_VIOLATION: {}V exceeds maximum safe threshold of {:.2}V (3.3V LVTTL nominal)!",
                        v, MAX_SAFE_VOLTAGE_VOLTS
                    ));
                } else {
                    result.insert(
                        "voltage_status".to_string(),
                        Value::String(format!(
                            "{}V is within safe bounds (<= {:.2}V).",
                            v, MAX_SAFE_VOLTAGE_VOLTS
                        )),
                    );
                }
            }
            None => {
                safe = false;
                violations.push(format!("Invalid numeric voltage: {}", v_val));
            }
        }
    }

    if let Some(addr_val) = args.get("address").filter(|v| !v.is_null()) {
        let parsed = match addr_val.as_str() {
            Some(s) => parse_u64_hex_or_dec(s),
            None => addr_val
                .as_u64()
                .ok_or_else(|| format!("Invalid address type: {}", addr_val)),
        };

        match parsed {
            Ok(addr) => {
                let rel_ipl_end = (SPANSION_BOOTLOADER_SIZE_BYTES as u64) - 1;
                let bus_ipl_start = AM34_FLASH_BASE as u64;
                let bus_ipl_end = bus_ipl_start + rel_ipl_end;

                let rel_nvram_start = SPANSION_NVRAM_BASE_OFFSET as u64;
                let rel_nvram_end = (SPANSION_FLASH_SIZE_BYTES as u64) - 1;
                let bus_nvram_start = AM34_NVRAM_BASE as u64;
                let bus_nvram_end = bus_ipl_start + rel_nvram_end;

                if (addr <= rel_ipl_end) || (bus_ipl_start <= addr && addr <= bus_ipl_end) {
                    safe = false;
                    violations.push(format!(
                        "FLASH_WRITE_VIOLATION: Address 0x{:08X} resides in protected IPL/Bootloader area! Write forbidden (E05).",
                        addr
                    ));
                } else if (rel_nvram_start <= addr && addr <= rel_nvram_end)
                    || (bus_nvram_start <= addr && addr <= bus_nvram_end)
                {
                    result.insert(
                        "address_status".to_string(),
                        Value::String(format!(
                            "Address 0x{:08X} is inside mutable NVRAM (U99 unbrick area).",
                            addr
                        )),
                    );
                } else {
                    result.insert(
                        "address_status".to_string(),
                        Value::String(format!(
                            "Address 0x{:08X} is outside NOR Flash range.",
                            addr
                        )),
                    );
                }
            }
            Err(err_msg) => {
                safe = false;
                violations.push(err_msg);
            }
        }
    }

    result.insert("safe".to_string(), Value::Bool(safe));
    result.insert(
        "violations".to_string(),
        Value::Array(violations.into_iter().map(Value::String).collect()),
    );
    Value::Object(result)
}

pub fn handle_pico_catalog(args: &Value) -> Value {
    let spec = build_hardware_ssot_spec();
    let filter_cat = optional_non_empty_str(args, "category").map(|s| s.to_lowercase());

    let commands: Vec<_> = spec
        .pico_commands
        .into_iter()
        .filter(|c| match &filter_cat {
            Some(f) => {
                c.category.to_lowercase().contains(f)
                    || c.id.to_lowercase().contains(f)
                    || c.syntax.to_lowercase().contains(f)
            }
            None => true,
        })
        .collect();

    json!({
        "thresholds": spec.thresholds,
        "usb_ids": spec.usb_ids,
        "indexed_manuals_count": spec.indexed_manuals_count,
        "pins": spec.pinout,
        "pinout": spec.pinout,
        "breakin_modes": spec.breakin_modes,
        "flash_geometries": spec.flash_geometries,
        "memory_regions": spec.memory_regions,
        "pico_commands": commands,
    })
}

pub fn handle_project_status(args: &Value, workspace_root: &Path) -> Value {
    let include_rag_stats = args
        .get("include_rag_stats")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let manuals_dir = workspace_root.join(DEFAULT_MANUALS_DIR);
    let pdf_count = count_pdf_files(&manuals_dir);
    let catalog = get_diga_model_catalog_dto();

    let rag_stats = if include_rag_stats {
        serde_json::to_value(panadiga_rag::corpus_stats()).unwrap_or(Value::Null)
    } else {
        Value::Null
    };

    json!({
        "workspace_root": workspace_root.to_string_lossy(),
        "cargo_workspace_crates": [
            "panadiga-core",
            "panadiga-rag",
            "panadiga-mcp",
            "panadiga-cli",
            "panadiga-desktop"
        ],
        "manuals_pdf_count": pdf_count,
        "supported_diga_models": catalog.len(),
        "diga_models": catalog,
        "rag_corpus_stats": rag_stats,
    })
}
