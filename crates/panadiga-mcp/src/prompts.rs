//! MCP Prompts (`prompts/list`, `prompts/get`) providing structured RE and recovery runbooks.

use anyhow::{Result, bail};
use serde_json::{Value, json};

pub fn list_prompts() -> Vec<Value> {
    vec![
        json!({
            "name": "u99_nvram_triage",
            "description": "Guided workflow to verify a Flash dump, audit the 64KB NVRAM (0x807F0000), and create a safe sanitized copy for U99 recovery.",
            "arguments": [
                { "name": "input_file", "description": "Path to 8MB Spansion dump", "required": false }
            ]
        }),
        json!({
            "name": "p7501_uart_diagnostics",
            "description": "Primary DMR-EH55 P7501 Async UART (115200 8N1) & discrete Boot/Reset workflow (PINCHECK, BUSDIAG, UARTMON, JIGBOOT, UARTSEND with SW_UART=Hi-Z) plus visual schematic lookup.",
            "arguments": [
                { "name": "serial_port", "description": "Serial port of Raspberry Pi Pico probe (e.g. COM10, /dev/ttyACM0)", "required": false }
            ]
        }),
        json!({
            "name": "cddiag_p7501_capture",
            "description": "Guided bench procedure for P7501 Async UART targets (AM34 / VEP79138) after XRESET + SW_UART sequence, capturing boot telemetry and NOR Flash via LIVE_CAPTURE or GDB RSP.",
            "arguments": [
                { "name": "serial_port", "description": "Serial port of Raspberry Pi Pico probe (e.g. COM10, /dev/ttyACM0)", "required": false }
            ]
        }),
        json!({
            "name": "frm_crypto_re",
            "description": "Guided reverse-engineering workflow for PANA_DVD.FRM TOC structure, entropy distribution, and Ghidra AM34 memory map import.",
            "arguments": [
                { "name": "frm_path", "description": "Path to PANA_DVD.FRM container", "required": false }
            ]
        }),
    ]
}

pub fn get_prompt(name: &str, args: &Value) -> Result<Value> {
    match name {
        "u99_nvram_triage" => {
            let input_file = args
                .get("input_file")
                .and_then(|v| v.as_str())
                .unwrap_or("firmware_and_images/<dump_8mb>.bin");
            Ok(json!({
                "description": "Surgical U99 NVRAM unbrick runbook enforcing IPL read-only and non-destructive output invariants.",
                "messages": [{
                    "role": "user",
                    "content": {
                        "type": "text",
                        "text": format!(
                            "Execute a U99 NVRAM safety triage on '{}':\n\
                             1. Call `panadiga_safety_check` for address `0x807F0000` (mutable) vs `0x80000000` (IPL E05 lockout).\n\
                             2. Call `panadiga_clean_nvram` with only `input_file` first (analysis-only) to inspect SHA-256 and non-0xFF micro-sectors.\n\
                             3. Only write to a separate `output_file` (never in-place) after verifying `ipl_sha256`.",
                            input_file
                        )
                    }
                }]
            }))
        }
        "p7501_uart_diagnostics" => {
            let port = args
                .get("serial_port")
                .and_then(|v| v.as_str())
                .unwrap_or("COM10");
            Ok(json!({
                "description": "Primary DMR-EH55 P7501 115200 8N1 UART & discrete Boot/Reset diagnostic workflow.",
                "messages": [{
                    "role": "user",
                    "content": {
                        "type": "text",
                        "text": format!(
                            "Diagnose DMR-EH55 via P7501 UART on port {}:\n\
                             1. Read `panadiga://pinout-p7501-uart` and `panadiga://visual-catalog` (use `panadiga_search_visuals` for `P7501`, `Q7510`, `Q7514`, `IC1506` schematics).\n\
                             2. Enforce Pin 5 (NSW5V -> GP3) strictly Hi-Z Input with pulls disabled.\n\
                             3. Run `python -m scripts.bench.runner health --port {}` (`PING`, `STATUS`, `PINCHECK`) and verify GP26 ADC0 VDD (~2.7V) without sticky LOW.\n\
                             4. Capture live AM34 boot logs via `UARTMON 20000` (normal mode) or `JIGBOOT` (`SW_UART=LOW`), and transmit UART payloads only with `UARTSEND <str>` (`SW_UART=Hi-Z` so Q7514 conducts). Always finish with `ISOLATE`.",
                            port, port
                        )
                    }
                }]
            }))
        }
        "cddiag_p7501_capture" => {
            let port = args
                .get("serial_port")
                .and_then(|v| v.as_str())
                .unwrap_or("COM10");
            Ok(json!({
                "description": "CDDIAG / P7501 UART boot-telemetry inspection and live Spansion NOR Flash capture workflow.",
                "messages": [{
                    "role": "user",
                    "content": {
                        "type": "text",
                        "text": format!(
                            "Procedure for P7501 Async UART target (115200 8N1) on port {} (see `p7501_uart_diagnostics` for full probe/send menu):\n\
                             1. Verify pinout & voltage limits (`panadiga://pinout-p7501-uart` & `panadiga://invariants`).\n\
                             2. Run `panadiga pico status --port {}` or `panadiga pico breakin --port {} --mode 2` to inspect live DCSR.\n\
                             3. Capture live NOR Flash blocks with `panadiga pico dump --port {} --addr 0x807F0000 --words 64` or open `panadiga gdb-bridge --port {}` and send `$m80000000,0040#` / `$m807F0000,0040#`.",
                            port, port, port, port, port
                        )
                    }
                }]
            }))
        }
        "frm_crypto_re" => {
            let frm_path = args
                .get("frm_path")
                .and_then(|v| v.as_str())
                .unwrap_or("firmware_and_images/PANA_DVD.FRM");
            Ok(json!({
                "description": "Reverse-engineering inspection of PANA_DVD.FRM modules and AM34 Ghidra symbol mapping.",
                "messages": [{
                    "role": "user",
                    "content": {
                        "type": "text",
                        "text": format!(
                            "Analyze container '{}':\n\
                             1. Run `panadiga_inspect_frm` on '{}'.\n\
                             2. Run `panadiga_ghidra_symbols` to inspect Matsushita AM34 vector table and I/O symbols.\n\
                             3. Run `panadiga_search_manuals` for AM34 / MN103S / P7501 / CPRM references.",
                            frm_path, frm_path
                        )
                    }
                }]
            }))
        }
        _ => bail!("Unknown Panadiga prompt: {}", name),
    }
}
