"""Unified cross-platform build, packaging, Pico UF2, Android APK, and launch runner."""

from __future__ import annotations

import argparse
import os
import platform
import shutil
import subprocess
import sys
import tarfile
import zipfile
from pathlib import Path
from typing import Final, Mapping, Sequence

SCRIPT_DIR = Path(__file__).resolve().parent
if str(SCRIPT_DIR.parent) not in sys.path:
    sys.path.insert(0, str(SCRIPT_DIR.parent))

from scripts.panadiga_ops import (
    ROOT,
    TOOLS_PYCACHE_IGNORE,
    build_env,
    locate_panadiga_binary,
    panadiga_binary_name,
    print_banner,
    print_status,
    resolve_executable,
    run_step,
    update_sha256_manifest,
)

UF2_NAMES: Final[tuple[str, ...]] = (
    "panasonic_dwire32a_rp2040.uf2",
    "panasonic_dwire32a_rp2350.uf2",
    "panasonic_dwire32a_unified.uf2",
)
STALE_DIST_ITEMS: Final[tuple[str, ...]] = (
    "run.cmd",
    "run.ps1",
    "run.sh",
    "WebView2Loader.dll",
    "firmware",
)


def _stage_tools_directory(dist_dir: Path) -> None:
    """Mirror runtime tools/ into dist/tools/ while excluding Python caches."""
    src_tools = ROOT / "tools"
    dst_tools = dist_dir / "tools"
    if dst_tools.exists():
        shutil.rmtree(dst_tools)
    if src_tools.is_dir():
        ignore_fn = shutil.ignore_patterns(*TOOLS_PYCACHE_IGNORE, "*.pyc")
        shutil.copytree(src_tools, dst_tools, ignore=ignore_fn)


def _package_desktop_archive(dist_dir: Path, releases_dir: Path) -> Path:
    """Create platform release archive (.zip on Windows, .tar.gz on POSIX) from dist/."""
    releases_dir.mkdir(parents=True, exist_ok=True)
    machine = platform.machine().lower()
    arch = "x86_64" if machine in {"amd64", "x86_64"} else ("aarch64" if machine in {"arm64", "aarch64"} else machine)

    if sys.platform == "win32":
        archive_path = releases_dir / f"panadiga-windows-{arch}.zip"
        with zipfile.ZipFile(archive_path, "w", compression=zipfile.ZIP_DEFLATED) as zf:
            for item in sorted(dist_dir.rglob("*")):
                if item.is_file() and releases_dir not in item.parents and item != archive_path:
                    zf.write(item, arcname=item.relative_to(dist_dir))
        return archive_path

    os_slug = "macos" if sys.platform == "darwin" else "linux"
    archive_path = releases_dir / f"panadiga-{os_slug}-{arch}.tar.gz"
    with tarfile.open(archive_path, "w:gz") as tf:
        for item in sorted(dist_dir.iterdir()):
            if item != releases_dir:
                tf.add(item, arcname=item.name)
    return archive_path


def build_desktop(env: Mapping[str, str]) -> Path:
    """Compile frontend bundle, export Ghidra SSOT, build Rust release, and stage dist/."""
    print_banner("BUILD DESKTOP: Unified Panadiga Engine (React + Tauri + Rust CLI)")
    run_step(["npm", "--prefix", "apps/desktop", "run", "build"], cwd=ROOT, env=env)
    run_step(["cargo", "run", "-p", "panadiga", "--", "ghidra", "export-all", "--dir", "tools"], cwd=ROOT, env=env)
    run_step(["cargo", "build", "--release", "-p", "panadiga"], cwd=ROOT, env=env)

    dist_dir = ROOT / "dist"
    releases_dir = dist_dir / "releases"
    dist_dir.mkdir(parents=True, exist_ok=True)

    for stale in STALE_DIST_ITEMS:
        target = dist_dir / stale
        if target.is_dir():
            shutil.rmtree(target)
        elif target.is_file():
            target.unlink()

    bin_name = panadiga_binary_name()
    built_bin = ROOT / "target" / "release" / bin_name
    if not built_bin.is_file():
        raise FileNotFoundError(f"Expected compiled binary missing: {built_bin}")
    dest_bin = dist_dir / bin_name
    shutil.copy2(built_bin, dest_bin)
    if sys.platform != "win32":
        dest_bin.chmod(dest_bin.stat().st_mode | 0o111)

    _stage_tools_directory(dist_dir)

    fw_bins_dir = ROOT / "src" / "pico" / "firmware_bins"
    for uf2_name in UF2_NAMES:
        uf2_src = fw_bins_dir / uf2_name
        if uf2_src.is_file():
            shutil.copy2(uf2_src, dist_dir / uf2_name)

    for doc_name in ("README.md", "LICENSE"):
        doc_src = ROOT / doc_name
        if doc_src.is_file():
            shutil.copy2(doc_src, dist_dir / doc_name)

    archive_path = _package_desktop_archive(dist_dir, releases_dir)
    update_sha256_manifest(releases_dir / "SHA256SUMS.txt", [archive_path])
    print_status("OK", f"Desktop release ready: {dest_bin} | Archive: {archive_path}", "32")
    return dest_bin


def _detect_android_sdk(env: Mapping[str, str]) -> tuple[Path, Path | None]:
    """Detect Android SDK and latest NDK directories cross-platform."""
    sdk_env = env.get("ANDROID_HOME") or env.get("ANDROID_SDK_ROOT")
    if sdk_env and Path(sdk_env).is_dir():
        sdk_dir = Path(sdk_env)
    else:
        fallback = (
            Path(env.get("LOCALAPPDATA", "")) / "Android" / "Sdk"
            if sys.platform == "win32"
            else Path.home() / "Android" / "Sdk"
        )
        if not fallback.is_dir():
            raise FileNotFoundError(
                "ANDROID_HOME not set and default Android SDK directory not found."
            )
        sdk_dir = fallback

    ndk_env = env.get("NDK_HOME") or env.get("ANDROID_NDK_ROOT")
    ndk_dir: Path | None = Path(ndk_env) if ndk_env and Path(ndk_env).is_dir() else None
    if ndk_dir is None and (sdk_dir / "ndk").is_dir():
        candidates = sorted((p for p in (sdk_dir / "ndk").iterdir() if p.is_dir()))
        if candidates:
            ndk_dir = candidates[-1]
    return sdk_dir, ndk_dir


def build_android(base_env: Mapping[str, str]) -> Path:
    """Compile Android ARM64 APK safely restoring Cargo.toml crate-type on completion."""
    print_banner("BUILD ANDROID: Tauri 2 Mobile APK (AArch64)")
    sdk_dir, ndk_dir = _detect_android_sdk(base_env)
    env = dict(base_env)
    env["ANDROID_HOME"] = str(sdk_dir)
    if ndk_dir is not None:
        env["NDK_HOME"] = str(ndk_dir)
        env["ANDROID_NDK_ROOT"] = str(ndk_dir)

    gen_android = ROOT / "gen" / "android"
    gen_android.mkdir(parents=True, exist_ok=True)
    props_lines = [f"sdk.dir={sdk_dir.as_posix()}"]
    if ndk_dir is not None:
        props_lines.append(f"ndk.dir={ndk_dir.as_posix()}")
    (gen_android / "local.properties").write_text("\n".join(props_lines) + "\n", encoding="utf-8")

    run_step(["rustup", "target", "add", "aarch64-linux-android"], cwd=ROOT, env=env)
    run_step(["npm", "--prefix", "apps/desktop", "run", "build"], cwd=ROOT, env=env)

    cargo_toml = ROOT / "Cargo.toml"
    original_toml = cargo_toml.read_text(encoding="utf-8")
    rlib_marker = 'crate-type = ["rlib"]'
    android_marker = 'crate-type = ["staticlib", "cdylib", "rlib"]'
    if rlib_marker in original_toml:
        cargo_toml.write_text(original_toml.replace(rlib_marker, android_marker, 1), encoding="utf-8")

    try:
        if not (gen_android / "app").is_dir():
            run_step(["npx", "@tauri-apps/cli", "android", "init", "--ci", "--skip-targets-install"], cwd=ROOT, env=env)
        run_step(["npx", "@tauri-apps/cli", "android", "build", "--apk", "--target", "aarch64"], cwd=ROOT, env=env)
    finally:
        # Guarantee Cargo.toml is always restored even when Android SDK/Gradle aborts
        cargo_toml.write_text(original_toml, encoding="utf-8")

    apk_candidates = sorted((gen_android / "app" / "build" / "outputs" / "apk").rglob("*.apk"))
    if not apk_candidates:
        raise FileNotFoundError("Android build completed but no .apk artifact was produced.")

    dist_dir = ROOT / "dist"
    releases_dir = dist_dir / "releases"
    releases_dir.mkdir(parents=True, exist_ok=True)
    dist_apk = dist_dir / "panadiga-android-arm64.apk"
    rel_apk = releases_dir / "panadiga-android-arm64.apk"
    shutil.copy2(apk_candidates[0], dist_apk)
    shutil.copy2(apk_candidates[0], rel_apk)
    update_sha256_manifest(releases_dir / "SHA256SUMS.txt", [rel_apk])
    print_status("OK", f"Android APK packaged at: {rel_apk}", "32")
    return rel_apk


def build_pico(env: Mapping[str, str]) -> None:
    """Compile RP2040 + RP2350 + Unified UF2 Pico firmware and sync firmware_bins/."""
    print_banner("BUILD PICO: DWire32A RP2040 + RP2350 + Unified Multi-Family UF2")
    fw_dir = ROOT / "dwire32a_jtag_research"
    if sys.platform == "win32":
        run_step(
            ["powershell", "-ExecutionPolicy", "Bypass", "-File", str(fw_dir / "build_pico_firmware.ps1"), "-Board", "all"],
            cwd=fw_dir,
            env=env,
        )
    else:
        for board, out_name in (("pico", "panasonic_dwire32a_rp2040.uf2"), ("pico2", "panasonic_dwire32a_rp2350.uf2")):
            bdir = fw_dir / f"build_{board}"
            run_step(["cmake", "-B", str(bdir), "-G", "Ninja", "-DCMAKE_BUILD_TYPE=Release", f"-DPICO_BOARD={board}"], cwd=fw_dir, env=env)
            run_step(["cmake", "--build", str(bdir), "--config", "Release"], cwd=fw_dir, env=env)
            shutil.copy2(bdir / "panasonic_dwire32a.uf2", fw_dir / out_name)
        run_step(
            [
                sys.executable,
                str(fw_dir / "merge_unified_uf2.py"),
                "--rp2040",
                str(fw_dir / "panasonic_dwire32a_rp2040.uf2"),
                "--rp2350",
                str(fw_dir / "panasonic_dwire32a_rp2350.uf2"),
                "--out",
                str(fw_dir / "panasonic_dwire32a_unified.uf2"),
            ],
            cwd=fw_dir,
            env=env,
        )

    dest_dir = ROOT / "src" / "pico" / "firmware_bins"
    dest_dir.mkdir(parents=True, exist_ok=True)
    for uf2_name in UF2_NAMES:
        src_uf2 = fw_dir / uf2_name
        if src_uf2.is_file():
            shutil.copy2(src_uf2, dest_dir / uf2_name)
    print_status("OK", f"Synced UF2 binaries into {dest_dir}", "32")


def launch_gui(env: Mapping[str, str], rebuild: bool = False) -> None:
    """Launch Desktop GUI detached, building desktop release first when missing."""
    binary = None if rebuild else locate_panadiga_binary()
    if binary is None:
        binary = build_desktop(env)

    print_status("INFO", f"Launching Panadiga Desktop: {binary}", "36")
    if sys.platform == "win32":
        flags = getattr(subprocess, "DETACHED_PROCESS", 0) | getattr(subprocess, "CREATE_NEW_PROCESS_GROUP", 0)
        subprocess.Popen([str(binary)], cwd=str(binary.parent), creationflags=flags)
    else:
        subprocess.Popen([str(binary)], cwd=str(binary.parent), start_new_session=True)


def run_cli(cli_args: Sequence[str], env: Mapping[str, str]) -> int:
    """Run native Panadiga CLI subcommand via compiled binary or cargo fallback."""
    binary = locate_panadiga_binary()
    if binary is not None:
        proc = subprocess.run([str(binary), *cli_args], cwd=str(ROOT), env=dict(env), check=False)
        return proc.returncode
    cargo_bin = resolve_executable("cargo", env=env)
    proc = subprocess.run([cargo_bin, "run", "-p", "panadiga", "--", *cli_args], cwd=str(ROOT), env=dict(env), check=False)
    return proc.returncode


def _parse_args(argv: Sequence[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Panadiga unified build, packaging, and launcher pipeline.")
    parser.add_argument("--desktop", action="store_true", help="Build Desktop release binary, staging dist/ and release archive.")
    parser.add_argument("--android", action="store_true", help="Build Android ARM64 APK via Tauri Mobile.")
    parser.add_argument("--pico", action="store_true", help="Build RP2040, RP2350, and unified Pico UF2 firmwares.")
    parser.add_argument("--all", action="store_true", help="Build Desktop and Android (plus Pico when ARM GCC exists).")
    parser.add_argument("--run", action="store_true", help="Launch Desktop GUI (auto-builds if missing).")
    parser.add_argument("--rebuild-run", action="store_true", help="Force rebuild Desktop release and launch GUI.")
    parser.add_argument("--cli", nargs=argparse.REMAINDER, help="Forward remaining arguments to the native Panadiga CLI.")
    return parser.parse_args(argv)


def main(argv: Sequence[str] | None = None) -> int:
    """Dispatch build/run/cli actions according to CLI flags."""
    args = _parse_args(argv)
    env = build_env()

    try:
        if args.cli is not None:
            return run_cli(args.cli, env)
        if args.rebuild_run:
            launch_gui(env, rebuild=True)
            return 0
        if args.run:
            launch_gui(env, rebuild=False)
            return 0
        if args.all:
            if shutil.which("arm-none-eabi-gcc", path=env.get("PATH")):
                build_pico(env)
            build_desktop(env)
            build_android(env)
            return 0
        if args.pico:
            build_pico(env)
            if not args.desktop and not args.android:
                return 0
        if args.android and not args.desktop:
            build_android(env)
            return 0

        build_desktop(env)
        if args.android:
            build_android(env)
        return 0
    except (subprocess.CalledProcessError, FileNotFoundError, RuntimeError) as exc:
        print_status("FAIL", f"Build pipeline failed: {exc}", "31")
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
