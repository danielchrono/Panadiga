"""Shared toolchain discovery, subprocess execution, logging, and SHA-256 helpers."""

from __future__ import annotations

from contextlib import contextmanager
import hashlib
import os
import shutil
import signal
import subprocess
import sys
import threading
from pathlib import Path
from typing import Any, Callable, Final, Iterator, Mapping, Sequence

ROOT: Final[Path] = Path(__file__).resolve().parent.parent
TOOLS_PYCACHE_IGNORE: Final[frozenset[str]] = frozenset(
    {"__pycache__", ".mypy_cache", ".pytest_cache", ".ruff_cache"}
)
_MSVC_CACHE_LOCK = threading.Lock()
_MSVC_DELTA_CACHE: dict[str, str] | None = None
_MSVC_CACHE_POPULATED = False
_PRINT_LOCK = threading.Lock()
_MANIFEST_LOCK = threading.Lock()


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


class ProcessAbortedError(RuntimeError):
    """Raised when a step is skipped or terminated after sibling cancellation."""


class _TrackerLocal(threading.local):
    tracker: ProcessTracker | None = None


_TRACKER_LOCAL = _TrackerLocal()


def _subprocess_group_kwargs() -> dict[str, Any]:
    if sys.platform == "win32":
        flag = int(getattr(subprocess, "CREATE_NEW_PROCESS_GROUP", 0))
        return {"creationflags": flag} if flag else {}
    return {"start_new_session": True}


def _terminate_process_tree(proc: subprocess.Popen[Any]) -> None:
    """Force-terminate a child process and its descendants without raising if already exited."""
    if proc.poll() is not None:
        return
    if sys.platform == "win32":
        subprocess.run(
            ["taskkill", "/F", "/T", "/PID", str(proc.pid)],
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            check=False,
        )
    else:
        killpg = getattr(os, "killpg", None)
        sigkill = getattr(signal, "SIGKILL", None)
        if callable(killpg) and sigkill is not None:
            try:
                killpg(proc.pid, sigkill)
            except OSError:
                pass
    try:
        if proc.poll() is None:
            proc.kill()
    except OSError:
        pass


class ProcessTracker:
    """Thread-safe registry of active child process trees for fail-fast sibling cancellation."""

    def __init__(self) -> None:
        self._lock = threading.Lock()
        self._aborted = False
        self._procs: set[subprocess.Popen[Any]] = set()

    def is_aborted(self) -> bool:
        """Return True once sibling cancellation has been triggered."""
        with self._lock:
            return self._aborted

    def register(self, proc: subprocess.Popen[Any]) -> bool:
        """Track a newly spawned process; returns False and terminates if already aborted."""
        with self._lock:
            if self._aborted:
                should_terminate = True
            else:
                self._procs.add(proc)
                should_terminate = False
        if should_terminate:
            _terminate_process_tree(proc)
            return False
        return True

    def unregister(self, proc: subprocess.Popen[Any]) -> None:
        """Stop tracking a finished child process."""
        with self._lock:
            self._procs.discard(proc)

    def terminate_all(self) -> None:
        """Mark tracker aborted and terminate every active child process tree."""
        with self._lock:
            self._aborted = True
            snapshot = list(self._procs)
            self._procs.clear()
        for proc in snapshot:
            _terminate_process_tree(proc)


def active_process_tracker() -> ProcessTracker | None:
    """Return the ProcessTracker bound to the current thread, if any."""
    return _TRACKER_LOCAL.tracker


@contextmanager
def use_process_tracker(tracker: ProcessTracker | None) -> Iterator[None]:
    """Bind a ProcessTracker to the current thread for fail-fast child cancellation."""
    previous = _TRACKER_LOCAL.tracker
    _TRACKER_LOCAL.tracker = tracker
    try:
        yield
    finally:
        _TRACKER_LOCAL.tracker = previous


def run_step(
    cmd: Sequence[str],
    *,
    cwd: Path | None = None,
    env: Mapping[str, str] | None = None,
    capture_output: bool = False,
) -> str:
    """Execute a command strictly and fail immediately on non-zero exit code."""
    if not cmd:
        raise ValueError("Cannot execute empty command sequence.")
    tracker = active_process_tracker()
    if tracker is not None and tracker.is_aborted():
        raise ProcessAbortedError(f"Skipped aborted step: {cmd[0]}")

    resolved = resolve_executable(cmd[0], env=env)
    argv = [resolved, *cmd[1:]]
    env_dict = dict(env) if env else None
    cwd_str = str(cwd or ROOT)

    if tracker is None:
        if capture_output:
            proc = subprocess.run(
                argv,
                cwd=cwd_str,
                env=env_dict,
                stdout=subprocess.PIPE,
                stderr=subprocess.STDOUT,
                text=True,
                encoding="utf-8",
                errors="replace",
                check=True,
            )
            return proc.stdout or ""
        subprocess.run(argv, cwd=cwd_str, env=env_dict, check=True)
        return ""

    group_kwargs = _subprocess_group_kwargs()
    child: subprocess.Popen[str] = subprocess.Popen(
        argv,
        cwd=cwd_str,
        env=env_dict,
        stdout=subprocess.PIPE if capture_output else None,
        stderr=subprocess.STDOUT if capture_output else None,
        text=True,
        encoding="utf-8" if capture_output else None,
        errors="replace" if capture_output else None,
        **group_kwargs,
    )
    if not tracker.register(child):
        try:
            child.wait(timeout=2)
        except subprocess.TimeoutExpired:
            pass
        raise ProcessAbortedError(f"Aborted step during launch: {cmd[0]}")

    try:
        stdout_data, _ = child.communicate()
    finally:
        tracker.unregister(child)

    if child.returncode != 0:
        if tracker.is_aborted():
            raise ProcessAbortedError(f"Terminated aborted step: {cmd[0]}")
        raise subprocess.CalledProcessError(child.returncode, argv, output=stdout_data)
    return stdout_data or ""


def print_banner(title: str, ansi_color: str = "36") -> None:
    """Print a standardized section banner."""
    sep = "=" * 70
    use_color = sys.stdout.isatty()
    prefix = f"\033[{ansi_color}m" if use_color else ""
    suffix = "\033[0m" if use_color else ""
    with _PRINT_LOCK:
        print(f"\n{prefix}{sep}\n>> {title}\n{sep}{suffix}", flush=True)


def print_status(tag: str, message: str, ansi_color: str) -> None:
    """Print a colored status tag line ([PASS], [FAIL], [INFO], [OK])."""
    with _PRINT_LOCK:
        if sys.stdout.isatty():
            print(f"\033[{ansi_color}m[{tag}]\033[0m {message}", flush=True)
        else:
            print(f"[{tag}] {message}", flush=True)


def sha256_file(path: Path) -> str:
    """Compute lowercase hexadecimal SHA-256 digest using zero-copy C file_digest when available."""
    file_digest_fn: Callable[[Any, str], Any] | None = getattr(hashlib, "file_digest", None)
    with path.open("rb") as handle:
        if file_digest_fn is not None:
            return str(file_digest_fn(handle, "sha256").hexdigest())
        digest = hashlib.sha256()
        while True:
            chunk = handle.read(1 << 20)
            if not chunk:
                break
            digest.update(chunk)
        return digest.hexdigest()


def update_sha256_manifest(manifest_path: Path, artifacts: Sequence[Path]) -> None:
    """Update or create SHA256SUMS.txt idempotently and thread-safely for given artifact paths."""
    with _MANIFEST_LOCK:
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


def _executable_suffix(stem: str) -> str:
    """Return platform executable filename for a Cargo binary stem."""
    return f"{stem}.exe" if sys.platform == "win32" else stem


def panadiga_binary_name() -> str:
    """Return native Panadiga CLI binary filename for the current OS."""
    return _executable_suffix("panadiga")


def panadiga_desktop_binary_name() -> str:
    """Return native Panadiga Tauri Desktop GUI binary filename for the current OS."""
    return _executable_suffix("panadiga-desktop")


def is_wsl() -> bool:
    """Detect whether execution runs inside Windows Subsystem for Linux."""
    proc_ver = Path("/proc/version")
    if not proc_ver.is_file():
        return False
    return "microsoft" in proc_ver.read_text(encoding="utf-8", errors="ignore").lower()


def _locate_binary(stem: str) -> Path | None:
    """Locate a prebuilt release binary by stem in dist/ or target/release/."""
    bin_name = _executable_suffix(stem)
    candidates: list[Path] = [
        ROOT / "dist" / bin_name,
        ROOT / "target" / "release" / bin_name,
    ]
    if is_wsl():
        candidates.extend([ROOT / "dist" / f"{stem}.exe", ROOT / "target" / "release" / f"{stem}.exe"])
    for cand in candidates:
        if cand.is_file():
            return cand
    return None


def locate_panadiga_binary() -> Path | None:
    """Locate prebuilt release Panadiga CLI binary in dist/ or target/release/."""
    return _locate_binary("panadiga")


def locate_panadiga_desktop_binary() -> Path | None:
    """Locate prebuilt release Panadiga Desktop GUI binary in dist/ or target/release/."""
    return _locate_binary("panadiga-desktop")


def find_msvc_vcvars64() -> Path | None:
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


def _discover_msvc_delta() -> dict[str, str] | None:
    """Discover MSVC x64 toolchain environment variables once per process."""
    global _MSVC_DELTA_CACHE, _MSVC_CACHE_POPULATED
    with _MSVC_CACHE_LOCK:
        if _MSVC_CACHE_POPULATED:
            return _MSVC_DELTA_CACHE

        vcvars = find_msvc_vcvars64()
        if vcvars is None:
            _MSVC_DELTA_CACHE = None
            _MSVC_CACHE_POPULATED = True
            return None

        probe_env: dict[str, str] = {}
        for sys_key in (
            "SystemDrive",
            "SystemRoot",
            "WINDIR",
            "ComSpec",
            "TEMP",
            "TMP",
            "ProgramData",
            "ALLUSERSPROFILE",
            "ProgramFiles",
            "ProgramFiles(x86)",
            "CommonProgramFiles",
            "CommonProgramFiles(x86)",
            "USERPROFILE",
            "APPDATA",
            "LOCALAPPDATA",
        ):
            if sys_key in os.environ:
                probe_env[sys_key] = os.environ[sys_key]
        sys_root = probe_env.get("SystemRoot") or r"C:\Windows"
        probe_env["PATH"] = f"{sys_root}\\System32;{sys_root}"
        cmd_line = f'cmd.exe /d /s /c "call "{vcvars}" >nul 2>&1 && set"'
        try:
            proc = subprocess.run(
                cmd_line,
                env=probe_env,
                stdin=subprocess.DEVNULL,
                capture_output=True,
                text=True,
                timeout=20,
                check=False,
            )
        except subprocess.TimeoutExpired:
            _MSVC_DELTA_CACHE = None
            _MSVC_CACHE_POPULATED = True
            return None

        if proc.returncode != 0:
            _MSVC_DELTA_CACHE = None
            _MSVC_CACHE_POPULATED = True
            return None

        delta: dict[str, str] = {}
        for line in proc.stdout.splitlines():
            key, sep, value = line.partition("=")
            if sep and key:
                delta[key] = value
        _MSVC_DELTA_CACHE = delta
        _MSVC_CACHE_POPULATED = True
        return _MSVC_DELTA_CACHE


def load_msvc_env(base_env: Mapping[str, str]) -> dict[str, str]:
    """Return environment augmented with MSVC x64 toolchain variables (cached across calls)."""
    env = dict(base_env)
    if sys.platform != "win32":
        return env
    if shutil.which("cl", path=env.get("PATH")) and "INCLUDE" in env and "LIB" in env:
        return env

    delta = _discover_msvc_delta()
    if delta is None:
        return env

    merged = {k: v for k, v in env.items() if k.upper() != "PATH"}
    base_path = env.get("PATH", "")
    for key, value in delta.items():
        if key.upper() == "PATH":
            merged["PATH"] = f"{value}{os.pathsep}{base_path}" if base_path else value
        else:
            merged[key] = value
    merged.setdefault("PATH", base_path)
    return merged
