"""
Unit Tests for Panadiga Engineering Typer CLI
"""

import json
from pathlib import Path
import pytest
from typer.testing import CliRunner
from tools.panadiga_cli import app

runner = CliRunner()


def test_cli_help():
    result = runner.invoke(app, ["--help"])
    assert result.exit_code == 0
    assert "Panadiga Engineering CLI" in result.stdout
    assert "memory-map" in result.stdout
    assert "verify-artifact" in result.stdout
    assert "gdb-config" in result.stdout


def test_memory_map_json_format(tmp_path: Path):
    out_file = tmp_path / "memory_map.json"
    result = runner.invoke(app, ["memory-map", "--output", str(out_file), "--format", "json"])
    assert result.exit_code == 0
    assert out_file.exists()

    data = json.loads(out_file.read_text(encoding="utf-8"))
    assert len(data) >= 5
    region_names = [r["name"] for r in data]
    assert "FLASH_NOR" in region_names
    assert "BOOT_ROM" in region_names
    assert "INTERNAL_SRAM" in region_names


def test_verify_artifact(tmp_path: Path):
    dummy_file = tmp_path / "test_artifact.bin"
    dummy_file.write_bytes(b"Panasonic DIGA DMR-EH55 NVRAM Test Vector")

    # Run without expected hash
    result = runner.invoke(app, ["verify-artifact", str(dummy_file)])
    assert result.exit_code == 0
    assert "SHA-256" in result.stdout

    # Run with correct expected hash
    import hashlib
    expected = hashlib.sha256(b"Panasonic DIGA DMR-EH55 NVRAM Test Vector").hexdigest()
    res_correct = runner.invoke(app, ["verify-artifact", str(dummy_file), "--sha256", expected])
    assert res_correct.exit_code == 0
    assert "HASH CONFIRMADO" in res_correct.stdout

    # Run with mismatched expected hash
    res_fail = runner.invoke(app, ["verify-artifact", str(dummy_file), "--sha256", "deadbeef0000"])
    assert res_fail.exit_code == 2
    assert "DIVERGÊNCIA DE HASH" in res_fail.stdout


def test_gdb_config_command():
    result = runner.invoke(app, ["gdb-config", "--port", "COM7", "--tcp-port", "3333"])
    assert result.exit_code == 0
    assert "COM7" in result.stdout
    assert "3333" in result.stdout
    assert "target remote 127.0.0.1:3333" in result.stdout
