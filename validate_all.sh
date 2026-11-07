#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "======================================================================"
echo "Panadiga Universal Code Quality & Commit Protection Gate (9 Gates)"
echo "======================================================================"

echo ""
echo "[1/9] Validating Rust Workspace..."
cargo check --workspace --all-targets
cargo test --workspace

echo ""
echo "[2/9] Validating TypeScript Types..."
(cd apps/desktop && npm run typecheck)

echo ""
echo "[3/9] Executing Desktop Frontend Unit Tests (Vitest)..."
(cd apps/desktop && npm test)

echo ""
echo "[4/9] Running Copy/Paste Detector (jscpd)..."
npx --prefix apps/desktop jscpd --config .jscpd.json

echo ""
echo "[5/9] Running UI/UX State Linter (uxlint)..."
(cd apps/desktop && npm run uxlint)

echo ""
echo "[6/9] Running Playwright End-to-End Tests (E2E)..."
(cd apps/desktop && npm run test:e2e)

echo ""
echo "[7/9] Validating Ghidra Native Generator (Rust SSOT)..."
cargo run -- ghidra export-all --dir tools

echo ""
echo "[8/9] Compiling & Executing C11 Pico Firmware Host Tests..."
gcc -std=c11 -Wall -Wextra -Werror -DPICO_HOST_TEST=1 -I dwire32a_jtag_research dwire32a_jtag_research/gdb_rsp.c dwire32a_jtag_research/tests/test_gdb_rsp_host.c -o test_gdb_rsp_host
./test_gdb_rsp_host
rm -f test_gdb_rsp_host

echo ""
echo "[9/9] Verifying Python Typer CLI, Mypy Strict & Pytest Suite..."
python3 -m py_compile tools/ghidra_am34_memory_map.py tools/ghidra_import_symbols_panadiga.py tools/ghidra_connect_debugger.py
python3 -m mypy --strict tools/panadiga_cli.py tools/rag_engine
python3 -m pytest tools/tests tools/rag_engine/tests

echo ""
echo "======================================================================"
echo "[SUCCESS] ALL 9 COMMIT QUALITY GATES PASSED!"
echo "======================================================================"
