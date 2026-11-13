"""Unified cross-platform build DAG, CI pipeline, step cache, packaging, and TUI runner."""

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
from concurrent.futures import FIRST_EXCEPTION, Future, ThreadPoolExecutor, wait
import platform
import shutil
import subprocess
import tarfile
import threading
import time
import zipfile
from pathlib import Path
from typing import Any, Callable, Final, Mapping, Sequence

from scripts.panadiga_cache import (
    clear_caches,
    copy_if_changed,
    discover_pico_sdk,
    discover_workspace_inventory,
    frontend_digest_and_outputs,
    ghidra_digest_and_outputs,
    is_step_fresh,
    optimize_build_env,
    pico_digest_and_outputs,
    record_step_cache,
)
from scripts.panadiga_ops import (
    ProcessTracker,
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
    use_process_tracker,
)
from scripts.tui import (
    PipelineSelection,
    default_all_selection,
    launch_tui,
    render_inventory_and_cache_report,
)
from scripts.validate import run_validation

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


_FLUSH_LOCK = threading.Lock()


def _flush_output(text: str) -> None:
    if text:
        with _FLUSH_LOCK:
            print(text, end="" if text.endswith("\n") else "\n", flush=True)


def _run_concurrent_tasks(
    tasks: Sequence[tuple[str, Callable[[], Any]]],
) -> dict[str, Any]:
    """Run independent callables concurrently with fail-fast child cancellation."""
    if not tasks:
        return {}
    if len(tasks) == 1:
        name, fn = tasks[0]
        return {name: fn()}

    tracker = ProcessTracker()

    def _wrapped(fn: Callable[[], Any]) -> Any:
        with use_process_tracker(tracker):
            return fn()

    results: dict[str, Any] = {}
    with ThreadPoolExecutor(max_workers=len(tasks)) as pool:
        fmap: dict[Future[Any], str] = {
            pool.submit(_wrapped, fn): name for name, fn in tasks
        }
        pending: set[Future[Any]] = set(fmap)
        try:
            while pending:
                done, pending = wait(pending, return_when=FIRST_EXCEPTION)
                for fut in done:
                    name = fmap[fut]
                    try:
                        results[name] = fut.result()
                    except BaseException:
                        for p in pending:
                            p.cancel()
                        tracker.terminate_all()
                        raise
        except KeyboardInterrupt:
            for p in pending:
                p.cancel()
            tracker.terminate_all()
            raise
    return results


def build_frontend(env: Mapping[str, str], *, use_cache: bool = True, capture_output: bool = False) -> bool:
    """Build Desktop React/Vite bundle; returns True when satisfied from hash cache."""
    digest, outputs = frontend_digest_and_outputs(ROOT)
    if use_cache and is_step_fresh(ROOT, "frontend", digest, outputs):
        if not capture_output:
            print_status("CACHE", "Frontend Vite UI cache hit (apps/desktop/dist unchanged).", "36")
        return True

    out = run_step(
        ["npm", "--prefix", "apps/desktop", "run", "build"],
        cwd=ROOT,
        env=env,
        capture_output=True,
    )
    _flush_output(out)
    record_step_cache(ROOT, "frontend", digest, outputs)
    return False


def export_ghidra_ssot(
    cli_bin: Path,
    env: Mapping[str, str],
    *,
    use_cache: bool = True,
) -> bool:
    """Export Ghidra Jython SSOT scripts; returns True when satisfied from cache."""
    digest, outputs = ghidra_digest_and_outputs(ROOT)
    if use_cache and is_step_fresh(ROOT, "ghidra", digest, outputs):
        return True

    out = run_step(
        [str(cli_bin), "ghidra", "export-all", "--dir", "tools/ghidra"],
        cwd=ROOT,
        env=env,
        capture_output=True,
    )
    _flush_output(out)
    record_step_cache(ROOT, "ghidra", digest, outputs)
    return False


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


def build_desktop(
    env: Mapping[str, str],
    *,
    skip_frontend: bool = False,
    use_cache: bool = True,
    capture_output: bool = False,
) -> Path:
    """Build CLI & Desktop GUI together in one parallel Cargo DAG, export Ghidra SSOT, and stage dist/."""
    print_banner("BUILD DESKTOP: Unified Parallel Cargo (CLI + Desktop GUI Release)")
    if not skip_frontend:
        build_frontend(env, use_cache=use_cache, capture_output=capture_output)

    cargo_out = run_step(
        ["cargo", "build", "--release", "-p", "panadiga-cli", "-p", "panadiga-desktop"],
        cwd=ROOT,
        env=env,
        capture_output=capture_output,
    )
    _flush_output(cargo_out)

    release_cli = ROOT / "target" / "release" / panadiga_binary_name()
    if export_ghidra_ssot(release_cli, env, use_cache=use_cache):
        print_status("CACHE", "Ghidra SSOT scripts unchanged (cache hit).", "36")

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


def detect_android_sdk(env: Mapping[str, str]) -> tuple[Path, Path | None]:
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


def _prepare_android_env(base_env: Mapping[str, str]) -> tuple[dict[str, str], Path, Path, Path]:
    sdk_dir, ndk_dir = detect_android_sdk(base_env)
    env = dict(base_env)
    env["ANDROID_HOME"] = str(sdk_dir)
    env.setdefault("CARGO_TARGET_DIR", str(ROOT / "target" / "android-cargo"))
    if ndk_dir is not None:
        env["NDK_HOME"] = str(ndk_dir)
        env["ANDROID_NDK_ROOT"] = str(ndk_dir)

    desktop_dir = ROOT / "apps" / "desktop"
    gen_android = desktop_dir / "src-tauri" / "gen" / "android"
    gen_android.mkdir(parents=True, exist_ok=True)
    props_lines = [f"sdk.dir={sdk_dir.as_posix()}"]
    if ndk_dir is not None:
        props_lines.append(f"ndk.dir={ndk_dir.as_posix()}")
    props_file = gen_android / "local.properties"
    desired = "\n".join(props_lines) + "\n"
    if not props_file.is_file() or props_file.read_text(encoding="utf-8", errors="ignore") != desired:
        props_file.write_text(desired, encoding="utf-8")
    return env, sdk_dir, desktop_dir, gen_android


def _install_android_rust_target(env: Mapping[str, str]) -> None:
    out = run_step(
        ["rustup", "target", "add", "aarch64-linux-android"],
        cwd=ROOT,
        env=env,
        capture_output=True,
    )
    _flush_output(out)


def build_android(
    base_env: Mapping[str, str],
    *,
    skip_frontend: bool = False,
    skip_prep: bool = False,
    use_cache: bool = True,
    capture_output: bool = False,
) -> Path:
    """Compile, zipalign, and sign Android ARM64 APK directly from panadiga-desktop workspace."""
    print_banner("BUILD ANDROID: Tauri 2 Mobile APK (AArch64)")
    env, sdk_dir, desktop_dir, gen_android = _prepare_android_env(base_env)

    prep_tasks: list[tuple[str, Callable[[], Any]]] = []
    if not skip_prep:
        prep_tasks.append(("rustup", lambda: _install_android_rust_target(env)))
    if not skip_frontend:
        prep_tasks.append(("frontend", lambda: build_frontend(env, use_cache=use_cache, capture_output=capture_output)))
    _run_concurrent_tasks(prep_tasks)

    if not (gen_android / "app").is_dir():
        init_out = run_step(
            ["npx", "@tauri-apps/cli", "android", "init", "--ci", "--skip-targets-install"],
            cwd=desktop_dir,
            env=env,
            capture_output=capture_output,
        )
        _flush_output(init_out)

    android_cmd = ["npx", "@tauri-apps/cli", "android", "build", "--apk", "--target", "aarch64"]
    if skip_frontend:
        skip_cfg = ROOT / "target" / ".panadiga-cache" / "tauri.skip_frontend.json"
        skip_cfg.parent.mkdir(parents=True, exist_ok=True)
        desired_cfg = '{"build":{"beforeBuildCommand":""}}\n'
        if not skip_cfg.is_file() or skip_cfg.read_text(encoding="utf-8", errors="ignore") != desired_cfg:
            skip_cfg.write_text(desired_cfg, encoding="utf-8")
        android_cmd.extend(["--config", str(skip_cfg)])

    build_out = run_step(android_cmd, cwd=desktop_dir, env=env, capture_output=capture_output)
    _flush_output(build_out)

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
    pico_env, _ = optimize_build_env(load_msvc_env(base_env), root=ROOT, use_cache=True)
    pico_env.setdefault("CMAKE_POLICY_VERSION_MINIMUM", "3.5")
    if not pico_env.get("PICO_SDK_PATH"):
        sdk = discover_pico_sdk(ROOT, pico_env)
        if sdk is not None:
            pico_env["PICO_SDK_PATH"] = str(sdk)
    return pico_env


def _build_single_pico_board(
    board: str,
    fw_dir: Path,
    build_root: Path,
    pico_env: Mapping[str, str],
) -> tuple[str, Path, str]:
    """Configure and compile one Pico board target cleanly out-of-tree."""
    bdir = build_root / board
    cmake_cmd = [
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
    ]
    ccache_bin = pico_env.get("PANADIGA_CCACHE")
    if ccache_bin:
        cmake_cmd.extend(
            [
                f"-DCMAKE_C_COMPILER_LAUNCHER={ccache_bin}",
                f"-DCMAKE_CXX_COMPILER_LAUNCHER={ccache_bin}",
            ]
        )

    log_a = run_step(cmake_cmd, cwd=ROOT, env=pico_env, capture_output=True)
    log_b = run_step(
        ["cmake", "--build", str(bdir), "--config", "Release", "--parallel"],
        cwd=ROOT,
        env=pico_env,
        capture_output=True,
    )
    built_uf2 = bdir / "panasonic_p7501_uart.uf2"
    if not built_uf2.is_file():
        raise FileNotFoundError(f"Expected compiled UF2 missing: {built_uf2}")
    return board, built_uf2, f"{log_a}{log_b}"


def _compile_pico_boards_parallel(pico_env: Mapping[str, str]) -> tuple[Path, Path]:
    """Build RP2040 and RP2350 CMake/Ninja targets simultaneously."""
    fw_dir = ROOT / "firmware" / "pico-p7501"
    build_root = ROOT / "target" / "pico-build"
    results = _run_concurrent_tasks(
        [
            ("pico", lambda: _build_single_pico_board("pico", fw_dir, build_root, pico_env)),
            ("pico2", lambda: _build_single_pico_board("pico2", fw_dir, build_root, pico_env)),
        ]
    )
    rp2040_uf2 = Path(results["pico"][1])
    rp2350_uf2 = Path(results["pico2"][1])
    for key in ("pico", "pico2"):
        logs = str(results[key][2])
        if logs:
            _flush_output(f"[{key.upper()}]\n{logs}")
    return rp2040_uf2, rp2350_uf2


def _merge_and_sync_pico_uf2(
    rp2040_uf2: Path,
    rp2350_uf2: Path,
    pico_env: Mapping[str, str],
    digest: str,
    outputs: Sequence[Path],
    *,
    prefer_release_merge: bool = False,
) -> None:
    """Merge dual UF2s and sync firmware_bins/ only where SHA-256 changed."""
    build_root = ROOT / "target" / "pico-build"
    dest_dir = ROOT / "crates" / "panadiga-core" / "src" / "pico" / "firmware_bins"
    dest_dir.mkdir(parents=True, exist_ok=True)
    staged_unified = build_root / "panasonic_p7501_uart_unified.uf2"

    existing_cli = locate_panadiga_binary()
    if existing_cli is not None:
        merge_cmd: list[str] = [str(existing_cli)]
    elif prefer_release_merge:
        merge_cmd = ["cargo", "run", "--release", "-p", "panadiga-cli", "--"]
    else:
        merge_cmd = ["cargo", "run", "-p", "panadiga-cli", "--"]

    run_step(
        [
            *merge_cmd,
            "pico",
            "merge-uf2",
            "--rp2040",
            str(rp2040_uf2),
            "--rp2350",
            str(rp2350_uf2),
            "--out",
            str(staged_unified),
        ],
        cwd=ROOT,
        env=pico_env,
    )
    changed_rp2040 = copy_if_changed(rp2040_uf2, dest_dir / "panasonic_p7501_uart_rp2040.uf2")
    changed_rp2350 = copy_if_changed(rp2350_uf2, dest_dir / "panasonic_p7501_uart_rp2350.uf2")
    changed_unified = copy_if_changed(staged_unified, dest_dir / "panasonic_p7501_uart_unified.uf2")
    record_step_cache(ROOT, "pico", digest, outputs)
    if changed_rp2040 or changed_rp2350 or changed_unified:
        print_status("OK", f"Synced updated UF2 binaries into {dest_dir}", "32")
    else:
        print_status("CACHE", "Rebuilt UF2 bytes identical to firmware_bins/ (zero-touch skip).", "36")


def build_pico(
    env: Mapping[str, str],
    *,
    prefer_release_merge: bool = False,
    use_cache: bool = True,
) -> bool:
    """Compile RP2040 + RP2350 Pico firmwares in parallel, merge UF2, and sync firmware_bins/."""
    print_banner("BUILD PICO: Parallel RP2040 + RP2350 + Unified UF2")
    pico_env = _prepare_pico_env(env)
    digest, outputs = pico_digest_and_outputs(ROOT, pico_env)
    if use_cache and is_step_fresh(ROOT, "pico", digest, outputs):
        print_status("CACHE", "Pico firmware UF2 cache hit (firmware_bins unchanged).", "36")
        return True

    rp2040_uf2, rp2350_uf2 = _compile_pico_boards_parallel(pico_env)
    _merge_and_sync_pico_uf2(
        rp2040_uf2,
        rp2350_uf2,
        pico_env,
        digest,
        outputs,
        prefer_release_merge=prefer_release_merge,
    )
    return False


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


def run_build_pipeline(
    selection: PipelineSelection,
    base_env: Mapping[str, str],
    *,
    strict_toolchains: bool = False,
) -> list[tuple[str, str, float]]:
    """Execute dependency-aware build & CI DAG: parallel when independent, serial at locks."""
    total_start = time.monotonic()
    env, accel = optimize_build_env(base_env, root=ROOT, use_cache=selection.use_cache)
    accel_desc = ", ".join(f"{k}={v}" for k, v in accel.items())
    print_banner(f"PANADIGA PIPELINE DAG | {accel_desc}")

    summary: list[tuple[str, str, float]] = []
    need_frontend = selection.frontend or selection.desktop or selection.android
    pico_env = _prepare_pico_env(env) if selection.pico else env
    pico_digest = ""
    pico_outputs: tuple[Path, ...] = ()
    pico_cached = False
    run_pico = False
    if selection.pico:
        has_arm = bool(shutil.which("arm-none-eabi-gcc", path=pico_env.get("PATH"))) and bool(
            pico_env.get("PICO_SDK_PATH")
        )
        if not has_arm:
            if strict_toolchains:
                raise FileNotFoundError("arm-none-eabi-gcc or PICO_SDK_PATH missing for --pico build.")
            print_status("INFO", "Pico toolchain/SDK absent; skipping RP2040/RP2350 firmware rebuild.", "33")
            summary.append(("Pico Firmware", "SKIP", 0.0))
        else:
            pico_digest, pico_outputs = pico_digest_and_outputs(ROOT, pico_env)
            if selection.use_cache and is_step_fresh(ROOT, "pico", pico_digest, pico_outputs):
                print_status("CACHE", "Pico firmware UF2 cache hit (firmware_bins unchanged).", "36")
                summary.append(("Pico Firmware", "CACHE HIT", 0.0))
                pico_cached = True
            else:
                run_pico = True

    run_android = False
    if selection.android:
        try:
            _prepare_android_env(env)
            run_android = True
        except FileNotFoundError as exc:
            if strict_toolchains:
                raise
            print_status("INFO", f"{exc} Skipping Android APK.", "33")
            summary.append(("Android APK", "SKIP", 0.0))

    # Wave 1: Parallel independent build & preparation tasks.
    wave1_tasks: list[tuple[str, Callable[[], Any]]] = []
    wave1_t0 = time.monotonic()
    if need_frontend:
        wave1_tasks.append(
            (
                "frontend",
                lambda: build_frontend(env, use_cache=selection.use_cache, capture_output=True),
            )
        )
    if run_pico and not pico_cached:
        wave1_tasks.append(("pico_boards", lambda: _compile_pico_boards_parallel(pico_env)))
    if run_android:
        wave1_tasks.append(("android_prep", lambda: _install_android_rust_target(env)))

    if wave1_tasks:
        w1 = _run_concurrent_tasks(wave1_tasks)
        w1_dt = time.monotonic() - wave1_t0
        if "frontend" in w1:
            fe_hit = bool(w1["frontend"])
            summary.append(("Frontend Vite UI", "CACHE HIT" if fe_hit else "BUILT", w1_dt))
        if "pico_boards" in w1:
            t_merge = time.monotonic()
            rp2040_uf2, rp2350_uf2 = w1["pico_boards"]
            _merge_and_sync_pico_uf2(
                rp2040_uf2,
                rp2350_uf2,
                pico_env,
                pico_digest,
                pico_outputs,
                prefer_release_merge=True,
            )
            summary.append(("Pico UF2 Dual+Merge", "BUILT", (time.monotonic() - wave1_t0)))
            _ = t_merge

    # Wave 2: Parallel 9-gate CI validation (after UF2/frontend sync, before heavy release/APK).
    if selection.ci:
        ci_dt = run_validation(env)
        gh_digest, gh_outs = ghidra_digest_and_outputs(ROOT)
        record_step_cache(ROOT, "ghidra", gh_digest, gh_outs)
        summary.append(("CI 9 Quality Gates", "PASS", ci_dt))

    # Wave 3: Compile Desktop Release (CLI+GUI unified Cargo DAG) and Android APK in parallel!
    wave3_tasks: list[tuple[str, Callable[[], Any]]] = []
    both_native = selection.desktop and run_android
    wave3_t0 = time.monotonic()

    def _run_desktop_lane() -> tuple[str, Path, float]:
        dt0 = time.monotonic()
        out = build_desktop(
            env,
            skip_frontend=True,
            use_cache=selection.use_cache,
            capture_output=both_native,
        )
        return ("Desktop Release + dist/", out, time.monotonic() - dt0)

    def _run_android_lane() -> tuple[str, Path, float]:
        dt0 = time.monotonic()
        out = build_android(
            env,
            skip_frontend=True,
            skip_prep=True,
            use_cache=selection.use_cache,
            capture_output=both_native,
        )
        return ("Android ARM64 APK", out, time.monotonic() - dt0)

    if selection.desktop:
        wave3_tasks.append(("desktop", _run_desktop_lane))
    if run_android:
        wave3_tasks.append(("android", _run_android_lane))

    if wave3_tasks:
        w3 = _run_concurrent_tasks(wave3_tasks)
        for key in ("desktop", "android"):
            if key in w3:
                lbl, _, lane_dt = w3[key]
                summary.append((lbl, "BUILT", lane_dt))
        _ = wave3_t0

    total_dt = time.monotonic() - total_start
    print_banner(f"PANADIGA PIPELINE COMPLETE (Elapsed: {total_dt:.1f}s)", ansi_color="32")
    for label, status, dt in summary:
        print_status(status, f"{label:<26} ({dt:.2f}s)", "32" if status != "SKIP" else "33")

    if selection.run_after and selection.desktop:
        launch_gui(env, rebuild=False)
    return summary


def parse_args(argv: Sequence[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Panadiga unified build, CI, cache, and TUI orchestrator.")
    parser.add_argument("--tui", action="store_true", help="Launch interactive TUI menu to choose targets, CI, and cache.")
    parser.add_argument("--ci", action="store_true", help="Execute 9 parallel Quality Gates in the pipeline.")
    parser.add_argument("--frontend", action="store_true", help="Build only the React/Vite desktop UI bundle.")
    parser.add_argument("--desktop", action="store_true", help="Build Desktop release binary, staging dist/ and release archive.")
    parser.add_argument("--android", action="store_true", help="Build Android ARM64 APK via Tauri Mobile.")
    parser.add_argument("--pico", action="store_true", help="Build RP2040, RP2350, and unified Pico UF2 firmwares.")
    parser.add_argument("--all", action="store_true", help="Build Pico (if toolchain exists), Desktop, and Android (if SDK/NDK exists).")
    parser.add_argument("--no-cache", action="store_true", help="Disable Panadiga hash cache and force rebuild of selected steps.")
    parser.add_argument("--cache-status", action="store_true", help="Print recursive workspace discovery, toolchains, and cache sizes.")
    parser.add_argument("--clean-cache", action="store_true", help="Clear Panadiga step/pico/vite caches.")
    parser.add_argument("--clean-all-caches", action="store_true", help="Clear all caches including Cargo target/debug and target/release.")
    parser.add_argument("--strict-toolchains", action="store_true", help="Fail instead of skipping if Pico/Android toolchains are absent in --all.")
    parser.add_argument("--run", action="store_true", help="Launch Desktop GUI (auto-builds if missing).")
    parser.add_argument("--rebuild-run", action="store_true", help="Force rebuild Desktop release and launch GUI.")
    parser.add_argument("--cli", nargs=argparse.REMAINDER, help="Forward remaining arguments to the native Panadiga CLI.")
    return parser.parse_args(argv)


def main(argv: Sequence[str] | None = None) -> int:
    """Dispatch build/CI/cache/TUI/run actions according to CLI flags."""
    args = parse_args(argv)
    base_env = build_env()
    use_cache = not args.no_cache
    env, _ = optimize_build_env(base_env, root=ROOT, use_cache=use_cache)

    try:
        if args.clean_all_caches or args.clean_cache:
            removed = clear_caches(ROOT, include_cargo=bool(args.clean_all_caches))
            print_status("OK", f"Cleared {len(removed)} cache directories.", "32")
            if not any((args.tui, args.ci, args.frontend, args.desktop, args.pico, args.android, args.all)):
                return 0

        if args.cache_status:
            inv = discover_workspace_inventory(ROOT, env)
            print(render_inventory_and_cache_report(ROOT, inv, env))
            return 0

        if args.tui:
            inv = discover_workspace_inventory(ROOT, env)
            chosen = launch_tui(inv, env)
            if chosen is None:
                return 0
            run_build_pipeline(chosen, env, strict_toolchains=args.strict_toolchains)
            return 0

        if args.cli is not None:
            return run_cli(args.cli, env)
        if args.rebuild_run:
            launch_gui(env, rebuild=True)
            return 0
        if args.run and not any((args.ci, args.desktop, args.pico, args.android, args.all, args.frontend)):
            launch_gui(env, rebuild=False)
            return 0

        if args.all:
            sel = PipelineSelection(
                ci=bool(args.ci),
                frontend=True,
                pico=True,
                desktop=True,
                android=True,
                use_cache=use_cache,
                run_after=bool(args.run),
            )
            run_build_pipeline(sel, env, strict_toolchains=args.strict_toolchains)
            return 0

        explicit = any((args.ci, args.frontend, args.desktop, args.pico, args.android))
        sel = PipelineSelection(
            ci=bool(args.ci),
            frontend=bool(args.frontend),
            pico=bool(args.pico),
            desktop=bool(args.desktop) or not explicit,
            android=bool(args.android),
            use_cache=use_cache,
            run_after=bool(args.run),
        )
        run_build_pipeline(sel, env, strict_toolchains=True)
        return 0
    except (subprocess.CalledProcessError, FileNotFoundError, RuntimeError) as exc:
        if isinstance(exc, subprocess.CalledProcessError):
            raw_diag: object = exc.output if exc.output else (exc.stdout if exc.stdout else exc.stderr)
            if isinstance(raw_diag, bytes):
                _flush_output(raw_diag.decode("utf-8", errors="replace").strip())
            elif isinstance(raw_diag, str) and raw_diag.strip():
                _flush_output(raw_diag.strip())
        print_status("FAIL", f"Build pipeline failed: {exc}", "31")
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
