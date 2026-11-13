"""Recursive workspace discovery, compiler/environment caching, and content-hash step cache."""

from __future__ import annotations

from dataclasses import dataclass
import hashlib
import json
import os
import platform
import shutil
import subprocess
import sys
import threading
from pathlib import Path
from typing import Any, Final, Mapping, Sequence

from scripts.panadiga_ops import ROOT, load_msvc_env, sha256_file

BUILD_IGNORE_DIRS: Final[frozenset[str]] = frozenset(
    {
        ".git",
        ".github",
        ".jscpd-report",
        ".mypy_cache",
        ".pytest_cache",
        ".ruff_cache",
        ".vite",
        "__pycache__",
        "build",
        "dist",
        "node_modules",
        "pico-build",
        "playwright-report",
        "target",
        "test-results",
    }
)
CACHE_DIR_NAME: Final[str] = ".panadiga-cache"
MANIFEST_FILENAME: Final[str] = "steps.json"
_CACHE_LOCK = threading.Lock()


@dataclass(frozen=True)
class WorkspaceInventory:
    """Discovered workspace layout, host capabilities, toolchains, and active caches."""

    root: Path
    os_name: str
    arch: str
    cpu_count: int
    crates: tuple[str, ...]
    apps: tuple[str, ...]
    firmware_targets: tuple[str, ...]
    tools: dict[str, str]


@dataclass(frozen=True)
class CacheEntryStat:
    """Size metadata for a build or compiler cache directory."""

    label: str
    path: Path
    exists: bool
    bytes_total: int


def find_repo_root(start: Path | None = None) -> Path:
    """Walk recursively upward to find the Panadiga repository root (Cargo.toml + scripts/build.py)."""
    probe = (start or Path.cwd()).resolve()
    if probe.is_file():
        probe = probe.parent
    current = probe
    while True:
        if (current / "Cargo.toml").is_file() and (current / "scripts" / "build.py").is_file():
            return current
        parent = current.parent
        if parent == current:
            return ROOT
        current = parent


def _iter_files_recursive(
    base: Path,
    *,
    extensions: frozenset[str] | None = None,
    ignore_prefixes: tuple[str, ...] = (),
) -> list[Path]:
    """Recursively collect files sorted deterministically while skipping build artifacts."""
    if not base.exists():
        return []
    if base.is_file():
        return [base]

    collected: list[Path] = []
    for dirpath_str, dirnames, filenames in os.walk(base):
        dirnames[:] = sorted(
            d
            for d in dirnames
            if d not in BUILD_IGNORE_DIRS
            and not any(d.startswith(prefix) for prefix in ignore_prefixes)
        )
        dirpath = Path(dirpath_str)
        for fname in sorted(filenames):
            fpath = dirpath / fname
            if extensions is not None and fpath.suffix.lower() not in extensions and fname not in extensions:
                continue
            collected.append(fpath)
    collected.sort(key=lambda p: p.as_posix())
    return collected


def _discover_manifests_recursive(root: Path, subdir: str, marker_filename: str) -> tuple[str, ...]:
    """Recursively discover workspace subprojects containing a specific manifest file."""
    base = root / subdir
    if not base.is_dir():
        return ()
    found: list[str] = []
    for path in _iter_files_recursive(base, extensions=frozenset({marker_filename}), ignore_prefixes=("pico-build",)):
        if path.name == marker_filename:
            found.append(path.parent.relative_to(root).as_posix())
    return tuple(sorted(set(found)))


def discover_pico_sdk(root: Path, env: Mapping[str, str]) -> Path | None:
    """Locate Raspberry Pi Pico SDK across environment variables and common install paths."""
    sdk_env = env.get("PICO_SDK_PATH")
    if sdk_env and Path(sdk_env).is_dir():
        return Path(sdk_env).resolve()
    candidates = (
        Path(r"C:\Program Files\Raspberry Pi\Pico SDK v2.0.0\pico-sdk"),
        Path(r"C:\Program Files\Raspberry Pi\Pico SDK v1.5.1\pico-sdk"),
        Path(r"C:\pico\pico-sdk"),
        Path(r"C:\pico-sdk"),
        Path.home() / "pico-sdk",
        Path.home() / ".pico-sdk" / "sdk" / "2.1.1",
        Path.home() / ".pico-sdk" / "sdk" / "2.0.0",
        Path.home() / ".pico-sdk" / "sdk" / "1.5.1",
        root / "pico-sdk",
    )
    for cand in candidates:
        if cand.is_dir():
            return cand.resolve()
    return None


def _discover_android_paths(env: Mapping[str, str]) -> tuple[Path | None, Path | None]:
    """Detect Android SDK and NDK directories without raising when absent."""
    sdk_env = env.get("ANDROID_HOME") or env.get("ANDROID_SDK_ROOT")
    sdk_dir: Path | None = Path(sdk_env) if sdk_env and Path(sdk_env).is_dir() else None
    if sdk_dir is None:
        fallback = (
            Path(env.get("LOCALAPPDATA", "")) / "Android" / "Sdk"
            if sys.platform == "win32"
            else Path.home() / "Android" / "Sdk"
        )
        if fallback.is_dir():
            sdk_dir = fallback
    ndk_env = env.get("NDK_HOME") or env.get("ANDROID_NDK_ROOT")
    ndk_dir: Path | None = Path(ndk_env) if ndk_env and Path(ndk_env).is_dir() else None
    if ndk_dir is None and sdk_dir is not None and (sdk_dir / "ndk").is_dir():
        ndk_versions = sorted(p for p in (sdk_dir / "ndk").iterdir() if p.is_dir())
        if ndk_versions:
            ndk_dir = ndk_versions[-1]
    return sdk_dir, ndk_dir


def _augment_windows_toolchain_path(env: dict[str, str]) -> None:
    """Prepend standard Windows ARM GCC and VS CMake/Ninja folders if not yet on PATH."""
    if sys.platform != "win32":
        return
    extra_dirs: list[Path] = []
    current_path = env.get("PATH", "")
    if not shutil.which("arm-none-eabi-gcc", path=current_path):
        for arm_base in (
            Path(r"C:\Program Files (x86)\Arm GNU Toolchain arm-none-eabi"),
            Path(r"C:\Program Files\Arm GNU Toolchain arm-none-eabi"),
        ):
            if arm_base.is_dir():
                for bin_dir in sorted(arm_base.glob("*/bin"), reverse=True):
                    if (bin_dir / "arm-none-eabi-gcc.exe").is_file():
                        extra_dirs.append(bin_dir)
                        break
    vs_cmake = Path(
        r"C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\Common7\IDE\CommonExtensions\Microsoft\CMake"
    )
    if not shutil.which("cmake", path=current_path) and (vs_cmake / "CMake" / "bin").is_dir():
        extra_dirs.append(vs_cmake / "CMake" / "bin")
    if not shutil.which("ninja", path=current_path) and (vs_cmake / "Ninja").is_dir():
        extra_dirs.append(vs_cmake / "Ninja")

    if extra_dirs:
        prefix = os.pathsep.join(str(d) for d in extra_dirs)
        env["PATH"] = f"{prefix}{os.pathsep}{current_path}" if current_path else prefix


def _resolve_tool(name: str, env: Mapping[str, str]) -> str | None:
    path_str = env.get("PATH")
    found = shutil.which(name, path=path_str)
    if found:
        return found
    cargo_candidate = Path.home() / ".cargo" / "bin" / (f"{name}.exe" if sys.platform == "win32" else name)
    return str(cargo_candidate) if cargo_candidate.is_file() else None


def optimize_build_env(
    base_env: Mapping[str, str],
    *,
    root: Path = ROOT,
    use_cache: bool = True,
) -> tuple[dict[str, str], dict[str, str]]:
    """Return environment configured for maximum parallelism and available compiler caches."""
    env = dict(base_env)
    active: dict[str, str] = {}

    jobs = str(max(1, os.cpu_count() or 4))
    env.setdefault("CARGO_BUILD_JOBS", jobs)
    env.setdefault("CMAKE_BUILD_PARALLEL_LEVEL", jobs)
    env.setdefault("NINJA_STATUS", "[%f/%t %es] ")
    env.setdefault("PYTHONUTF8", "1")
    active["jobs"] = env["CARGO_BUILD_JOBS"]

    if sys.platform != "win32":
        env.setdefault("MAKEFLAGS", f"-j{jobs}")

    _augment_windows_toolchain_path(env)

    pico_sdk = discover_pico_sdk(root, env)
    if pico_sdk is not None:
        env.setdefault("PICO_SDK_PATH", str(pico_sdk))

    if use_cache:
        sccache_bin = _resolve_tool("sccache", env)
        if sccache_bin and not env.get("RUSTC_WRAPPER"):
            env["RUSTC_WRAPPER"] = sccache_bin
            active["rustc_wrapper"] = sccache_bin
        ccache_bin = _resolve_tool("ccache", env)
        if ccache_bin:
            env.setdefault("PANADIGA_CCACHE", ccache_bin)
            env.setdefault("CCACHE_COMPILERCHECK", "content")
            active["ccache"] = ccache_bin
        lock_src = root / "apps" / "desktop" / "package-lock.json"
        lock_dst = root / "apps" / "desktop" / "node_modules" / ".package-lock.json"
        if lock_src.is_file() and lock_dst.is_file():
            env.setdefault("npm_config_prefer_offline", "true")
            active["npm_offline"] = "true"
        active["step_cache"] = "enabled"
    else:
        active["step_cache"] = "disabled"

    return env, active


def discover_workspace_inventory(root: Path, env: Mapping[str, str]) -> WorkspaceInventory:
    """Recursively scan workspace components and available cross-platform toolchains."""
    opt_env, _ = optimize_build_env(env, root=root, use_cache=True)
    path_val = opt_env.get("PATH", "")

    crates = _discover_manifests_recursive(root, "crates", "Cargo.toml") + _discover_manifests_recursive(
        root, "apps", "Cargo.toml"
    )
    apps = _discover_manifests_recursive(root, "apps", "package.json")
    firmware_targets = _discover_manifests_recursive(root, "firmware", "CMakeLists.txt")

    tools: dict[str, str] = {}
    for tool_name in (
        "cargo",
        "rustc",
        "node",
        "npm",
        "arm-none-eabi-gcc",
        "cmake",
        "ninja",
        "sccache",
        "ccache",
        "java",
    ):
        resolved = _resolve_tool(tool_name, opt_env)
        if resolved:
            tools[tool_name] = resolved

    for c_cand in ("gcc", "clang"):
        resolved_c = shutil.which(c_cand, path=path_val)
        if resolved_c:
            tools["c11_host"] = resolved_c
            break
    if "c11_host" not in tools and sys.platform == "win32":
        msvc_env = load_msvc_env(opt_env)
        cl_path = shutil.which("cl", path=msvc_env.get("PATH"))
        if cl_path:
            tools["c11_host"] = cl_path

    pico_sdk = discover_pico_sdk(root, opt_env)
    if pico_sdk is not None:
        tools["pico_sdk"] = str(pico_sdk)

    android_sdk, android_ndk = _discover_android_paths(opt_env)
    if android_sdk is not None:
        tools["android_sdk"] = str(android_sdk)
    if android_ndk is not None:
        tools["android_ndk"] = str(android_ndk)

    return WorkspaceInventory(
        root=root,
        os_name=platform.system(),
        arch=platform.machine(),
        cpu_count=max(1, os.cpu_count() or 4),
        crates=crates,
        apps=apps,
        firmware_targets=firmware_targets,
        tools=tools,
    )


def _manifest_path(cache_dir: Path) -> Path:
    return cache_dir / MANIFEST_FILENAME


def default_cache_dir(root: Path = ROOT) -> Path:
    return root / "target" / CACHE_DIR_NAME


def _load_manifest_unlocked(cache_dir: Path) -> dict[str, Any]:
    mf = _manifest_path(cache_dir)
    if not mf.is_file():
        return {}
    try:
        loaded = json.loads(mf.read_text(encoding="utf-8"))
        return loaded if isinstance(loaded, dict) else {}
    except (OSError, ValueError):
        return {}


def _write_manifest_unlocked(cache_dir: Path, data: Mapping[str, Any]) -> None:
    cache_dir.mkdir(parents=True, exist_ok=True)
    tmp = _manifest_path(cache_dir).with_suffix(".tmp")
    tmp.write_text(json.dumps(data, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    tmp.replace(_manifest_path(cache_dir))


def compute_tree_digest(
    root: Path,
    input_paths: Sequence[Path],
    *,
    extensions: frozenset[str] | None = None,
    extra_salt: Sequence[str] = (),
) -> str:
    """Compute deterministic SHA-256 across files, directories, and toolchain salt strings."""
    hasher = hashlib.sha256()
    seen: set[Path] = set()
    files: list[Path] = []
    for p in input_paths:
        for item in _iter_files_recursive(p, extensions=extensions, ignore_prefixes=("build",)):
            resolved = item.resolve()
            if resolved not in seen:
                seen.add(resolved)
                files.append(item)

    files.sort(key=lambda f: f.as_posix())
    for item in files:
        try:
            rel = item.resolve().relative_to(root.resolve()).as_posix()
        except ValueError:
            rel = item.name
        hasher.update(rel.encode("utf-8"))
        hasher.update(b"\0")
        hasher.update(sha256_file(item).encode("ascii"))
        hasher.update(b"\n")

    for salt in extra_salt:
        hasher.update(salt.encode("utf-8", errors="replace"))
        hasher.update(b"\0")
    return hasher.hexdigest()


def is_step_fresh(
    root: Path,
    step_key: str,
    input_digest: str,
    outputs: Sequence[Path],
    *,
    cache_dir: Path | None = None,
) -> bool:
    """Return True when input_digest and all recorded output file SHA-256 digests match."""
    cdir = cache_dir or default_cache_dir(root)
    with _CACHE_LOCK:
        manifest = _load_manifest_unlocked(cdir)
    entry = manifest.get(step_key)
    if not isinstance(entry, dict):
        return False
    if entry.get("input_digest") != input_digest:
        return False
    recorded_outputs = entry.get("outputs")
    if not isinstance(recorded_outputs, dict):
        return False

    for out_path in outputs:
        if not out_path.is_file():
            return False
        rel = _relative_key(root, out_path)
        if recorded_outputs.get(rel) != sha256_file(out_path):
            return False
    return True


def record_step_cache(
    root: Path,
    step_key: str,
    input_digest: str,
    outputs: Sequence[Path],
    *,
    cache_dir: Path | None = None,
) -> None:
    """Persist input digest and output SHA-256 checksums for subsequent incremental builds."""
    cdir = cache_dir or default_cache_dir(root)
    out_hashes: dict[str, str] = {}
    for out_path in outputs:
        if out_path.is_file():
            out_hashes[_relative_key(root, out_path)] = sha256_file(out_path)

    with _CACHE_LOCK:
        manifest = _load_manifest_unlocked(cdir)
        manifest[step_key] = {"input_digest": input_digest, "outputs": out_hashes}
        _write_manifest_unlocked(cdir, manifest)


def _relative_key(root: Path, path: Path) -> str:
    try:
        return path.resolve().relative_to(root.resolve()).as_posix()
    except ValueError:
        return path.as_posix()


def copy_if_changed(src: Path, dst: Path) -> bool:
    """Copy src -> dst only when dst is absent or its SHA-256 differs, preserving mtime on hits."""
    if not src.is_file():
        raise FileNotFoundError(f"Source artifact missing: {src}")
    if dst.is_file() and sha256_file(src) == sha256_file(dst):
        return False
    dst.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(src, dst)
    return True


def frontend_digest_and_outputs(root: Path) -> tuple[str, tuple[Path, ...]]:
    """Compute input digest and expected artifact paths for the React/Vite desktop UI."""
    app_dir = root / "apps" / "desktop"
    inputs = (
        app_dir / "src",
        app_dir / "index.html",
        app_dir / "package.json",
        app_dir / "package-lock.json",
        app_dir / "tsconfig.json",
        app_dir / "tsconfig.node.json",
        app_dir / "vite.config.ts",
    )
    outputs = (app_dir / "dist" / "index.html",)
    return compute_tree_digest(root, inputs), outputs


def pico_digest_and_outputs(root: Path, env: Mapping[str, str]) -> tuple[str, tuple[Path, ...]]:
    """Compute input digest and UF2 output paths for RP2040/RP2350 Pico firmware."""
    fw_dir = root / "firmware" / "pico-p7501"
    dest_dir = root / "crates" / "panadiga-core" / "src" / "pico" / "firmware_bins"
    salt = (
        f"pico_sdk={env.get('PICO_SDK_PATH', '')}",
        f"ccache={env.get('PANADIGA_CCACHE', '')}",
    )
    exts = frozenset({".c", ".h", ".pio", ".cmake", "CMakeLists.txt"})
    inputs = (fw_dir,)
    outputs = (
        dest_dir / "panasonic_p7501_uart_rp2040.uf2",
        dest_dir / "panasonic_p7501_uart_rp2350.uf2",
        dest_dir / "panasonic_p7501_uart_unified.uf2",
    )
    return compute_tree_digest(root, inputs, extensions=exts, extra_salt=salt), outputs


def ghidra_digest_and_outputs(root: Path) -> tuple[str, tuple[Path, ...]]:
    """Compute input digest and generated Jython output paths for Ghidra SSOT scripts."""
    inputs = (root / "crates" / "panadiga-core" / "src" / "ssot",)
    ghidra_dir = root / "tools" / "ghidra"
    outputs = (
        ghidra_dir / "ghidra_am34_memory_map.py",
        ghidra_dir / "ghidra_import_symbols_panadiga.py",
        ghidra_dir / "ghidra_connect_debugger.py",
    )
    return compute_tree_digest(root, inputs), outputs


def _dir_size_bytes(path: Path) -> int:
    if path.is_file():
        return path.stat().st_size
    if not path.is_dir():
        return 0
    total = 0
    for dirpath_str, _, filenames in os.walk(path):
        dirpath = Path(dirpath_str)
        for fname in filenames:
            try:
                total += (dirpath / fname).stat().st_size
            except OSError:
                pass
    return total


def inspect_cache_entries(root: Path = ROOT) -> list[CacheEntryStat]:
    """Return size and existence metrics across Panadiga build caches."""
    candidates: tuple[tuple[str, Path], ...] = (
        ("Panadiga Step Cache", default_cache_dir(root)),
        ("Pico CMake Build", root / "target" / "pico-build"),
        ("Vite Cache", root / "apps" / "desktop" / "node_modules" / ".vite"),
        ("Frontend Dist", root / "apps" / "desktop" / "dist"),
        ("Cargo Debug Cache", root / "target" / "debug"),
        ("Cargo Release Cache", root / "target" / "release"),
        ("Cargo Android Cache", root / "target" / "android-cargo"),
    )
    return [
        CacheEntryStat(
            label=label,
            path=path,
            exists=path.exists(),
            bytes_total=_dir_size_bytes(path) if path.exists() else 0,
        )
        for label, path in candidates
    ]


def clear_caches(root: Path = ROOT, *, include_cargo: bool = False) -> list[Path]:
    """Remove Panadiga incremental step/build caches cleanly; optionally wipe target/ too."""
    targets = [
        default_cache_dir(root),
        root / "target" / "pico-build",
        root / "apps" / "desktop" / "node_modules" / ".vite",
        root / "apps" / "desktop" / "dist",
    ]
    if include_cargo:
        targets.extend(
            [
                root / "target" / "debug",
                root / "target" / "release",
                root / "target" / "android-cargo",
            ]
        )
    removed: list[Path] = []
    for t in targets:
        if t.exists():
            shutil.rmtree(t, ignore_errors=True)
            removed.append(t)
    return removed


def query_compiler_cache_stats(env: Mapping[str, str]) -> list[str]:
    """Collect optional sccache/ccache summary strings without failing when unavailable."""
    lines: list[str] = []
    for tool, args in (("sccache", ["--show-stats"]), ("ccache", ["--print-stats"])):
        exe = _resolve_tool(tool, env)
        if not exe:
            continue
        try:
            proc = subprocess.run(
                [exe, *args],
                env=dict(env),
                capture_output=True,
                text=True,
                timeout=3,
                check=False,
            )
            if proc.returncode == 0 and proc.stdout.strip():
                first_lines = " | ".join(l.strip() for l in proc.stdout.splitlines()[:4] if l.strip())
                lines.append(f"{tool}: {first_lines}")
        except (OSError, subprocess.TimeoutExpired):
            pass
    return lines
