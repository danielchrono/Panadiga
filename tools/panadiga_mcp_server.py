#!/usr/bin/env python3
"""Panadiga Model Context Protocol (MCP) Server.

Exposes native Panadiga engineering, storage forensics, hardware safety checks,
and manual lookup tools to AI agents over standard JSON-RPC 2.0 stdio transport.
"""

from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
from typing import Any, Dict, List, Optional


WORKSPACE_ROOT = Path(
    os.environ.get("PANADIGA_WORKSPACE_ROOT", Path(__file__).resolve().parent.parent)
)
if str(WORKSPACE_ROOT) not in sys.path:
    sys.path.insert(0, str(WORKSPACE_ROOT))

_AM34_TUPLES = (
    ("INTERNAL_SRAM", 0x00000000, 0x0003FFFF, 256, "rwx"),
    ("CORE_MMIO", 0x10000000, 0x1000FFFF, 64, "rw-"),
    ("MAIN_SDRAM", 0x40000000, 0x43FFFFFF, 65536, "rwx"),
    ("FLASH_NOR_IPL", 0x80000000, 0x807EFFFF, 8128, "r-x"),
    ("NVRAM_U99", 0x807F0000, 0x807FFFFF, 64, "rw-"),
    ("SYSTEM_MMIO", 0xC0000000, 0xC0003FFF, 16, "rw-"),
    ("DWIRE32A_OCD", 0xC0004000, 0xC00040FF, 1, "rw-"),
    ("ATA_IDE_MMIO", 0xD4000000, 0xD4000FFF, 4, "rw-"),
)

AM34_REGIONS: List[Dict[str, Any]] = [
    {
        "name": t[0],
        "start": f"0x{t[1]:08X}",
        "end": f"0x{t[2]:08X}",
        "size_kb": t[3],
        "perms": t[4],
    }
    for t in _AM34_TUPLES
]

TOOLS_DEFINITIONS: List[Dict[str, Any]] = [
    {
        "name": "panadiga_memory_map",
        "description": "Returns the official physical memory map of the Matsushita AM34 / UniPhier MN2DS0016 processor.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "filter_name": {
                    "type": "string",
                    "description": "Optional region name filter (e.g. NVRAM_U99, FLASH_NOR_IPL)",
                }
            },
        },
    },
    {
        "name": "panadiga_inspect_hdd",
        "description": "Inspects a Panasonic DMR-EH55 MEIHFS HDD binary dump: validates MBR tag 'SH', mirror sector at 28MB, and partition layout.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "image_path": {
                    "type": "string",
                    "description": "Path to the HDD dump image. Defaults to standard 100MB dump.",
                }
            },
        },
    },
    {
        "name": "panadiga_safety_check",
        "description": "Validates electrical voltage and memory write addresses against deterministic Panadiga hardware invariants.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "voltage": {
                    "type": "number",
                    "description": "Voltage value in Volts to test against the 3.3V maximum safety threshold.",
                },
                "address": {
                    "type": "string",
                    "description": "Hexadecimal memory bus address (e.g. '0x807F0000') to test for write permission.",
                },
            },
        },
    },
    {
        "name": "panadiga_clean_nvram",
        "description": "Analyzes or performs surgical 64KB NVRAM sanitization (U99 unbrick) on a Spansion S29JL064H 8MB flash dump.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "input_file": {
                    "type": "string",
                    "description": "Path to original 8MB flash dump file.",
                },
                "output_file": {
                    "type": "string",
                    "description": "Optional path where sanitized flash file should be saved with NVRAM reset to 0xFF.",
                },
            },
            "required": ["input_file"],
        },
    },
    {
        "name": "panadiga_search_manuals",
        "description": "Searches terms or error codes (such as U99, CN55001, MEIHFS) in the 15 Panasonic DMR-EH55 service manuals.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "Search term or hardware identifier.",
                }
            },
            "required": ["query"],
        },
    },
    {
        "name": "panadiga_rag_query",
        "description": "Runs a guarded query against the repository's RAG knowledge engine and safety critic.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "Technical query regarding Panasonic DIGA engineering.",
                }
            },
            "required": ["query"],
        },
    },
]


def handle_memory_map(args: Dict[str, Any]) -> Dict[str, Any]:
    filter_name = args.get("filter_name")
    if filter_name:
        fn = str(filter_name).upper()
        res = [r for r in AM34_REGIONS if fn in r["name"]]
    else:
        res = AM34_REGIONS
    return {"regions": res, "total": len(res)}


def handle_inspect_hdd(args: Dict[str, Any]) -> Dict[str, Any]:
    rel_path = args.get(
        "image_path", "firmware_and_images/panasonic DMR EH-55 Firmware hdd_100.bin"
    )
    img_file = WORKSPACE_ROOT / rel_path
    if not img_file.is_file():
        return {"error": f"Image file not found: {img_file}"}

    size_bytes = img_file.stat().st_size
    with img_file.open("rb") as f:
        mbr_data = f.read(512)
        has_sh_tag = mbr_data[6:8] == b"SH"
        guid_bytes = mbr_data[8:16]
        guid_hex = guid_bytes.hex()

        # Check mirror at 28 MB (57344 * 512 = 29360128)
        f.seek(57344 * 512)
        mirror_data = f.read(512)
        mirror_matches = (mbr_data == mirror_data)

    return {
        "image_path": str(img_file),
        "size_bytes": size_bytes,
        "size_mb": round(size_bytes / (1024 * 1024), 2),
        "mbr_tag_sh_valid": has_sh_tag,
        "disk_guid": guid_hex,
        "mirror_sector_57344_identical": mirror_matches,
        "known_partitions": [
            {"name": "Journaling / Catalog", "offset_mb": 8, "lba": 16384},
            {"name": "Firmware Primary", "offset_mb": 32, "magic": "5187138d5b0afb22"},
            {"name": "Firmware Backup", "offset_mb": 64, "magic": "5187138d5b0afb22"},
        ],
    }


def handle_safety_check(args: Dict[str, Any]) -> Dict[str, Any]:
    results: Dict[str, Any] = {"safe": True, "violations": []}

    voltage = args.get("voltage")
    if voltage is not None:
        v_float = float(voltage)
        if v_float > 3.3:
            results["safe"] = False
            results["violations"].append(
                f"VOLTAGE_VIOLATION: {v_float}V exceeds maximum safe threshold of 3.3V (LVTTL)!"
            )
        else:
            results["voltage_status"] = f"{v_float}V is within safe bounds (<= 3.3V)."

    address_str = args.get("address")
    if address_str is not None:
        try:
            raw_addr = str(address_str).strip()
            addr = int(raw_addr, 16) if raw_addr.lower().startswith("0x") else int(raw_addr)
            if (0x00000000 <= addr <= 0x007EFFFF) or (0x80000000 <= addr <= 0x807EFFFF):
                results["safe"] = False
                results["violations"].append(
                    f"FLASH_WRITE_VIOLATION: Address 0x{addr:08X} resides in protected IPL/Bootloader area! Write forbidden."
                )
            elif (0x007F0000 <= addr <= 0x007FFFFF) or (0x807F0000 <= addr <= 0x807FFFFF):
                results["address_status"] = f"Address 0x{addr:08X} is inside mutable NVRAM (U99 unbrick area)."
            else:
                results["address_status"] = f"Address 0x{addr:08X} is outside NOR Flash range."
        except ValueError:
            results["safe"] = False
            results["violations"].append(f"Invalid address format: {address_str}")

    return results


def handle_clean_nvram(args: Dict[str, Any]) -> Dict[str, Any]:
    input_path = WORKSPACE_ROOT / args["input_file"]
    if not input_path.is_file():
        return {"error": f"Input flash dump not found: {input_path}"}

    data = bytearray(input_path.read_bytes())
    if len(data) != 8388608:
        return {"error": f"Invalid flash dump size: {len(data)} bytes (expected 8388608 / 8 MB)"}

    ipl_hash = hashlib.sha256(data[0:8323072]).hexdigest()
    nvram_region = data[0x7F0000:0x800000]
    nvram_corrupt_hash = hashlib.sha256(nvram_region).hexdigest()

    output_path_str = args.get("output_file")
    saved = False
    if output_path_str:
        out_path = WORKSPACE_ROOT / output_path_str
        if out_path.resolve() == input_path.resolve():
            return {"error": "Forensic Invariant Violation: Cannot overwrite original flash dump in-place."}
        # Replace 64KB with 0xFF
        data[0x7F0000:0x800000] = b"\xFF" * 65536
        out_path.write_bytes(data)
        saved = True

    return {
        "status": "success",
        "ipl_bytes_checked": 8323072,
        "ipl_sha256": ipl_hash,
        "nvram_original_sha256": nvram_corrupt_hash,
        "is_all_ff": all(b == 0xFF for b in nvram_region),
        "sanitized_file_created": saved,
        "output_file": output_path_str if saved else None,
    }


def handle_search_manuals(args: Dict[str, Any]) -> Dict[str, Any]:
    query = args["query"].lower()
    manuals_dir = WORKSPACE_ROOT / "manuals_and_schematics"
    matches: List[Dict[str, str]] = []

    if manuals_dir.is_dir():
        for p in manuals_dir.glob("*.pdf"):
            if query in p.name.lower():
                matches.append({"file": p.name, "match_type": "filename"})

    return {
        "query": query,
        "direct_matches": matches,
        "total_manuals_in_catalog": len(list(manuals_dir.glob("*.pdf"))) if manuals_dir.is_dir() else 0,
        "note": "Use 'panadiga_rag_query' for in-depth OCR and semantic analysis of manual content.",
    }


def _resolve_panadiga_cmd() -> List[str]:
    candidates = [
        WORKSPACE_ROOT / "target" / "debug" / "panadiga.exe",
        WORKSPACE_ROOT / "target" / "release" / "panadiga.exe",
        WORKSPACE_ROOT / "target" / "debug" / "panadiga",
        WORKSPACE_ROOT / "target" / "release" / "panadiga",
    ]
    existing = [c for c in candidates if c.is_file()]
    if existing:
        existing.sort(key=lambda p: p.stat().st_mtime, reverse=True)
        return [str(existing[0])]
    return ["cargo", "run", "--quiet", "--"]


def handle_rag_query(args: Dict[str, Any]) -> Dict[str, Any]:
    query = str(args["query"])
    try:
        cmd = _resolve_panadiga_cmd() + ["rag", "query", "--json", query]
        proc = subprocess.run(
            cmd,
            cwd=str(WORKSPACE_ROOT),
            capture_output=True,
            text=True,
            encoding="utf-8",
            check=True,
        )
        payload: Dict[str, Any] = json.loads(proc.stdout)
        safety_obj = payload.get("safety", {})
        passed_safety = bool(safety_obj.get("is_safe", True))
        return {
            "query": query,
            "answer": payload.get("answer", ""),
            "confidence": payload.get("confidence", 0.0),
            "groundedness_score": payload.get("groundedness_score", 0.0),
            "safety_passed": passed_safety,
            "is_abstained": payload.get("is_abstained", False),
            "citations": payload.get("citations", []),
        }
    except Exception as e:
        return {
            "query": query,
            "error": f"RAG execution fallback: {e}",
            "hint": "Run 'cargo build' to compile the native Rust panadiga RAG engine.",
        }


def execute_tool(name: str, arguments: Dict[str, Any]) -> Dict[str, Any]:
    if name == "panadiga_memory_map":
        return handle_memory_map(arguments)
    elif name == "panadiga_inspect_hdd":
        return handle_inspect_hdd(arguments)
    elif name == "panadiga_safety_check":
        return handle_safety_check(arguments)
    elif name == "panadiga_clean_nvram":
        return handle_clean_nvram(arguments)
    elif name == "panadiga_search_manuals":
        return handle_search_manuals(arguments)
    elif name == "panadiga_rag_query":
        return handle_rag_query(arguments)
    else:
        raise ValueError(f"Unknown tool: {name}")


def run_stdio() -> None:
    """Standard JSON-RPC 2.0 loop for Model Context Protocol."""
    while True:
        line = sys.stdin.readline()
        if not line:
            break
        line = line.strip()
        if not line:
            continue

        try:
            req = json.loads(line)
        except json.JSONDecodeError:
            continue

        req_id = req.get("id")
        method = req.get("method")
        params = req.get("params", {})

        if method == "initialize":
            resp = {
                "jsonrpc": "2.0",
                "id": req_id,
                "result": {
                    "protocolVersion": "2024-11-05",
                    "capabilities": {"tools": {}},
                    "serverInfo": {
                        "name": "panadiga-mcp-server",
                        "version": "1.0.0",
                    },
                },
            }
        elif method == "notifications/initialized" or req_id is None:
            continue
        elif method == "tools/list":
            resp = {
                "jsonrpc": "2.0",
                "id": req_id,
                "result": {"tools": TOOLS_DEFINITIONS},
            }
        elif method == "tools/call":
            tool_name = params.get("name", "")
            tool_args = params.get("arguments", {})
            try:
                out = execute_tool(tool_name, tool_args)
                resp = {
                    "jsonrpc": "2.0",
                    "id": req_id,
                    "result": {
                        "content": [
                            {"type": "text", "text": json.dumps(out, indent=2)}
                        ]
                    },
                }
            except Exception as ex:
                resp = {
                    "jsonrpc": "2.0",
                    "id": req_id,
                    "error": {"code": -32000, "message": str(ex)},
                }
        elif method == "ping":
            resp = {"jsonrpc": "2.0", "id": req_id, "result": {}}
        else:
            resp = {
                "jsonrpc": "2.0",
                "id": req_id,
                "error": {"code": -32601, "message": f"Method not found: {method}"},
            }

        sys.stdout.write(json.dumps(resp) + "\n")
        sys.stdout.flush()


def main() -> None:
    if "--tools" in sys.argv:
        print(json.dumps(TOOLS_DEFINITIONS, indent=2))
    elif "--ping" in sys.argv:
        print("PONG - Panadiga MCP Server OK")
    else:
        run_stdio()


if __name__ == "__main__":
    main()
