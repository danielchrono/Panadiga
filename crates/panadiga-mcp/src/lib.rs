//! Native Rust Model Context Protocol (MCP) Server for Panadiga.
//!
//! Modularized into protocol transport, resources, prompts, hardware tools,
//! forensic tools, and knowledge/RAG search tools over JSON-RPC 2.0 stdio.

pub mod prompts;
pub mod protocol;
pub mod resources;
pub mod tools;
pub mod workspace;

pub use protocol::handle_jsonrpc_line;
pub use tools::{execute_tool, tool_definitions};
pub use workspace::resolve_workspace_root;

use anyhow::Result;
use std::io::{self, BufRead, Write};
use std::path::Path;

/// Runs the JSON-RPC 2.0 MCP loop against generic buffered streams (used in stdio & tests).
pub fn run_stdio_with_io<R: BufRead, W: Write>(
    mut reader: R,
    mut writer: W,
    workspace_root: &Path,
) -> Result<()> {
    let mut line = String::new();
    loop {
        line.clear();
        let bytes = reader.read_line(&mut line)?;
        if bytes == 0 {
            break;
        }
        if let Some(resp) = handle_jsonrpc_line(&line, workspace_root) {
            writeln!(writer, "{}", serde_json::to_string(&resp)?)?;
            writer.flush()?;
        }
    }
    Ok(())
}

/// Starts the native Rust MCP CLI/stdio server while isolating logger output to stderr.
pub fn run_mcp_server(show_tools: bool, ping_only: bool) -> Result<()> {
    // CRITICAL: Keep stdout strictly clean for JSON-RPC 2.0 protocol framing.
    panadiga_core::logger::set_sink(|level, msg| {
        eprintln!("[{}] {}", level.as_str(), msg);
    });

    let mut stdout = io::stdout().lock();
    if show_tools {
        writeln!(
            stdout,
            "{}",
            serde_json::to_string_pretty(&tool_definitions())?
        )?;
        stdout.flush()?;
        return Ok(());
    }
    if ping_only {
        writeln!(stdout, "PONG - Panadiga Native Rust MCP Server OK")?;
        stdout.flush()?;
        return Ok(());
    }

    let stdin = io::stdin().lock();
    let root = resolve_workspace_root();
    run_stdio_with_io(stdin, stdout, &root)
}

#[cfg(test)]
mod tests {
    use super::*;
    use panadiga_core::constants::{
        AM34_SYSTEM_MEMORY_REGIONS, SPANSION_BOOTLOADER_SIZE_BYTES, SPANSION_FLASH_SIZE_BYTES,
        SPANSION_NVRAM_BASE_OFFSET,
    };
    use serde_json::{Value, json};
    use std::fs;
    use std::io::Cursor;

    #[test]
    fn test_tool_definitions_and_memory_map() {
        let defs = tool_definitions();
        assert_eq!(defs.len(), 13);

        let all = execute_tool("panadiga_memory_map", &json!({}), Path::new(".")).unwrap();
        assert_eq!(all["total"], AM34_SYSTEM_MEMORY_REGIONS.len());

        let nvram = execute_tool(
            "panadiga_memory_map",
            &json!({ "filter_name": "nvram_u99" }),
            Path::new("."),
        )
        .unwrap();
        assert_eq!(nvram["total"], 1);
        assert_eq!(nvram["regions"][0]["start"], "0x807F0000");
    }

    #[test]
    fn test_safety_check_invariants() {
        let overvolt = execute_tool(
            "panadiga_safety_check",
            &json!({ "voltage": 5.0 }),
            Path::new("."),
        )
        .unwrap();
        assert_eq!(overvolt["safe"], false);
        assert!(
            overvolt["violations"][0]
                .as_str()
                .unwrap()
                .contains("VOLTAGE_VIOLATION")
        );

        let safe_volt = execute_tool(
            "panadiga_safety_check",
            &json!({ "voltage": 3.3 }),
            Path::new("."),
        )
        .unwrap();
        assert_eq!(safe_volt["safe"], true);

        let ipl_write = execute_tool(
            "panadiga_safety_check",
            &json!({ "address": "0x80000100" }),
            Path::new("."),
        )
        .unwrap();
        assert_eq!(ipl_write["safe"], false);
        assert!(
            ipl_write["violations"][0]
                .as_str()
                .unwrap()
                .contains("FLASH_WRITE_VIOLATION")
        );

        let nvram_write = execute_tool(
            "panadiga_safety_check",
            &json!({ "address": "0x807F0000" }),
            Path::new("."),
        )
        .unwrap();
        assert_eq!(nvram_write["safe"], true);
        assert!(
            nvram_write["address_status"]
                .as_str()
                .unwrap()
                .contains("mutable NVRAM")
        );
    }

    #[test]
    fn test_clean_nvram_rejects_in_place_and_sanitizes_copy() {
        let tmp = std::env::temp_dir().join(format!("panadiga_mcp_nvram_{}", std::process::id()));
        fs::create_dir_all(&tmp).unwrap();

        let in_path = tmp.join("flash_in.bin");
        let out_path = tmp.join("flash_out.bin");

        let mut flash = vec![0xA5u8; SPANSION_FLASH_SIZE_BYTES];
        flash[SPANSION_NVRAM_BASE_OFFSET] = 0x11;
        fs::write(&in_path, &flash).unwrap();

        let same_res = execute_tool(
            "panadiga_clean_nvram",
            &json!({
                "input_file": "flash_in.bin",
                "output_file": "flash_in.bin"
            }),
            &tmp,
        )
        .unwrap();
        assert!(
            same_res["error"]
                .as_str()
                .unwrap()
                .contains("Forensic Invariant Violation")
        );

        let clean_res = execute_tool(
            "panadiga_clean_nvram",
            &json!({
                "input_file": "flash_in.bin",
                "output_file": "flash_out.bin"
            }),
            &tmp,
        )
        .unwrap();
        assert_eq!(clean_res["status"], "success");
        assert_eq!(clean_res["sanitized_file_created"], true);
        assert_eq!(
            clean_res["ipl_bytes_checked"],
            SPANSION_BOOTLOADER_SIZE_BYTES
        );

        let cleaned_bytes = fs::read(&out_path).unwrap();
        assert_eq!(
            &cleaned_bytes[..SPANSION_BOOTLOADER_SIZE_BYTES],
            &flash[..SPANSION_BOOTLOADER_SIZE_BYTES]
        );
        assert!(
            cleaned_bytes[SPANSION_NVRAM_BASE_OFFSET..SPANSION_FLASH_SIZE_BYTES]
                .iter()
                .all(|&b| b == 0xFF)
        );

        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_new_dev_tools_and_resources_prompts() {
        let catalog = execute_tool("panadiga_pico_catalog", &json!({}), Path::new(".")).unwrap();
        assert!(!catalog["pico_commands"].as_array().unwrap().is_empty());
        assert_eq!(catalog["pins"].as_array().unwrap().len(), 6);

        let ghidra = execute_tool(
            "panadiga_ghidra_symbols",
            &json!({ "filter": "nvram" }),
            Path::new("."),
        )
        .unwrap();
        assert!(!ghidra["memory_blocks"].as_array().unwrap().is_empty());

        let res_invariants =
            resources::read_resource("panadiga://invariants", Path::new(".")).unwrap();
        let text = res_invariants["contents"][0]["text"].as_str().unwrap();
        assert!(text.contains("0x807F0000"));
        let res_visuals =
            resources::read_resource("panadiga://visual-catalog", Path::new(".")).unwrap();
        let vis_manifest_text = res_visuals["contents"][0]["text"].as_str().unwrap();
        assert!(vis_manifest_text.contains("na_p62_zoom.png"));
        let vis_search = execute_tool(
            "panadiga_search_visuals",
            &json!({ "query": "Q7514", "limit": 5 }),
            Path::new("."),
        )
        .unwrap();
        assert!(vis_search["matched"].as_u64().unwrap() >= 1);
        assert!(
            resources::read_resource("panadiga://docs/../../etc/passwd", Path::new(".")).is_err()
        );

        let prompt =
            prompts::get_prompt("cddiag_p7501_capture", &json!({ "serial_port": "COM10" }))
                .unwrap();
        assert!(
            prompt["messages"][0]["content"]["text"]
                .as_str()
                .unwrap()
                .contains("COM10")
        );

        let p7501_prompt =
            prompts::get_prompt("p7501_uart_diagnostics", &json!({ "serial_port": "COM10" }))
                .unwrap();
        let p7501_text = p7501_prompt["messages"][0]["content"]["text"]
            .as_str()
            .unwrap();
        assert!(p7501_text.contains("COM10"));
        assert!(p7501_text.contains("P7501"));
    }

    #[test]
    fn test_jsonrpc_stdio_protocol_and_capabilities() {
        let input = concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{}}\n",
            "{\"jsonrpc\":\"2.0\",\"method\":\"notifications/initialized\"}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\"}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"panadiga_safety_check\",\"arguments\":{\"voltage\":5.0,\"address\":\"0x80000000\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":4,\"method\":\"resources/list\"}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":5,\"method\":\"prompts/list\"}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":6,\"method\":\"ping\"}\n"
        );
        let mut output = Vec::new();
        run_stdio_with_io(Cursor::new(input.as_bytes()), &mut output, Path::new(".")).unwrap();

        let text = String::from_utf8(output).unwrap();
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 6);

        let r1: Value = serde_json::from_str(lines[0]).unwrap();
        assert_eq!(r1["result"]["serverInfo"]["name"], "panadiga-mcp-server");
        assert!(r1["result"]["capabilities"]["resources"].is_object());
        assert!(r1["result"]["capabilities"]["prompts"].is_object());

        let r2: Value = serde_json::from_str(lines[1]).unwrap();
        assert_eq!(r2["result"]["tools"].as_array().unwrap().len(), 13);

        let r3: Value = serde_json::from_str(lines[2]).unwrap();
        let body = r3["result"]["content"][0]["text"].as_str().unwrap();
        assert!(body.contains("VOLTAGE_VIOLATION"));
        assert!(body.contains("FLASH_WRITE_VIOLATION"));

        let r4: Value = serde_json::from_str(lines[3]).unwrap();
        assert!(r4["result"]["resources"].as_array().unwrap().len() >= 5);

        let r5: Value = serde_json::from_str(lines[4]).unwrap();
        assert_eq!(r5["result"]["prompts"].as_array().unwrap().len(), 4);

        let r6: Value = serde_json::from_str(lines[5]).unwrap();
        assert_eq!(r6["id"], 6);
    }
}
