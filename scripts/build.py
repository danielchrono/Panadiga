"""Unified cross-platform build, packaging, Pico UF2, Android APK, and launch runner."""

from __future__ import annotations

import os
import sys

_HERE = os.path.dirname(os.path.abspath(__file__))
if _HERE in sys.path:
    sys.path.remove(_HERE)
_REPO_ROOT = os.path.dirname(_HERE)
if _REPO_ROOT not in sys.path:
    sys.path.insert(0, _REPO_ROOT)

import argparse
import platform
import shutil
import subprocess
import tarfile
import zipfile
from pathlib import Path
from typing import Final, Mapping, Sequence

from scripts.panadiga_ops import (
    ROOT,
    TOOLS_PYCACHE_IGNORE,
    build_env,
    load_msvc_env,
    locate_panadiga_binary,
    locate_panadiga_desktop_binary,
    panadiga_binary_name,
    panadiga_desktop_binary_name,
    print_banner,
    print_status,
    resolve_executable,
    run_step,
    update_sha256_manifest,
)

UF2_NAMES: Final[tuple[str, ...]] = (
    "panasonic_p7501_uart_rp2040.uf2",
    "panasonic_p7501_uart_rp2350.uf2",
    "panasonic_p7501_uart_unified.uf2",
)
STALE_DIST_ITEMS: Final[tuple[str, ...]] = (
    "run.cmd",
    "run.ps1",
    "run.sh",
    "WebView2Loader.dll",
    "firmware",
    "README.md",
)


def _stage_tools_directory(dist_dir: Path) -> None:
    """Stage clean runtime tools into dist/tools/ per target OS without tests or dev scripts."""
    src_tools = ROOT / "tools"
    dst_tools = dist_dir / "tools"
    if dst_tools.exists():
        shutil.rmtree(dst_tools)
    if not src_tools.is_dir():
        return

    dst_tools.mkdir(parents=True, exist_ok=True)
    ignore_fn = shutil.ignore_patterns(*TOOLS_PYCACHE_IGNORE, "*.pyc")

    ghidra_src = src_tools / "ghidra"
    if ghidra_src.is_dir():
        shutil.copytree(ghidra_src, dst_tools / "ghidra", ignore=ignore_fn)

    tools_readme = src_tools / "README.md"
    if tools_readme.is_file():
        shutil.copy2(tools_readme, dst_tools / "README.md")

    if sys.platform == "win32":
        for cmd_file in sorted(src_tools.glob("*.cmd")):
            shutil.copy2(cmd_file, dst_tools / cmd_file.name)
        hxd_src = src_tools / "HxD"
        if hxd_src.is_dir():
            shutil.copytree(hxd_src, dst_tools / "HxD", ignore=ignore_fn)


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


def _copy_release_binary(bin_name: str, dist_dir: Path) -> Path:
    """Copy a compiled target/release binary into dist/ with executable permissions on POSIX."""
    built_bin = ROOT / "target" / "release" / bin_name
    if not built_bin.is_file():
        raise FileNotFoundError(f"Expected compiled binary missing: {built_bin}")
    dest_bin = dist_dir / bin_name
    shutil.copy2(built_bin, dest_bin)
    if sys.platform != "win32":
        dest_bin.chmod(dest_bin.stat().st_mode | 0o111)
    return dest_bin


def build_desktop(env: Mapping[str, str]) -> Path:
    """Compile frontend bundle, export Ghidra SSOT, build Rust release binaries, and stage dist/."""
    print_banner("BUILD DESKTOP: Unified Panadiga Engine (React + Tauri + Rust CLI)")
    run_step(["npm", "--prefix", "apps/desktop", "run", "build"], cwd=ROOT, env=env)
    run_step(["cargo", "run", "-p", "panadiga-cli", "--", "ghidra", "export-all", "--dir", "tools/ghidra"], cwd=ROOT, env=env)
    run_step(["cargo", "build", "--release", "-p", "panadiga-cli", "-p", "panadiga-desktop"], cwd=ROOT, env=env)

    dist_dir = ROOT / "dist"
    releases_dir = dist_dir / "releases"
    dist_dir.mkdir(parents=True, exist_ok=True)

    for stale in STALE_DIST_ITEMS:
        target = dist_dir / stale
        if target.is_dir():
            shutil.rmtree(target)
        elif target.is_file():
            target.unlink()

    cli_bin = _copy_release_binary(panadiga_binary_name(), dist_dir)
    gui_bin = _copy_release_binary(panadiga_desktop_binary_name(), dist_dir)

    _stage_tools_directory(dist_dir)

    fw_bins_dir = ROOT / "crates" / "panadiga-core" / "src" / "pico" / "firmware_bins"
    for uf2_name in UF2_NAMES:
        uf2_src = fw_bins_dir / uf2_name
        if uf2_src.is_file():
            shutil.copy2(uf2_src, dist_dir / uf2_name)

    license_src = ROOT / "LICENSE"
    if license_src.is_file():
        shutil.copy2(license_src, dist_dir / "LICENSE")

    archive_path = _package_desktop_archive(dist_dir, releases_dir)
    update_sha256_manifest(releases_dir / "SHA256SUMS.txt", [archive_path])
    print_status("OK", f"Desktop release ready: {gui_bin} & {cli_bin} | Archive: {archive_path}", "32")
    return gui_bin


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


def _find_build_tool(sdk_dir: Path, tool_stem: str) -> Path:
    """Resolve the newest Android SDK build-tools executable (zipalign / apksigner)."""
    bt_dir = sdk_dir / "build-tools"
    if not bt_dir.is_dir():
        raise FileNotFoundError(f"Android build-tools directory missing under {sdk_dir}")
    exts = (".bat", ".exe", "") if sys.platform == "win32" else ("",)
    for ver_dir in sorted((p for p in bt_dir.iterdir() if p.is_dir()), reverse=True):
        for ext in exts:
            candidate = ver_dir / f"{tool_stem}{ext}"
            if candidate.is_file():
                return candidate
    raise FileNotFoundError(f"Android SDK tool '{tool_stem}' not found under {bt_dir}")


def _ensure_signed_apk(
    sdk_dir: Path,
    source_apk: Path,
    dest_apk: Path,
    env: Mapping[str, str],
) -> None:
    """Align and sign an Android APK with debug keystore so it is directly sideloadable."""
    apksigner = _find_build_tool(sdk_dir, "apksigner")
    zipalign = _find_build_tool(sdk_dir, "zipalign")

    if "unsigned" not in source_apk.name.lower():
        verify_proc = subprocess.run(
            [str(apksigner), "verify", str(source_apk)],
            cwd=ROOT,
            env=dict(env),
            capture_output=True,
            check=False,
        )
        if verify_proc.returncode == 0:
            shutil.copy2(source_apk, dest_apk)
            return

    keystore = Path.home() / ".android" / "debug.keystore"
    if not keystore.is_file():
        keystore.parent.mkdir(parents=True, exist_ok=True)
        run_step(
            [
                "keytool",
                "-genkey",
                "-v",
                "-keystore",
                str(keystore),
                "-alias",
                "androiddebugkey",
                "-keyalg",
                "RSA",
                "-keysize",
                "2048",
                "-validity",
                "10000",
                "-storepass",
                "android",
                "-keypass",
                "android",
                "-dname",
                "CN=Android Debug,O=Android,C=US",
            ],
            cwd=ROOT,
            env=env,
        )

    aligned_tmp = dest_apk.with_suffix(".aligned.apk")
    run_step([str(zipalign), "-p", "-f", "4", str(source_apk), str(aligned_tmp)], cwd=ROOT, env=env)
    try:
        run_step(
            [
                str(apksigner),
                "sign",
                "--ks",
                str(keystore),
                "--ks-key-alias",
                "androiddebugkey",
                "--ks-pass",
                "pass:android",
                "--key-pass",
                "pass:android",
                "--out",
                str(dest_apk),
                str(aligned_tmp),
            ],
            cwd=ROOT,
            env=env,
        )
    finally:
        aligned_tmp.unlink(missing_ok=True)

    run_step([str(apksigner), "verify", str(dest_apk)], cwd=ROOT, env=env)


def build_android(base_env: Mapping[str, str]) -> Path:
    """Compile, zipalign, and sign Android ARM64 APK directly from panadiga-desktop workspace."""
    print_banner("BUILD ANDROID: Tauri 2 Mobile APK (AArch64)")
    sdk_dir, ndk_dir = _detect_android_sdk(base_env)
    env = dict(base_env)
    env["ANDROID_HOME"] = str(sdk_dir)
    if ndk_dir is not None:
        env["NDK_HOME"] = str(ndk_dir)
        env["ANDROID_NDK_ROOT"] = str(ndk_dir)

    desktop_dir = ROOT / "apps" / "desktop"
    gen_android = desktop_dir / "src-tauri" / "gen" / "android"
    gen_android.mkdir(parents=True, exist_ok=True)
    props_lines = [f"sdk.dir={sdk_dir.as_posix()}"]
    if ndk_dir is not None:
        props_lines.append(f"ndk.dir={ndk_dir.as_posix()}")
    (gen_android / "local.properties").write_text("\n".join(props_lines) + "\n", encoding="utf-8")

    run_step(["rustup", "target", "add", "aarch64-linux-android"], cwd=ROOT, env=env)
    run_step(["npm", "--prefix", "apps/desktop", "run", "build"], cwd=ROOT, env=env)
    if not (gen_android / "app").is_dir():
        run_step(["npx", "@tauri-apps/cli", "android", "init", "--ci", "--skip-targets-install"], cwd=desktop_dir, env=env)
    run_step(["npx", "@tauri-apps/cli", "android", "build", "--apk", "--target", "aarch64"], cwd=desktop_dir, env=env)

    apk_root = gen_android / "app" / "build" / "outputs" / "apk"
    apk_candidates = sorted(apk_root.rglob("*.apk"))
    if not apk_candidates:
        raise FileNotFoundError("Android build completed but no .apk artifact was produced.")
    preferred = [p for p in apk_candidates if "release" in p.name.lower()]
    source_apk = preferred[0] if preferred else apk_candidates[0]

    dist_dir = ROOT / "dist"
    releases_dir = dist_dir / "releases"
    releases_dir.mkdir(parents=True, exist_ok=True)
    dist_apk = dist_dir / "panadiga-android-arm64.apk"
    rel_apk = releases_dir / "panadiga-android-arm64.apk"

    _ensure_signed_apk(sdk_dir, source_apk, dist_apk, env)
    shutil.copy2(dist_apk, rel_apk)
    update_sha256_manifest(releases_dir / "SHA256SUMS.txt", [rel_apk])
    print_status("OK", f"Signed Android APK packaged at: {rel_apk}", "32")
    return rel_apk


def _prepare_pico_env(base_env: Mapping[str, str]) -> dict[str, str]:
    """Augment build environment with MSVC host env, Pico SDK, and ARM embedded toolchain."""
    pico_env = load_msvc_env(base_env)
    pico_env.setdefault("CMAKE_POLICY_VERSION_MINIMUM", "3.5")

    if not pico_env.get("PICO_SDK_PATH"):
        sdk_candidates = [
            Path(r"C:\Program Files\Raspberry Pi\Pico SDK v2.0.0\pico-sdk"),
            Path(r"C:\Program Files\Raspberry Pi\Pico SDK v1.5.1\pico-sdk"),
            Path(r"C:\pico\pico-sdk"),
            Path(r"C:\pico-sdk"),
            Path.home() / "pico-sdk",
            Path.home() / ".pico-sdk" / "sdk" / "2.0.0",
            Path.home() / ".pico-sdk" / "sdk" / "1.5.1",
            ROOT / "pico-sdk",
        ]
        for cand in sdk_candidates:
            if cand.is_dir():
                pico_env["PICO_SDK_PATH"] = str(cand.resolve())
                break

    if sys.platform == "win32" and not shutil.which("arm-none-eabi-gcc", path=pico_env.get("PATH")):
        for root_dir in (
            Path(r"C:\Program Files (x86)\Arm GNU Toolchain arm-none-eabi"),
            Path(r"C:\Program Files\Arm GNU Toolchain arm-none-eabi"),
        ):
            if root_dir.is_dir():
                bins = sorted(root_dir.glob("*/bin"), reverse=True)
                for b in bins:
                    if (b / "arm-none-eabi-gcc.exe").is_file():
                        pico_env["PATH"] = f"{b}{os.pathsep}{pico_env.get('PATH', '')}"
                        break

    if sys.platform == "win32":
        extra_dirs: list[Path] = []
        vs_cmake_base = Path(
            r"C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\Common7\IDE\CommonExtensions\Microsoft\CMake"
        )
        if not shutil.which("cmake", path=pico_env.get("PATH")):
            extra_dirs.append(vs_cmake_base / "CMake" / "bin")
        if not shutil.which("ninja", path=pico_env.get("PATH")):
            extra_dirs.append(vs_cmake_base / "Ninja")
        for d in extra_dirs:
            if d.is_dir():
                pico_env["PATH"] = f"{d}{os.pathsep}{pico_env.get('PATH', '')}"
    return pico_env


def build_pico(env: Mapping[str, str]) -> None:
    """Compile RP2040 + RP2350 + Unified UF2 Pico firmware out-of-tree and sync firmware_bins/."""
    print_banner("BUILD PICO: P7501 Async UART RP2040 + RP2350 + Unified UF2")
    fw_dir = ROOT / "firmware" / "pico-p7501"
    build_root = ROOT / "target" / "pico-build"
    dest_dir = ROOT / "crates" / "panadiga-core" / "src" / "pico" / "firmware_bins"
    dest_dir.mkdir(parents=True, exist_ok=True)
    pico_env = _prepare_pico_env(env)

    for board, out_name in (("pico", "panasonic_p7501_uart_rp2040.uf2"), ("pico2", "panasonic_p7501_uart_rp2350.uf2")):
        bdir = build_root / board
        run_step(
            [
                "cmake",
                "-S",
                str(fw_dir),
                "-B",
                str(bdir),
                "-G",
                "Ninja",
                "-DCMAKE_BUILD_TYPE=Release",
                f"-DPICO_BOARD={board}",
                "-DCMAKE_POLICY_VERSION_MINIMUM=3.5",
            ],
            cwd=ROOT,
            env=pico_env,
        )
        run_step(["cmake", "--build", str(bdir), "--config", "Release"], cwd=ROOT, env=pico_env)
        built_uf2 = bdir / "panasonic_p7501_uart.uf2"
        if not built_uf2.is_file():
            raise FileNotFoundError(f"Expected compiled UF2 missing: {built_uf2}")
        shutil.copy2(built_uf2, dest_dir / out_name)

    run_step(
        [
            "cargo",
            "run",
            "-p",
            "panadiga-cli",
            "--",
            "pico",
            "merge-uf2",
            "--rp2040",
            str(dest_dir / "panasonic_p7501_uart_rp2040.uf2"),
            "--rp2350",
            str(dest_dir / "panasonic_p7501_uart_rp2350.uf2"),
            "--out",
            str(dest_dir / "panasonic_p7501_uart_unified.uf2"),
        ],
        cwd=ROOT,
        env=pico_env,
    )
    print_status("OK", f"Synced UF2 binaries into {dest_dir}", "32")


def launch_gui(env: Mapping[str, str], rebuild: bool = False) -> None:
    """Launch Desktop GUI detached, building desktop release first when missing."""
    binary = None if rebuild else locate_panadiga_desktop_binary()
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
    proc = subprocess.run([cargo_bin, "run", "-p", "panadiga-cli", "--", *cli_args], cwd=str(ROOT), env=dict(env), check=False)
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
