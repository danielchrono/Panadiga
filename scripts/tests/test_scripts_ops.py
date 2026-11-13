"""Unit tests for cross-platform scripts/ helpers (ops, build, validation)."""

from __future__ import annotations

from pathlib import Path
import pytest

from scripts.build import detect_android_sdk, parse_args
from scripts.panadiga_cache import (
    compute_tree_digest,
    copy_if_changed,
    discover_workspace_inventory,
    find_repo_root,
    is_step_fresh,
    record_step_cache,
)
from scripts.tui import PipelineSelection, build_execution_plan
from scripts.validate import parse_args as parse_validate_args
from scripts.panadiga_ops import (
    ProcessAbortedError,
    ProcessTracker,
    ROOT,
    active_process_tracker,
    find_msvc_vcvars64,
    load_msvc_env,
    locate_panadiga_binary,
    locate_panadiga_desktop_binary,
    panadiga_binary_name,
    panadiga_desktop_binary_name,
    resolve_executable,
    sha256_file,
    update_sha256_manifest,
    use_process_tracker,
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

    detected_sdk, detected_ndk = detect_android_sdk({"ANDROID_HOME": str(sdk_dir)})
    assert detected_sdk == sdk_dir
    assert detected_ndk == ndk_v2


def test_resolve_executable_raises_for_missing_command() -> None:
    with pytest.raises(FileNotFoundError):
        resolve_executable("__panadiga_nonexistent_tool_xyz__")


def test_build_cli_arg_parsing() -> None:
    args = parse_args(["--desktop", "--pico"])
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


def test_process_tracker_and_validate_args() -> None:
    parsed = parse_validate_args(["--sequential", "--jobs", "4"])
    assert parsed.sequential is True
    assert parsed.jobs == 4

    tracker = ProcessTracker()
    assert active_process_tracker() is None
    with use_process_tracker(tracker):
        assert active_process_tracker() is tracker
        assert tracker.is_aborted() is False
        tracker.terminate_all()
        assert tracker.is_aborted() is True
    assert active_process_tracker() is None
    assert issubclass(ProcessAbortedError, RuntimeError)


def test_recursive_root_cache_roundtrip_and_tui_plan(tmp_path: Path) -> None:
    nested = ROOT / "apps" / "desktop" / "src"
    assert find_repo_root(nested) == ROOT

    src_dir = tmp_path / "src"
    src_dir.mkdir()
    input_file = src_dir / "main.c"
    input_file.write_text("int main(void) { return 0; }\n", encoding="utf-8")
    out_file = tmp_path / "out.bin"
    out_file.write_bytes(b"PANADIGA-BIN")

    cache_dir = tmp_path / ".cache"
    digest = compute_tree_digest(tmp_path, [src_dir])
    assert not is_step_fresh(tmp_path, "demo", digest, [out_file], cache_dir=cache_dir)
    record_step_cache(tmp_path, "demo", digest, [out_file], cache_dir=cache_dir)
    assert is_step_fresh(tmp_path, "demo", digest, [out_file], cache_dir=cache_dir)

    out_file.write_bytes(b"MODIFIED")
    assert not is_step_fresh(tmp_path, "demo", digest, [out_file], cache_dir=cache_dir)

    mirror = tmp_path / "mirror.bin"
    assert copy_if_changed(out_file, mirror) is True
    assert copy_if_changed(out_file, mirror) is False

    inventory = discover_workspace_inventory(ROOT, {})
    assert "crates/panadiga-core" in inventory.crates
    assert "apps/desktop" in inventory.apps
    assert "firmware/pico-p7501" in inventory.firmware_targets

    plan = build_execution_plan(
        PipelineSelection(ci=True, frontend=True, pico=True, desktop=True, android=True),
        inventory,
        {},
    )
    assert any("PARALELO" in wave.concurrency for wave in plan)
    assert any("SERIAL" in wave.concurrency for wave in plan)

    args_tui = parse_args(["--tui", "--ci", "--no-cache"])
    assert args_tui.tui is True
    assert args_tui.ci is True
    assert args_tui.no_cache is True
