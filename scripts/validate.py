"""Panadiga universal 9 Quality Gates validation suite (Windows, Linux, macOS)."""

from __future__ import annotations

import os
import shutil
import subprocess
import sys
import tempfile
import time
from pathlib import Path
from typing import Final, Mapping

SCRIPT_DIR = Path(__file__).resolve().parent
if str(SCRIPT_DIR.parent) not in sys.path:
    sys.path.insert(0, str(SCRIPT_DIR.parent))

from scripts.panadiga_ops import ROOT, build_env, print_banner, print_status, run_step

DESKTOP_DIR: Final[Path] = ROOT / "apps" / "desktop"
C11_HOST_SOURCES: Final[tuple[str, ...]] = (
    "dwire32a_jtag_research/gdb/gdb_rsp_codec.c",
    "dwire32a_jtag_research/gdb/gdb_rsp_memory.c",
    "dwire32a_jtag_research/gdb/gdb_rsp_cpu.c",
    "dwire32a_jtag_research/tests/test_gdb_rsp_host.c",
)
PYTHON_COMPILE_TARGETS: Final[tuple[str, ...]] = (
    "tools/ghidra_am34_memory_map.py",
    "tools/ghidra_import_symbols_panadiga.py",
    "tools/ghidra_connect_debugger.py",
    "scripts/panadiga_ops.py",
    "scripts/build.py",
    "scripts/validate.py",
)


def _find_msvc_vcvars64() -> Path | None:
    """Locate MSVC vcvars64.bat via Visual Studio Locator on Windows."""
    if sys.platform != "win32":
        return None
    program_files_x86 = Path(os.environ.get("ProgramFiles(x86)", r"C:\Program Files (x86)"))
    vswhere = program_files_x86 / "Microsoft Visual Studio" / "Installer" / "vswhere.exe"
    if not vswhere.is_file():
        return None
    proc = subprocess.run(
        [str(vswhere), "-latest", "-products", "*", "-find", "**\\vcvars64.bat"],
        capture_output=True,
        text=True,
        check=False,
    )
    for line in proc.stdout.splitlines():
        cand = Path(line.strip())
        if cand.is_file():
            return cand
    return None


def _compile_c11_host_test(exe_path: Path, env: Mapping[str, str]) -> None:
    """Compile C11 Pico RSP host test suite with strict warnings enabled."""
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
                "dwire32a_jtag_research",
                *C11_HOST_SOURCES,
                "-o",
                str(exe_path),
            ]
            run_step(cmd, cwd=ROOT, env=env)
            return

    vcvars = _find_msvc_vcvars64()
    if vcvars is not None:
        sources_win = " ".join(src.replace("/", "\\") for src in C11_HOST_SOURCES)
        batch_lines = (
            f'@echo off\n'
            f'call "{vcvars}"\n'
            f'if errorlevel 1 exit /b 1\n'
            f'cd /d "{ROOT}"\n'
            f'cl /nologo /std:c11 /W4 /WX /DPICO_HOST_TEST=1 /D_CRT_SECURE_NO_WARNINGS '
            f'/Idwire32a_jtag_research {sources_win} /Fe"{exe_path}"\n'
        )
        with tempfile.NamedTemporaryFile("w", suffix=".cmd", delete=False, encoding="utf-8") as tmp:
            tmp.write(batch_lines)
            tmp_path = Path(tmp.name)
        try:
            subprocess.run(["cmd.exe", "/d", "/c", str(tmp_path)], cwd=str(ROOT), env=dict(env), check=True)
        finally:
            tmp_path.unlink(missing_ok=True)
        return

    raise RuntimeError("No C11 compiler (gcc, clang, or MSVC cl) found for Pico firmware host tests.")


def _gate_rust(env: Mapping[str, str]) -> None:
    print_banner("1/9. Validating Rust Workspace (Cargo Fmt, Check & Tests)")
    run_step(["cargo", "fmt", "--all", "--", "--check"], cwd=ROOT, env=env)
    run_step(["cargo", "check", "--workspace", "--all-targets"], cwd=ROOT, env=env)
    run_step(["cargo", "test", "--workspace", "--verbose"], cwd=ROOT, env=env)
    print_status("PASS", "Rust workspace formatted, compiled, and all tests passed!", "32")


def _gate_typescript(env: Mapping[str, str]) -> None:
    print_banner("2/9. Validating TypeScript Strict Types (tsc --noEmit)")
    run_step(["npm", "run", "typecheck"], cwd=DESKTOP_DIR, env=env)
    print_status("PASS", "TypeScript strict type check succeeded with 0 errors!", "32")


def _gate_vitest(env: Mapping[str, str]) -> None:
    print_banner("3/9. Executing Desktop Frontend Unit Tests (Vitest)")
    run_step(["npm", "test"], cwd=DESKTOP_DIR, env=env)
    print_status("PASS", "All Vitest frontend and ViewModel unit tests passed!", "32")


def _gate_jscpd(env: Mapping[str, str]) -> None:
    print_banner("4/9. Running Copy/Paste Detector (jscpd threshold 0% / 0 clones)")
    run_step(["npx", "--prefix", "apps/desktop", "jscpd", "--config", ".jscpd.json"], cwd=ROOT, env=env)
    print_status("PASS", "Code duplication verified at 0% (0 clones found)!", "32")


def _gate_uxlint(env: Mapping[str, str]) -> None:
    print_banner("5/9. Running UI/UX State Linter (uxlint)")
    run_step(["npm", "run", "uxlint"], cwd=DESKTOP_DIR, env=env)
    print_status("PASS", "UX state completeness audit passed!", "32")


def _gate_playwright(env: Mapping[str, str]) -> None:
    print_banner("6/9. Running Playwright End-to-End Tests (Chromium)")
    run_step(["npm", "run", "test:e2e"], cwd=DESKTOP_DIR, env=env)
    print_status("PASS", "All Playwright E2E browser tests passed!", "32")


def _gate_ghidra(env: Mapping[str, str]) -> None:
    print_banner("7/9. Validating Ghidra Native Generator (panadiga ghidra)")
    run_step(["cargo", "run", "--", "ghidra", "export-all", "--dir", str(ROOT / "tools")], cwd=ROOT, env=env)
    print_status("PASS", "Ghidra scripts generated directly from native Rust SSOT!", "32")


def _gate_c11_host(env: Mapping[str, str]) -> None:
    print_banner("8/9. Compiling & Executing C11 Pico Firmware Host Tests")
    exe_name = "test_gdb_rsp_host.exe" if sys.platform == "win32" else "test_gdb_rsp_host"
    exe_path = ROOT / exe_name
    try:
        _compile_c11_host_test(exe_path, env)
        subprocess.run([str(exe_path)], cwd=str(ROOT), env=dict(env), check=True)
    finally:
        if exe_path.exists():
            exe_path.unlink(missing_ok=True)
        for obj_file in ROOT.glob("*.obj"):
            obj_file.unlink(missing_ok=True)
    print_status("PASS", "All C11 Pico firmware host tests compiled and passed!", "32")


def _gate_python(env: Mapping[str, str]) -> None:
    print_banner("9/9. Verifying Python Typer CLI, Mypy Strict & Pytest Suite")
    run_step([sys.executable, "-m", "py_compile", *PYTHON_COMPILE_TARGETS], cwd=ROOT, env=env)
    run_step(
        [sys.executable, "-m", "mypy", "--strict", "tools/panadiga_cli.py", "tools/rag_engine", "scripts"],
        cwd=ROOT,
        env=env,
    )
    run_step([sys.executable, "-m", "pytest", "tools/tests", "tools/rag_engine/tests"], cwd=ROOT, env=env)
    print_status("PASS", "Python syntax, mypy --strict, and pytest suites all passed!", "32")


def main() -> int:
    """Execute all 9 mandatory Quality Gates sequentially."""
    start = time.monotonic()
    env = build_env()
    gates = (
        _gate_rust,
        _gate_typescript,
        _gate_vitest,
        _gate_jscpd,
        _gate_uxlint,
        _gate_playwright,
        _gate_ghidra,
        _gate_c11_host,
        _gate_python,
    )
    try:
        for gate in gates:
            gate(env)
    except (subprocess.CalledProcessError, RuntimeError, FileNotFoundError) as exc:
        print_status("FAIL", f"Validation gate failed: {exc}", "31")
        return 1

    elapsed = time.monotonic() - start
    print_banner(
        f"ALL 9 COMMIT QUALITY GATES PASSED! (Elapsed: {elapsed:.1f}s)\n"
        "Repository is in pristine condition for git commit & push.",
        ansi_color="32",
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
