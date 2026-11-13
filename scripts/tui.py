"""Cross-platform zero-dependency interactive TUI for Panadiga build, CI & cache orchestration."""

from __future__ import annotations

from dataclasses import dataclass
import os
from pathlib import Path
import sys
from typing import Any, Callable, Mapping

from scripts.panadiga_cache import (
    WorkspaceInventory,
    clear_caches,
    frontend_digest_and_outputs,
    ghidra_digest_and_outputs,
    inspect_cache_entries,
    is_step_fresh,
    pico_digest_and_outputs,
    query_compiler_cache_stats,
)


@dataclass
class PipelineSelection:
    """User-selected targets and options for DAG execution."""

    ci: bool = False
    frontend: bool = True
    pico: bool = False
    desktop: bool = True
    android: bool = False
    use_cache: bool = True
    run_after: bool = False


@dataclass(frozen=True)
class DagWave:
    """Single dependency-ordered execution stage displayed before running."""

    title: str
    concurrency: str
    steps: tuple[str, ...]


def default_all_selection(inventory: WorkspaceInventory, *, include_ci: bool = True) -> PipelineSelection:
    """Construct full build selection respecting discovered cross-platform toolchains."""
    has_pico = all(k in inventory.tools for k in ("arm-none-eabi-gcc", "pico_sdk", "cmake"))
    has_android = all(k in inventory.tools for k in ("android_sdk", "android_ndk"))
    return PipelineSelection(
        ci=include_ci,
        frontend=True,
        pico=has_pico,
        desktop=True,
        android=has_android,
        use_cache=True,
        run_after=False,
    )


def _format_bytes(num_bytes: int) -> str:
    value = float(num_bytes)
    for unit in ("B", "KiB", "MiB", "GiB"):
        if value < 1024.0 or unit == "GiB":
            return f"{int(value)} {unit}" if unit == "B" else f"{value:.1f} {unit}"
        value /= 1024.0
    return f"{num_bytes} B"


def _step_cache_tag(
    step: str,
    selection: PipelineSelection,
    inventory: WorkspaceInventory,
    env: Mapping[str, str],
) -> str:
    if not selection.use_cache:
        return "[CACHE OFF]"
    if step == "frontend":
        digest, outs = frontend_digest_and_outputs(inventory.root)
        return "[CACHE HIT]" if is_step_fresh(inventory.root, "frontend", digest, outs) else "[BUILD]"
    if step == "pico":
        digest, outs = pico_digest_and_outputs(inventory.root, env)
        return "[CACHE HIT]" if is_step_fresh(inventory.root, "pico", digest, outs) else "[BUILD]"
    if step == "ghidra":
        digest, outs = ghidra_digest_and_outputs(inventory.root)
        return "[CACHE HIT]" if is_step_fresh(inventory.root, "ghidra", digest, outs) else "[EXPORT]"
    return "[INCREMENTAL]"


def build_execution_plan(
    selection: PipelineSelection,
    inventory: WorkspaceInventory,
    env: Mapping[str, str],
) -> list[DagWave]:
    """Compute dependency-ordered parallel/serial DAG waves for display and execution."""
    waves: list[DagWave] = []
    need_frontend = selection.frontend or selection.desktop or selection.android

    wave1_steps: list[str] = []
    if need_frontend:
        wave1_steps.append(f"Frontend Vite UI {_step_cache_tag('frontend', selection, inventory, env)}")
    if selection.pico:
        wave1_steps.append(
            f"Pico RP2040 + RP2350 (CMake/Ninja 2x paralelo) {_step_cache_tag('pico', selection, inventory, env)}"
        )
    if selection.android:
        wave1_steps.append("Android Prep (rustup target + local.properties)")
    if wave1_steps:
        mode = "PARALELO" if len(wave1_steps) > 1 or selection.pico else "DIRETO"
        waves.append(DagWave("Wave 1 - Preparo & Frontends Independentes", mode, tuple(wave1_steps)))

    if selection.pico:
        waves.append(
            DagWave(
                "Wave 2 - Barreira Serial Pico UF2",
                "SERIAL",
                ("Merge Unified UF2 + zero-touch sync (firmware_bins/)",),
            )
        )

    if selection.ci:
        waves.append(
            DagWave(
                "Wave 3 - Pipeline CI (9 Quality Gates)",
                "PARALELO (9 lanes + ProcessTracker)",
                ("Rust/Ghidra + TS + Vitest + jscpd + uxlint + Playwright + C11 + Mypy/Pytest",),
            )
        )

    if selection.desktop and selection.android:
        ghidra_tag = _step_cache_tag("ghidra", selection, inventory, env)
        waves.append(
            DagWave(
                "Wave 4 - Desktop Release + Android ARM64 APK",
                "PARALELO (target/ || target/android-cargo/)",
                (
                    f"Desktop: cargo build --release -p panadiga-cli -p panadiga-desktop + Ghidra {ghidra_tag}",
                    "Android: cargo aarch64 + Gradle APK + zipalign + apksigner",
                ),
            )
        )
    elif selection.desktop:
        ghidra_tag = _step_cache_tag("ghidra", selection, inventory, env)
        waves.append(
            DagWave(
                "Wave 4 - Desktop Release Packaging",
                "PARALELO (1 DAG Cargo unificado)",
                (
                    "cargo build --release -p panadiga-cli -p panadiga-desktop",
                    f"Ghidra SSOT {ghidra_tag} + Staging dist/ + Archive + SHA256SUMS.txt",
                ),
            )
        )
    elif selection.android:
        waves.append(
            DagWave(
                "Wave 4 - Android ARM64 APK",
                "DIRETO (target/android-cargo/)",
                ("tauri android build --apk --target aarch64 + zipalign + apksigner",),
            )
        )

    if selection.run_after and selection.desktop:
        waves.append(DagWave("Wave 6 - Launch", "DETACHED", ("Iniciar Panadiga Desktop GUI",)))

    return waves


def _enable_ansi_console() -> None:
    if hasattr(sys.stdout, "reconfigure"):
        try:
            sys.stdout.reconfigure(encoding="utf-8", errors="replace")
        except OSError:
            pass
    if sys.platform != "win32":
        return
    try:
        import ctypes

        kernel32 = ctypes.windll.kernel32
        handle = kernel32.GetStdHandle(-11)
        mode = ctypes.c_uint32()
        if kernel32.GetConsoleMode(handle, ctypes.byref(mode)):
            kernel32.SetConsoleMode(handle, mode.value | 0x0004)
    except Exception:
        pass


def _read_keypress() -> str:
    """Read normalized navigation key cross-platform without third-party dependencies."""
    if sys.platform == "win32":
        import msvcrt

        getwch: Callable[[], str] = getattr(msvcrt, "getwch")
        ch = getwch()
        if ch in ("\x00", "\xe0"):
            ext = getwch()
            return {"H": "UP", "P": "DOWN", "K": "LEFT", "M": "RIGHT"}.get(ext, "")
        if ch in ("\r", "\n"):
            return "ENTER"
        if ch == " ":
            return "SPACE"
        if ch == "\x03":
            raise KeyboardInterrupt
        if ch == "\x1b":
            return "ESC"
        return ch.lower()

    import termios
    import tty

    fd = sys.stdin.fileno()
    old = termios.tcgetattr(fd)
    try:
        tty.setraw(fd)
        ch = sys.stdin.read(1)
        if ch == "\x03":
            raise KeyboardInterrupt
        if ch == "\x1b":
            next_ch = sys.stdin.read(1) if sys.stdin in _select_readable(fd, 0.04) else ""
            if next_ch == "[":
                code = sys.stdin.read(1)
                return {"A": "UP", "B": "DOWN", "C": "RIGHT", "D": "LEFT"}.get(code, "")
            return "ESC"
        if ch in ("\r", "\n"):
            return "ENTER"
        if ch == " ":
            return "SPACE"
        return ch.lower()
    finally:
        termios.tcsetattr(fd, termios.TCSADRAIN, old)


def _select_readable(fd: int, timeout_s: float) -> list[Any]:
    import select

    readable, _, _ = select.select([sys.stdin], [], [], timeout_s)
    return readable


def render_inventory_and_cache_report(root: Path, inventory: WorkspaceInventory, env: Mapping[str, str]) -> str:
    """Format recursive workspace discovery and cache metrics into readable lines."""
    lines = [
        "=" * 74,
        f"  PANADIGA RECURSIVE WORKSPACE & CACHE ({inventory.os_name} {inventory.arch} | {inventory.cpu_count} CPUs)",
        f"  Root: {root}",
        "=" * 74,
        f"  Crates ({len(inventory.crates)}): {', '.join(inventory.crates) or '-'}",
        f"  Apps   ({len(inventory.apps)}): {', '.join(inventory.apps) or '-'}",
        f"  FW     ({len(inventory.firmware_targets)}): {', '.join(inventory.firmware_targets) or '-'}",
        "-" * 74,
        "  Toolchains & Aceleradores:",
    ]
    for key in (
        "cargo",
        "rustc",
        "node",
        "npm",
        "arm-none-eabi-gcc",
        "pico_sdk",
        "cmake",
        "ninja",
        "c11_host",
        "android_sdk",
        "android_ndk",
        "sccache",
        "ccache",
    ):
        status = inventory.tools.get(key, "N/D")
        lines.append(f"    - {key:<18}: {status}")

    lines.append("-" * 74)
    lines.append("  Caches Locais:")
    for entry in inspect_cache_entries(root):
        state = _format_bytes(entry.bytes_total) if entry.exists else "vazio"
        lines.append(f"    - {entry.label:<22}: {state:>10}  ({entry.path})")

    cc_stats = query_compiler_cache_stats(env)
    if cc_stats:
        lines.append("-" * 74)
        for st in cc_stats:
            lines.append(f"  {st}")
    lines.append("=" * 74)
    return "\n".join(lines)


def _toggle_row(sel: PipelineSelection, row_idx: int) -> None:
    attrs = ("ci", "frontend", "pico", "desktop", "android", "use_cache", "run_after")
    if 0 <= row_idx < len(attrs):
        name = attrs[row_idx]
        setattr(sel, name, not getattr(sel, name))


def _render_tui_screen(
    sel: PipelineSelection,
    cursor: int,
    inventory: WorkspaceInventory,
    env: Mapping[str, str],
    notice: str,
) -> None:
    has_pico = all(k in inventory.tools for k in ("arm-none-eabi-gcc", "pico_sdk", "cmake"))
    has_android = all(k in inventory.tools for k in ("android_sdk", "android_ndk"))
    accel = []
    if "sccache" in inventory.tools:
        accel.append("sccache")
    if "ccache" in inventory.tools:
        accel.append("ccache")
    accel_str = ", ".join(accel) if accel else "hash steps.json + Cargo/Vite/Gradle"

    rows: tuple[tuple[str, bool, str], ...] = (
        ("Pipeline CI (9 Quality Gates em paralelo)", sel.ci, ""),
        ("Frontend Desktop UI (React 19 + Vite)", sel.frontend or sel.desktop or sel.android, ""),
        ("Pico Firmware (RP2040 + RP2350 + Unified UF2)", sel.pico, "" if has_pico else " [ARM GCC/SDK ausente]"),
        ("Desktop Release (panadiga CLI + Tauri GUI + dist/)", sel.desktop, ""),
        ("Android ARM64 APK (aarch64-linux-android)", sel.android, "" if has_android else " [SDK/NDK ausente]"),
        ("Usar Cache de Build (hash + sccache/ccache)", sel.use_cache, ""),
        ("Abrir Desktop GUI ao finalizar", sel.run_after, ""),
    )

    waves = build_execution_plan(sel, inventory, env)
    sys.stdout.write("\033[H\033[J")
    print("=" * 78)
    print(f"  PANADIGA INTERACTIVE TUI BUILD & CI  |  {inventory.os_name} {inventory.arch} ({inventory.cpu_count} cores)")
    print(f"  Root: {inventory.root}")
    print(f"  Cache/Accel: {accel_str}")
    print("=" * 78)
    print("  Selecione os alvos de execucao:")
    for idx, (label, checked, suffix) in enumerate(rows):
        pointer = ">" if idx == cursor else " "
        mark = "x" if checked else " "
        num = idx + 1
        print(f"   {pointer} [{mark}] {num}. {label}{suffix}")

    print("-" * 78)
    print("  Plano DAG (Paralelo quando possivel, Serial quando necessario):")
    if not waves:
        print("    (Nenhum alvo marcado)")
    else:
        for w in waves:
            print(f"    * {w.title}  [{w.concurrency}]")
            for step in w.steps:
                print(f"        -> {step}")

    print("-" * 78)
    print("  Atalhos: Up/Down mover | Espaco/1-7 marcar | [a] Tudo+CI | [b] Build Tudo")
    print("           [d] Desktop  | [v] So CI | [c] Caches | [Enter] Executar | [q] Sair")
    if notice:
        print(f"  [!] {notice}")
    sys.stdout.flush()


def _handle_cache_submenu(inventory: WorkspaceInventory, env: Mapping[str, str]) -> str:
    sys.stdout.write("\033[H\033[J")
    print(render_inventory_and_cache_report(inventory.root, inventory, env))
    print("  [l] Limpar cache incremental Panadiga   [x] Limpar TUDO (+ Cargo target)")
    print("  [qualquer outra tecla] Voltar ao menu principal")
    sys.stdout.flush()
    key = _read_keypress()
    if key == "l":
        removed = clear_caches(inventory.root, include_cargo=False)
        return f"Limpo cache Panadiga ({len(removed)} diretorios)."
    if key == "x":
        removed = clear_caches(inventory.root, include_cargo=True)
        return f"Limpo cache completo incluindo Cargo target ({len(removed)} diretorios)."
    return ""


def _fallback_numeric_prompt(inventory: WorkspaceInventory, env: Mapping[str, str]) -> PipelineSelection | None:
    print(render_inventory_and_cache_report(inventory.root, inventory, env))
    print("Menu rapido:")
    print("  1) Pipeline Completo (CI + Pico* + Frontend + Desktop + Android*)")
    print("  2) Build Tudo (sem CI)")
    print("  3) Desktop Release (Frontend + CLI + Desktop)")
    print("  4) Somente CI (9 Quality Gates em paralelo)")
    print("  5) Somente Pico Firmware")
    print("  6) Limpar Cache Panadiga")
    print("  q) Sair")
    choice = input("Escolha [1-6/q]: ").strip().lower()
    if choice == "1":
        return default_all_selection(inventory, include_ci=True)
    if choice == "2":
        return default_all_selection(inventory, include_ci=False)
    if choice == "3":
        return PipelineSelection(frontend=True, desktop=True)
    if choice == "4":
        return PipelineSelection(ci=True, frontend=False, desktop=False)
    if choice == "5":
        return PipelineSelection(frontend=False, pico=True, desktop=False)
    if choice == "6":
        clear_caches(inventory.root, include_cargo=False)
        print("[OK] Caches Panadiga removidos.")
        return None
    return None


def launch_tui(inventory: WorkspaceInventory, env: Mapping[str, str]) -> PipelineSelection | None:
    """Run interactive terminal selector returning chosen targets, or None on exit."""
    _enable_ansi_console()
    if not (sys.stdin.isatty() and sys.stdout.isatty()):
        return _fallback_numeric_prompt(inventory, env)

    sel = default_all_selection(inventory, include_ci=True)
    cursor = 0
    notice = ""
    row_count = 7

    while True:
        _render_tui_screen(sel, cursor, inventory, env, notice)
        notice = ""
        try:
            key = _read_keypress()
        except (OSError, ImportError):
            return _fallback_numeric_prompt(inventory, env)

        if key in ("UP", "k"):
            cursor = (cursor - 1) % row_count
        elif key in ("DOWN", "j"):
            cursor = (cursor + 1) % row_count
        elif key == "SPACE":
            _toggle_row(sel, cursor)
        elif key in ("1", "2", "3", "4", "5", "6", "7"):
            idx = int(key) - 1
            cursor = idx
            _toggle_row(sel, idx)
        elif key == "a":
            sel = default_all_selection(inventory, include_ci=True)
            notice = "Preset selecionado: Pipeline Completo (CI + Build Tudo)."
        elif key == "b":
            sel = default_all_selection(inventory, include_ci=False)
            notice = "Preset selecionado: Build Tudo (sem CI)."
        elif key == "d":
            sel = PipelineSelection(ci=False, frontend=True, desktop=True, use_cache=sel.use_cache)
            notice = "Preset selecionado: Desktop Release Rapido."
        elif key == "v":
            sel = PipelineSelection(ci=True, frontend=False, desktop=False, use_cache=sel.use_cache)
            notice = "Preset selecionado: Somente Quality Gates CI."
        elif key == "c":
            notice = _handle_cache_submenu(inventory, env)
        elif key in ("q", "ESC"):
            print("\nCancelado pelo usuario.")
            return None
        elif key == "ENTER":
            if not (sel.ci or sel.frontend or sel.pico or sel.desktop or sel.android):
                notice = "Marque ao menos um alvo antes de executar!"
                continue
            print()
            return sel
        else:
            if key == "" and os.name == "nt":
                continue
