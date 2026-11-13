"""Unit tests for cross-platform scripts/ helpers (ops, build, validation)."""

from __future__ import annotations

from pathlib import Path
import pytest

from scripts.build import _detect_android_sdk, _parse_args
from scripts.panadiga_ops import (
    find_msvc_vcvars64,
    load_msvc_env,
    locate_panadiga_binary,
    locate_panadiga_desktop_binary,
    panadiga_binary_name,
    panadiga_desktop_binary_name,
    resolve_executable,
    sha256_file,
    update_sha256_manifest,
)


def test_sha256_manifest_is_idempotent(tmp_path: Path) -> None:
    artifact = tmp_path / "panadiga-test.bin"
    artifact.write_bytes(b"PANASONIC-AM34-TEST")
    digest = sha256_file(artifact).upper()

    manifest = tmp_path / "SHA256SUMS.txt"
    manifest.write_text("0000  other.zip\nDEADBEEF  panadiga-test.bin\n", encoding="utf-8")

    update_sha256_manifest(manifest, [artifact])
    lines = manifest.read_text(encoding="utf-8").splitlines()
    assert lines == ["0000  other.zip", f"{digest}  panadiga-test.bin"]


def test_detect_android_sdk_from_environment(tmp_path: Path) -> None:
    sdk_dir = tmp_path / "Android" / "Sdk"
    ndk_v1 = sdk_dir / "ndk" / "26.1.10909125"
    ndk_v2 = sdk_dir / "ndk" / "27.0.12077973"
    ndk_v1.mkdir(parents=True)
    ndk_v2.mkdir(parents=True)

    detected_sdk, detected_ndk = _detect_android_sdk({"ANDROID_HOME": str(sdk_dir)})
    assert detected_sdk == sdk_dir
    assert detected_ndk == ndk_v2


def test_resolve_executable_raises_for_missing_command() -> None:
    with pytest.raises(FileNotFoundError):
        resolve_executable("__panadiga_nonexistent_tool_xyz__")


def test_build_cli_arg_parsing() -> None:
    args = _parse_args(["--desktop", "--pico"])
    assert args.desktop is True
    assert args.pico is True
    assert args.android is False


def test_workspace_binary_names_and_locators() -> None:
    cli_name = panadiga_binary_name()
    gui_name = panadiga_desktop_binary_name()
    assert cli_name.startswith("panadiga")
    assert gui_name.startswith("panadiga-desktop")
    assert callable(locate_panadiga_binary)
    assert callable(locate_panadiga_desktop_binary)
    vcvars = find_msvc_vcvars64()
    assert vcvars is None or vcvars.name.lower() == "vcvars64.bat"
    augmented = load_msvc_env({"PATH": ""})
    assert "PATH" in augmented
