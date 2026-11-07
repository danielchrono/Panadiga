"""Unit tests for Panadiga Model Context Protocol (MCP) Server."""

from pathlib import Path
from typing import Any, Dict
import pytest
from tools.panadiga_mcp_server import (
    AM34_REGIONS,
    TOOLS_DEFINITIONS,
    execute_tool,
    handle_inspect_hdd,
    handle_memory_map,
    handle_rag_query,
    handle_safety_check,
)


def test_mcp_tools_definitions() -> None:
    assert len(TOOLS_DEFINITIONS) >= 5
    names = [t["name"] for t in TOOLS_DEFINITIONS]
    assert "panadiga_memory_map" in names
    assert "panadiga_safety_check" in names
    assert "panadiga_inspect_hdd" in names


def test_mcp_memory_map_tool() -> None:
    res_all = handle_memory_map({})
    assert res_all["total"] == len(AM34_REGIONS)

    res_filtered = handle_memory_map({"filter_name": "NVRAM"})
    assert res_filtered["total"] == 1
    assert res_filtered["regions"][0]["name"] == "NVRAM_U99"


def test_mcp_safety_check_voltage() -> None:
    safe_run = handle_safety_check({"voltage": 3.3})
    assert safe_run["safe"] is True

    unsafe_run = handle_safety_check({"voltage": 5.0})
    assert unsafe_run["safe"] is False
    assert any("VOLTAGE_VIOLATION" in v for v in unsafe_run["violations"])


def test_mcp_safety_check_flash_address() -> None:
    # 0x80000000 (bus) and 0x00000000 (file offset) are inside bootloader IPL (forbidden write)
    ipl_run = handle_safety_check({"address": "0x80000000"})
    assert ipl_run["safe"] is False
    assert any("FLASH_WRITE_VIOLATION" in v for v in ipl_run["violations"])

    ipl_offset_run = handle_safety_check({"address": "0x00000000"})
    assert ipl_offset_run["safe"] is False

    # 0x807F0000 and 0x007F0000 are inside NVRAM (safe for unbrick write)
    nvram_run = handle_safety_check({"address": "0x807F0000"})
    assert nvram_run["safe"] is True
    nvram_offset_run = handle_safety_check({"address": "0x007F0000"})
    assert nvram_offset_run["safe"] is True

    # Invalid hex address must mark safe=False
    bad_addr = handle_safety_check({"address": "0xZZZ"})
    assert bad_addr["safe"] is False


def test_mcp_inspect_hdd_and_rag_query() -> None:
    hdd_res = handle_inspect_hdd({})
    if "error" not in hdd_res:
        assert hdd_res["mbr_tag_sh_valid"] is True
        assert hdd_res["mirror_sector_57344_identical"] is True

    rag_res = handle_rag_query({"query": "What causes error U99 on Panasonic DMR-EH55?"})
    assert "error" not in rag_res
    assert rag_res["safety_passed"] is True
    assert rag_res["is_abstained"] is False


def test_mcp_execute_tool_dispatch() -> None:
    res = execute_tool("panadiga_memory_map", {})
    assert "regions" in res

    with pytest.raises(ValueError, match="Unknown tool"):
        execute_tool("invalid_tool_name", {})
