"""Shared toolchain discovery, subprocess execution, logging, and SHA-256 helpers."""

from __future__ import annotations

import hashlib
import os
import shutil
import subprocess
import sys
from pathlib import Path
from typing import Final, Mapping, Sequence

ROOT: Final[Path] = Path(__file__).resolve().parent.parent
TOOLS_PYCACHE_IGNORE: Final[frozenset[str]] = frozenset(
    {"__pycache__", ".mypy_cache", ".pytest_cache", ".ruff_cache"}
)


def build_env() -> dict[str, str]:
    """Return process environment with Cargo user bin prepended when present."""
    env = dict(os.environ)
    cargo_bin = Path.home() / ".cargo" / "bin"
    if cargo_bin.is_dir():
        current_path = env.get("PATH", "")
        cargo_str = str(cargo_bin)
        if cargo_str not in current_path.split(os.pathsep):
            env["PATH"] = f"{cargo_str}{os.pathsep}{current_path}" if current_path else cargo_str
    return env


def resolve_executable(name: str, env: Mapping[str, str] | None = None) -> str:
    """Resolve an executable path using PATH (handles .cmd/.exe cleanly on Windows)."""
    path_arg = env.get("PATH") if env is not None else None
    found = shutil.which(name, path=path_arg)
    if found is None:
        raise FileNotFoundError(f"Required executable '{name}' not found on PATH.")
    return found


def run_step(
    cmd: Sequence[str],
    *,
    cwd: Path | None = None,
    env: Mapping[str, str] | None = None,
) -> None:
    """Execute a command strictly and fail immediately on non-zero exit code."""
    if not cmd:
        raise ValueError("Cannot execute empty command sequence.")
    resolved = resolve_executable(cmd[0], env=env)
    argv = [resolved, *cmd[1:]]
    subprocess.run(argv, cwd=str(cwd or ROOT), env=dict(env) if env else None, check=True)


def print_banner(title: str, ansi_color: str = "36") -> None:
    """Print a standardized section banner."""
    sep = "=" * 70
    use_color = sys.stdout.isatty()
    prefix = f"\033[{ansi_color}m" if use_color else ""
    suffix = "\033[0m" if use_color else ""
    print(f"\n{prefix}{sep}\n>> {title}\n{sep}{suffix}", flush=True)


def print_status(tag: str, message: str, ansi_color: str) -> None:
    """Print a colored status tag line ([PASS], [FAIL], [INFO], [OK])."""
    if sys.stdout.isatty():
        print(f"\033[{ansi_color}m[{tag}]\033[0m {message}", flush=True)
    else:
        print(f"[{tag}] {message}", flush=True)


def sha256_file(path: Path) -> str:
    """Compute lowercase hexadecimal SHA-256 digest of a file via chunked streaming."""
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        while True:
            chunk = handle.read(65536)
            if not chunk:
                break
            digest.update(chunk)
    return digest.hexdigest()


def update_sha256_manifest(manifest_path: Path, artifacts: Sequence[Path]) -> None:
    """Update or create SHA256SUMS.txt idempotently for the given artifact paths."""
    existing_lines: list[str] = []
    target_names = {art.name for art in artifacts}
    if manifest_path.is_file():
        for line in manifest_path.read_text(encoding="utf-8").splitlines():
            stripped = line.strip()
            if not stripped:
                continue
            parts = stripped.split()
            entry_name = parts[-1] if parts else ""
            if entry_name not in target_names:
                existing_lines.append(stripped)

    for art in artifacts:
        if art.is_file():
            existing_lines.append(f"{sha256_file(art).upper()}  {art.name}")

    manifest_path.parent.mkdir(parents=True, exist_ok=True)
    manifest_path.write_text("\n".join(existing_lines) + "\n", encoding="utf-8")


def panadiga_binary_name() -> str:
    """Return native Panadiga binary filename for the current OS."""
    return "panadiga.exe" if sys.platform == "win32" else "panadiga"


def is_wsl() -> bool:
    """Detect whether execution runs inside Windows Subsystem for Linux."""
    proc_ver = Path("/proc/version")
    if not proc_ver.is_file():
        return False
    return "microsoft" in proc_ver.read_text(encoding="utf-8", errors="ignore").lower()


def locate_panadiga_binary() -> Path | None:
    """Locate prebuilt release Panadiga binary in dist/ or target/release/."""
    bin_name = panadiga_binary_name()
    candidates: list[Path] = [
        ROOT / "dist" / bin_name,
        ROOT / "target" / "release" / bin_name,
    ]
    if is_wsl():
        candidates.extend([ROOT / "dist" / "panadiga.exe", ROOT / "target" / "release" / "panadiga.exe"])
    for cand in candidates:
        if cand.is_file():
            return cand
    return None
