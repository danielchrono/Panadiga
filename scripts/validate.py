"""Panadiga universal 9 Quality Gates validation suite (Windows, Linux, macOS)."""

from __future__ import annotations

import os
import sys

_SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
sys.path = [os.path.dirname(_SCRIPT_DIR), *(p for p in sys.path if os.path.abspath(p or ".") != _SCRIPT_DIR)]

import argparse
from concurrent.futures import FIRST_EXCEPTION, Future, ThreadPoolExecutor, wait
from pathlib import Path
import shutil
import subprocess
import tempfile
import time
from typing import Callable, Final, Mapping

from scripts.panadiga_ops import (
    ProcessTracker,
    ROOT,
    build_env,
    load_msvc_env,
    print_banner,
    print_status,
    run_step,
    use_process_tracker,
)

DESKTOP_DIR: Final[Path] = ROOT / "apps" / "desktop"
C11_HOST_SOURCES: Final[tuple[str, ...]] = (
    "firmware/pico-p7501/common/p7501_hex.c",
    "firmware/pico-p7501/hal/p7501_led_logic.c",
    "firmware/pico-p7501/gdb/gdb_rsp_codec.c",
    "firmware/pico-p7501/gdb/gdb_rsp_memory.c",
    "firmware/pico-p7501/gdb/gdb_rsp_cpu.c",
    "firmware/pico-p7501/uart/p7501_uart_codec.c",
    "firmware/pico-p7501/uart/p7501_uart_args.c",
    "firmware/pico-p7501/tests/test_uart_codec.c",
    "firmware/pico-p7501/tests/test_gdb_rsp_host.c",
)
PYTHON_COMPILE_TARGETS: Final[tuple[str, ...]] = (
    "tools/ghidra/ghidra_am34_memory_map.py",
    "tools/ghidra/ghidra_import_symbols_panadiga.py",
    "tools/ghidra/ghidra_connect_debugger.py",
    "scripts/panadiga_ops.py",
    "scripts/panadiga_cache.py",
    "scripts/tui.py",
    "scripts/build.py",
    "scripts/validate.py",
    "scripts/dev/generate_schematics_and_cad.py",
    "scripts/re/__init__.py",
    "scripts/re/mn10300.py",
    "scripts/re/pana_frm.py",
    "scripts/bench/__init__.py",
    "scripts/bench/protocol.py",
    "scripts/bench/decoders.py",
    "scripts/bench/runner.py",
)


def _compile_c11_host_test(
    exe_path: Path,
    work_dir: Path,
    env: Mapping[str, str],
    *,
    capture_output: bool = False,
) -> None:
    """Compile C11 Pico RSP host test suite into an isolated temp directory with strict warnings."""
    for compiler in ("gcc", "clang"):
        if shutil.which(compiler, path=env.get("PATH")):
            cmd = [
                compiler,
                "-std=c11",
                "-Wall",
                "-Wextra",
                "-Werror",
                "-DPICO_HOST_TEST=1",
                "-I",
                "firmware/pico-p7501",
                *C11_HOST_SOURCES,
                "-o",
                str(exe_path),
            ]
            run_step(cmd, cwd=ROOT, env=env, capture_output=capture_output)
            return

    if sys.platform == "win32":
        msvc_env = load_msvc_env(env)
        if shutil.which("cl", path=msvc_env.get("PATH")):
            obj_dir = work_dir.as_posix().rstrip("/") + "/"
            run_step(
                [
                    "cl",
                    "/nologo",
                    "/MP",
                    "/std:c11",
                    "/W4",
                    "/WX",
                    "/DPICO_HOST_TEST=1",
                    "/D_CRT_SECURE_NO_WARNINGS",
                    "/Ifirmware/pico-p7501",
                    *C11_HOST_SOURCES,
                    f"/Fo:{obj_dir}",
                    f"/Fe:{exe_path}",
                ],
                cwd=ROOT,
                env=msvc_env,
                capture_output=capture_output,
            )
            return

    raise RuntimeError("No C11 compiler (gcc, clang, or MSVC cl) found for Pico firmware host tests.")


def _gate_rust(env: Mapping[str, str], *, capture_output: bool = False) -> None:
    if not capture_output:
        print_banner("1/9. Validating Rust Workspace (Cargo Fmt, Check & Tests)")
    run_step(["cargo", "fmt", "--all", "--", "--check"], cwd=ROOT, env=env, capture_output=capture_output)
    run_step(["cargo", "check", "--workspace", "--all-targets"], cwd=ROOT, env=env, capture_output=capture_output)
    run_step(["cargo", "test", "--workspace"], cwd=ROOT, env=env, capture_output=capture_output)
    if not capture_output:
        print_status("PASS", "Rust workspace formatted, compiled, and all tests passed!", "32")


def _gate_typescript(env: Mapping[str, str], *, capture_output: bool = False) -> None:
    if not capture_output:
        print_banner("2/9. Validating TypeScript Strict Types (tsc --noEmit)")
    run_step(["npm", "run", "typecheck"], cwd=DESKTOP_DIR, env=env, capture_output=capture_output)
    if not capture_output:
        print_status("PASS", "TypeScript strict type check succeeded with 0 errors!", "32")


def _gate_vitest(env: Mapping[str, str], *, capture_output: bool = False) -> None:
    if not capture_output:
        print_banner("3/9. Executing Desktop Frontend Unit Tests (Vitest)")
    run_step(["npm", "test"], cwd=DESKTOP_DIR, env=env, capture_output=capture_output)
    if not capture_output:
        print_status("PASS", "All Vitest frontend and ViewModel unit tests passed!", "32")


def _gate_jscpd(env: Mapping[str, str], *, capture_output: bool = False) -> None:
    if not capture_output:
        print_banner("4/9. Running Copy/Paste Detector (jscpd threshold 0% / 0 clones)")
    run_step(["npx", "--prefix", "apps/desktop", "jscpd", "--config", ".jscpd.json"], cwd=ROOT, env=env, capture_output=capture_output)
    if not capture_output:
        print_status("PASS", "Code duplication verified at 0% (0 clones found)!", "32")


def _gate_uxlint(env: Mapping[str, str], *, capture_output: bool = False) -> None:
    if not capture_output:
        print_banner("5/9. Running UI/UX State Linter (uxlint)")
    run_step(["npm", "run", "uxlint"], cwd=DESKTOP_DIR, env=env, capture_output=capture_output)
    if not capture_output:
        print_status("PASS", "UX state completeness audit passed!", "32")


def _gate_playwright(env: Mapping[str, str], *, capture_output: bool = False) -> None:
    if not capture_output:
        print_banner("6/9. Running Playwright End-to-End Tests (Chromium)")
    run_step(["npm", "run", "test:e2e"], cwd=DESKTOP_DIR, env=env, capture_output=capture_output)
    if not capture_output:
        print_status("PASS", "All Playwright E2E browser tests passed!", "32")


def _gate_ghidra(env: Mapping[str, str], *, capture_output: bool = False) -> None:
    if not capture_output:
        print_banner("7/9. Validating Ghidra Native Generator (panadiga ghidra)")
    ghidra_dir = str(ROOT / "tools" / "ghidra")
    run_step(
        ["cargo", "run", "-p", "panadiga-cli", "--", "ghidra", "export-all", "--dir", ghidra_dir],
        cwd=ROOT,
        env=env,
        capture_output=capture_output,
    )
    if not capture_output:
        print_status("PASS", "Ghidra scripts generated directly from native Rust SSOT!", "32")


def _gate_c11_host(env: Mapping[str, str], *, capture_output: bool = False) -> None:
    if not capture_output:
        print_banner("8/9. Compiling & Executing C11 Pico Firmware Host Tests")
    exe_name = "test_gdb_rsp_host.exe" if sys.platform == "win32" else "test_gdb_rsp_host"
    with tempfile.TemporaryDirectory(prefix="panadiga_c11_") as tmpdir_str:
        work_dir = Path(tmpdir_str)
        exe_path = work_dir / exe_name
        _compile_c11_host_test(exe_path, work_dir, env, capture_output=capture_output)
        run_step([str(exe_path)], cwd=ROOT, env=env, capture_output=capture_output)
    if not capture_output:
        print_status("PASS", "All C11 Pico firmware host tests compiled and passed!", "32")


def _gate_python_mypy(env: Mapping[str, str], *, capture_output: bool = False) -> None:
    run_step([sys.executable, "-m", "mypy", "--strict", "scripts"], cwd=ROOT, env=env, capture_output=capture_output)


def _gate_python_compile_and_pytest(env: Mapping[str, str], *, capture_output: bool = False) -> None:
    run_step([sys.executable, "-m", "py_compile", *PYTHON_COMPILE_TARGETS], cwd=ROOT, env=env, capture_output=capture_output)
    run_step([sys.executable, "-m", "pytest", "scripts/tests"], cwd=ROOT, env=env, capture_output=capture_output)


def _gate_python(env: Mapping[str, str], *, capture_output: bool = False) -> None:
    if not capture_output:
        print_banner("9/9. Verifying Build Scripts, Ghidra Jython Syntax, Mypy Strict & Pytest")
    _gate_python_mypy(env, capture_output=capture_output)
    _gate_python_compile_and_pytest(env, capture_output=capture_output)
    if not capture_output:
        print_status("PASS", "Python build scripts, Ghidra syntax, mypy --strict, and pytest passed!", "32")


def _lane_rust_and_ghidra(env: Mapping[str, str]) -> None:
    _gate_rust(env, capture_output=True)
    _gate_ghidra(env, capture_output=True)


def _run_parallel_lanes(env: Mapping[str, str], max_jobs: int) -> None:
    lanes: tuple[tuple[str, Callable[[Mapping[str, str]], None]], ...] = (
        ("1+7/9 Rust Workspace + Ghidra", _lane_rust_and_ghidra),
        ("2/9 TypeScript (tsc --noEmit)", lambda e: _gate_typescript(e, capture_output=True)),
        ("3/9 Vitest Unit Suite", lambda e: _gate_vitest(e, capture_output=True)),
        ("4/9 Copy/Paste (jscpd 0%)", lambda e: _gate_jscpd(e, capture_output=True)),
        ("5/9 UI/UX State Lint", lambda e: _gate_uxlint(e, capture_output=True)),
        ("6/9 Playwright E2E", lambda e: _gate_playwright(e, capture_output=True)),
        ("8/9 C11 Firmware Host Tests", lambda e: _gate_c11_host(e, capture_output=True)),
        ("9a/9 Python Mypy Strict", lambda e: _gate_python_mypy(e, capture_output=True)),
        ("9b/9 Python Compile + Pytest", lambda e: _gate_python_compile_and_pytest(e, capture_output=True)),
    )
    worker_count = max(1, min(max_jobs, len(lanes)))
    print_banner(
        f"PANADIGA PARALLEL QUALITY GATES ({len(lanes)} lanes | max_workers={worker_count})"
    )

    tracker = ProcessTracker()

    def _timed_run(label: str, fn: Callable[[Mapping[str, str]], None]) -> tuple[str, float]:
        t0 = time.monotonic()
        with use_process_tracker(tracker):
            fn(env)
        return label, time.monotonic() - t0

    with ThreadPoolExecutor(max_workers=worker_count) as pool:
        future_map: dict[Future[tuple[str, float]], str] = {
            pool.submit(_timed_run, label, fn): label for label, fn in lanes
        }
        pending: set[Future[tuple[str, float]]] = set(future_map)
        try:
            while pending:
                done, pending = wait(pending, return_when=FIRST_EXCEPTION)
                for fut in done:
                    label = future_map[fut]
                    try:
                        done_label, dt = fut.result()
                        print_status("PASS", f"{done_label} ({dt:.2f}s)", "32")
                    except subprocess.CalledProcessError as exc:
                        for p in pending:
                            p.cancel()
                        tracker.terminate_all()
                        captured = exc.output or exc.stdout or exc.stderr or ""
                        print_status("FAIL", f"{label} failed (exit {exc.returncode})", "31")
                        if captured:
                            print(captured, flush=True)
                        raise
                    except Exception:
                        for p in pending:
                            p.cancel()
                        tracker.terminate_all()
                        print_status("FAIL", f"{label} failed", "31")
                        raise
        except KeyboardInterrupt:
            for p in pending:
                p.cancel()
            tracker.terminate_all()
            raise


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Panadiga 9 Quality Gates runner.")
    parser.add_argument(
        "--sequential",
        action="store_true",
        help="Run all 9 quality gates sequentially with live unbuffered logs.",
    )
    parser.add_argument(
        "--jobs",
        type=int,
        default=max(2, min(9, os.cpu_count() or 4)),
        help="Maximum parallel validation lanes (default: min(9, cpu_count)).",
    )
    return parser.parse_args(argv)


def run_validation(
    env: Mapping[str, str],
    *,
    sequential: bool = False,
    max_jobs: int | None = None,
) -> float:
    """Execute all 9 Quality Gates and return total elapsed seconds."""
    start = time.monotonic()
    jobs = max_jobs if max_jobs is not None else max(2, min(9, os.cpu_count() or 4))
    if sequential:
        for gate in (
            _gate_rust,
            _gate_typescript,
            _gate_vitest,
            _gate_jscpd,
            _gate_uxlint,
            _gate_playwright,
            _gate_ghidra,
            _gate_c11_host,
            _gate_python,
        ):
            gate(env)
    else:
        _run_parallel_lanes(env, jobs)

    elapsed = time.monotonic() - start
    print_banner(
        f"ALL 9 COMMIT QUALITY GATES PASSED! (Elapsed: {elapsed:.1f}s)\n"
        "Repository is in pristine condition for git commit & push.",
        ansi_color="32",
    )
    return elapsed


def main(argv: list[str] | None = None) -> int:
    """Execute all 9 mandatory Quality Gates (parallel DAG by default)."""
    args = parse_args(argv)
    env = build_env()
    try:
        run_validation(env, sequential=args.sequential, max_jobs=args.jobs)
    except (subprocess.CalledProcessError, RuntimeError, FileNotFoundError) as exc:
        print_status("FAIL", f"Validation gate failed: {exc}", "31")
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
