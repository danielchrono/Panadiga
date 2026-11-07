"""Codebase Audit Packager & Evaluator Bundle Generator.

Collects and packages the entire Panadiga codebase (Rust core, Python CLI,
Tauri/React frontend with strict i18n, and hardware guidelines) into a
self-contained, tool-free evaluation dossier for high-end reasoning models
(Argon, Claude 3.5/3.7, Opus 5.5).
"""

from __future__ import annotations

import argparse
import datetime
import os
import subprocess
from pathlib import Path
from typing import Dict, List, Tuple

BASE_DIR = Path(__file__).resolve().parent.parent

# Core files grouped by evaluation domain
FILES_TO_AUDIT: Dict[str, List[str]] = {
    "Domain 1: Rust Core & Hardware Reverse Engineering (panadiga)": [
        "Cargo.toml",
        "src/lib.rs",
        "src/main.rs",
        "src/constants.rs",
        "src/nvram.rs",
        "src/hdd.rs",
        "src/crypto.rs",
        "src/pico.rs",
        "src/headers.rs",
        "src/scan.rs",
        "src/manuals.rs",
        "src/ghidra.rs",
        "tests/panadiga_tests.rs",
    ],
    "Domain 2: Python Workbench Administration & Memory Map (Typer / Mypy)": [
        "tools/panadiga_cli.py",
        "tools/tests/test_panadiga_cli.py",
        "tools/ghidra_am34_memory_map.py",
        "tools/ghidra_importar_simbolos_panadiga.py",
        "tools/ghidra_conectar_debugger.py",
    ],
    "Domain 3: Desktop Frontend, Tauri IPC & i18n Strictness": [
        "apps/desktop/package.json",
        "apps/desktop/src/types/domain.ts",
        "apps/desktop/src/services/digaApi.ts",
        "apps/desktop/src/services/ipc.ts",
        "apps/desktop/src/i18n/translations.ts",
        "apps/desktop/src/i18n/useTranslation.ts",
        "apps/desktop/src/i18n/translations.test.ts",
        "apps/desktop/src/views/ManualsView.tsx",
        "apps/desktop/src/views/NvramView.tsx",
        "apps/desktop/src/views/PicoBenchView.tsx",
        "apps/desktop/src/views/HddStorageView.tsx",
        "apps/desktop/src/views/FrmCryptoView.tsx",
        "apps/desktop/src/views/DashboardView.tsx",
    ],
    "Domain 4: Zero-Error RAG Engine & Hardware Safety Shield": [
        "tools/rag_engine/config.py",
        "tools/rag_engine/models.py",
        "tools/rag_engine/guardrails.py",
        "tools/rag_engine/indexer.py",
        "tools/rag_engine/retriever.py",
        "tools/rag_engine/critic.py",
        "tools/rag_engine/safety_invariants.py",
        "tools/rag_engine/generator.py",
        "tools/rag_engine/pipeline.py",
        "tools/rag_engine/cli.py",
        "tools/rag_engine/tests/test_rag_zero_error.py",
    ],
    "Domain 5: Architectural Directives, Safety Invariants & Guidelines": [
        "docs/AI_GUIDELINES_AND_ZERO_ERROR_RAG.md",
        "docs/BLUEPRINT_ESQUEMA_ELETRICO_BANCADA.md",
        "docs/ARQUITETURA_STORAGE_MEIHDFS_PARTICOES.md",
    ],
    "Domain 6: Build Automation & Universal Quality Gates": [
        "validate_all.ps1",
        "build_all.ps1",
        ".github/workflows/ci.yml",
    ],
    "Domain 7: RP2040/RP2350 Pico Firmware & DWire32A PIO Engine (C & Assembly)": [
        "dwire32a_jtag_research/dwire32a.pio",
        "dwire32a_jtag_research/dwire32a_types.h",
        "dwire32a_jtag_research/dwire32a_hw.h",
        "dwire32a_jtag_research/dwire32a_hw.c",
        "dwire32a_jtag_research/dwire32a_pico_main.c",
        "dwire32a_jtag_research/gdb_rsp.h",
        "dwire32a_jtag_research/gdb_rsp.c",
        "dwire32a_jtag_research/ram_loader.c",
        "dwire32a_jtag_research/CMakeLists.txt",
    ],
}


def run_command_capture(cmd: str) -> str:
    """Runs a shell command and captures its standard output / error."""
    try:
        proc = subprocess.run(
            cmd,
            shell=True,
            cwd=str(BASE_DIR),
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
            timeout=120,
        )
        return proc.stdout.strip()
    except Exception as e:
        return f"[Command execution failed: {e}]"


def get_git_info() -> Tuple[str, str]:
    """Retrieves current git commit and branch."""
    commit = run_command_capture("git rev-parse --short HEAD")
    branch = run_command_capture("git rev-parse --abbrev-ref HEAD")
    return branch, commit


def count_file_stats(path: Path) -> Tuple[int, int]:
    """Returns line count and byte size of a file."""
    try:
        text = path.read_text(encoding="utf-8", errors="replace")
        lines = len(text.splitlines())
        return lines, len(text.encode("utf-8"))
    except Exception:
        return 0, 0


def generate_dossier(output_path: Path, run_checks: bool = True) -> None:
    now_str = datetime.datetime.now().strftime("%Y-%m-%d %H:%M:%S")
    branch, commit = get_git_info()

    doc_parts: List[str] = []

    # 1. System Prompt & Instruction Header for Argon / Opus 5.5
    doc_parts.append(f"""# CODEBASE MASTER AUDIT DOSSIER & EVALUATOR PACK
## Target Evaluators: Argon & Claude Opus 5.5 (Zero Tool-Call Self-Contained Review)
**Target Repository:** Panasonic DIGA DMR-EH55 Reverse Engineering, Forensics & Workbench Suite  
**Generated At:** {now_str}  
**Git Branch / Commit:** `{branch}` @ `{commit}`  

---

## 🚨 MANDATORY INSTRUCTIONS FOR AUDITING AI (ARGON / OPUS 5.5)

You are acting as an elite **Principal Embedded Systems Architect, Senior Rust Safety Auditor, and Full-Stack Quality Gate Inspector**.
You have been provided with the **complete, full-text source code** of all critical components of this repository in this single document.

> **CRITICAL DIRECTIVE ON TOOLS:**
> **DO NOT INVOKE ANY EXTERNAL TOOLS.**
> Every source file, configuration, test runner log, and architectural blueprint required for this audit has been pre-read and embedded below in full. Conduct your entire evaluation directly in your output response.

### Core Audit Questions & Evaluation Criteria:

1. **Silicon & Electrical Safety Invariants (3.3V Limits & Root-of-Trust):**
   - Does any routine or suggestion in `src/pico.rs`, `src/nvram.rs`, or `tools/panadiga_cli.py` violate the 3.3V nominal rail threshold (absolute max 3.35V)? Ensure the MN2DS0016 SoC and Spansion Flash are never exposed to 5V.
   - Does `src/nvram.rs` properly protect the IPL bootloader root-of-trust (`0x00000000..0x007EFFFF`, sectors `SA0..SA133`) against erasure during U99 repairs, restricting surgery strictly to NVRAM `SA134..SA141` (`0x007F0000..0x007FFFFF`)?
   - Is `CN55001` handled strictly as a half-duplex Matsushita DWire32A port (SDATA + SCLK) without any legacy 4-wire JTAG confusion?

2. **Rust Engineering & Memory Safety (`src/*.rs`):**
   - Evaluate memory safety, absence of undefined behavior, idiomatic error propagation (`anyhow` / `thiserror`), zero-copy slicing (`memmap2`), and big/little endian handling (`byteorder`).
   - Audit `src/hdd.rs` for MEIHDFS partition parsing, LBA 0 superblock validation (`"SH"` tag), mirror synchronization at LBA 57,344, and integer overflow safety on large 64-bit disk offsets.
   - Audit `src/crypto.rs` for optical container `PANA_DVD.FRM` ECB codebook extraction and entropy calculations.

3. **Frontend Architecture, Tauri IPC & Strict i18n:**
   - In `apps/desktop/src/`: Verify that **NO user-visible string** is hardcoded without passing through `useTranslation().t(...)`.
   - Verify compile-time type safety in `apps/desktop/src/types/domain.ts` and `apps/desktop/src/services/digaApi.ts`.
   - Verify that fallback strings and mock data preserve domain invariants.

4. **Zero-Error AI / RAG Guidelines & Invariants:**
   - Review `docs/AI_GUIDELINES_AND_ZERO_ERROR_RAG.md` and evaluate whether the 6-layer defense (QueryGuard, HybridRetriever, CRAG, Grounded Synthesis, FactCritic, Invariant Shield) provides formal mathematical guarantees against hallucinated offsets and dangerous voltages.

5. **RP2040/RP2350 C Firmware & PIO State Machine (`dwire32a_jtag_research/*`):**
   - Audit `dwire32a.pio` for half-duplex bus turnaround, setup/hold timing (250ns setup, 500ns hold at 1 MHz SCLK), and High-Z release at the end of TX.
   - Audit `dwire32a_hw.c` for open-drain reset emulation on GP5 (never driving 3.3V push-pull into reset supervisor), ADC0 VDD threshold verification (>= 2.5V), and Cold Break-In mode execution.
   - Audit `gdb_rsp.c` for buffer bounds safety, checksum calculation, shadow register synchronization, and memory windowing.

### Required Verdict Structure:
- **1. Executive Summary & Production Readiness Score (0-100%)**
- **2. Table of Findings by Severity**:
  - `[CRITICAL]`: Hardware destruction, flash bricking, panics, security bypasses.
  - `[MAJOR]`: Logic bugs, data corruption risk, missing error handling, i18n bypasses.
  - `[MINOR]`: Code style, dead code, redundant allocations.
  - `[RECOMMENDATION]`: Future-proofing, performance optimizations.
- **3. File-by-File Technical Deep Dive**:
  - Subsystem A: Rust Low-Level Engine (`src/*.rs`)
  - Subsystem B: Python Administration CLI (`tools/panadiga_cli.py`)
  - Subsystem C: Frontend & i18n Strictness (`apps/desktop/`)
  - Subsystem D: Hardware Safety & Invariant Guarantees (`tools/rag_engine/`)
  - Subsystem E: RP2040/RP2350 C Firmware & DWire32A PIO Engine (`dwire32a_jtag_research/*`)
- **4. Concrete Remediated Code Diffs (if issues are detected)**
- **5. Final Formal Certification Statement**

---
""")

    # 2. Automated Test Baseline Status
    doc_parts.append("## 1. Automated Test Baseline & System Verification\n\n")
    if run_checks:
        doc_parts.append("### 1.1 Rust Test Suite (`cargo test`)\n```text\n")
        cargo_out = run_command_capture("cargo test -- --nocapture")
        doc_parts.append(cargo_out + "\n```\n\n")

        doc_parts.append("### 1.2 Python CLI & Pytest Suites (`tools/tests` & `tools/rag_engine/tests`)\n```text\n")
        pytest_out = run_command_capture("python -m pytest tools/tests tools/rag_engine/tests -v")
        doc_parts.append(pytest_out + "\n```\n\n")

        doc_parts.append("### 1.3 Desktop Frontend Test Suite (`vitest`)\n```text\n")
        vitest_out = run_command_capture("npm test --prefix apps/desktop -- --run")
        doc_parts.append(vitest_out + "\n```\n\n")

        doc_parts.append("### 1.4 Adversarial Safety Benchmark (`tools.rag_engine.cli benchmark`)\n```text\n")
        benchmark_out = run_command_capture("python -m tools.rag_engine.cli benchmark")
        doc_parts.append(benchmark_out + "\n```\n\n")
    else:
        doc_parts.append("*[Automated test execution skipped per user flag]*\n\n")

    # 3. File Inventory and Statistics
    doc_parts.append("## 2. Codebase Inventory & Statistics\n\n")
    total_files = 0
    total_lines = 0
    total_bytes = 0

    doc_parts.append("| Category | File Path | Lines | Size (KB) |\n| :--- | :--- | :--- | :--- |\n")
    for category, files in FILES_TO_AUDIT.items():
        for rel_path in files:
            p = BASE_DIR / rel_path
            if p.exists():
                l, b = count_file_stats(p)
                total_files += 1
                total_lines += l
                total_bytes += b
                doc_parts.append(f"| {category.split(':')[0]} | `{rel_path}` | {l:,} | {b/1024:.1f} |\n")
            else:
                doc_parts.append(f"| {category.split(':')[0]} | `{rel_path}` (Missing) | - | - |\n")

    doc_parts.append(f"\n**Total Audited Files:** {total_files} | **Total Lines:** {total_lines:,} | **Total Volume:** {total_bytes/1024:.1f} KB\n\n---\n\n")

    # 4. Full Source Code Listings
    doc_parts.append("## 3. Full Source Code Listings for Direct AI Audit\n\n")

    for category, files in FILES_TO_AUDIT.items():
        doc_parts.append(f"### {category}\n\n")
        for rel_path in files:
            p = BASE_DIR / rel_path
            if not p.exists():
                doc_parts.append(f"> [!WARNING] File not found: `{rel_path}`\n\n")
                continue

            lines, b = count_file_stats(p)
            ext = p.suffix.lstrip(".")
            lang = "rust" if ext == "rs" else "python" if ext == "py" else "typescript" if ext in ("ts", "tsx") else "toml" if ext == "toml" else "json" if ext == "json" else "powershell" if ext == "ps1" else "yaml" if ext in ("yml", "yaml") else "markdown"

            content = p.read_text(encoding="utf-8", errors="replace")

            doc_parts.append(f"#### File: `{rel_path}` ({lines:,} lines, {b/1024:.1f} KB)\n")
            doc_parts.append(f"```{lang}\n{content}\n```\n\n---\n\n")

    # 5. Reviewer Questionnaire & Deliverable Formats
    doc_parts.append("""## 4. Required Reviewer Output Format for Argon & Opus 5.5

When rendering your audit verdict, please structure your report using the following markdown sections:

1. **Executive Verdict & Readiness Score (0-100%):** Summary of codebase maturity, safety posture, and stability.
2. **Detailed Findings by Severity:**
   - For each finding, provide:
     - `Severity`: [CRITICAL | MAJOR | MINOR | RECOMMENDATION]
     - `File & Line Range`: `path/to/file.rs#L123-L145`
     - `Vulnerability / Issue Description`: Technical explanation.
     - `Proof-of-Concept / Hazard Scenario`: Why this fails in real hardware/runtime.
     - `Recommended Patch / Remediated Code`: Drop-in replacement code.
3. **Subsystem-by-Subsystem Deep Dive:**
   - Subsystem A: Rust Low-Level Engine (`src/*.rs`)
   - Subsystem B: Python Administration CLI (`tools/panadiga_cli.py`)
   - Subsystem C: Frontend & i18n Strictness (`apps/desktop/`)
   - Subsystem D: Hardware Safety & Invariant Guarantees
4. **Final Architectural Assessment & Certification:** Formal statement on whether the software is safe for deployment on physical Panasonic DMR-EH55 bench setups.
""")

    output_path.write_text("".join(doc_parts), encoding="utf-8")
    print(f"[✓] Audit dossier successfully written to: {output_path.resolve()}")
    print(f"    Total files bundled: {total_files}")
    print(f"    Total lines of code: {total_lines:,}")
    print(f"    Dossier size: {output_path.stat().st_size / 1024:.1f} KB")


def main():
    parser = argparse.ArgumentParser(description="Generate full codebase audit dossier for Argon & Opus 5.5")
    parser.add_argument(
        "-o", "--output",
        default="AUDIT_DOSSIER_FOR_ARGON_AND_OPUS.md",
        help="Target output markdown path"
    )
    parser.add_argument(
        "--no-checks",
        action="store_true",
        help="Skip running cargo test, pytest, and npm test"
    )
    args = parser.parse_args()

    out_file = BASE_DIR / args.output
    generate_dossier(out_file, run_checks=not args.no_checks)


if __name__ == "__main__":
    main()
