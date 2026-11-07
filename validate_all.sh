#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "======================================================================"
echo "Panadiga Universal Code Quality & Commit Protection Gate (8 Gates)"
echo "======================================================================"

echo ""
echo "[1/8] Validating Rust Workspace..."
cargo check --workspace --all-targets
cargo test --workspace

echo ""
echo "[2/8] Validating TypeScript Types..."
(cd apps/desktop && npm run typecheck)

echo ""
echo "[3/8] Executing Desktop Frontend Unit Tests (Vitest)..."
(cd apps/desktop && npm test)

echo ""
echo "[4/8] Running Copy/Paste Detector (jscpd)..."
npx --prefix apps/desktop jscpd --config .jscpd.json

echo ""
echo "[5/8] Running UI/UX State Linter (uxlint)..."
(cd apps/desktop && npm run uxlint)

echo ""
echo "[6/8] Running Playwright End-to-End Tests (E2E)..."
(cd apps/desktop && npm run test:e2e)

echo ""
echo "[7/8] Validating Ghidra Native Generator (Rust SSOT)..."
cargo run -- ghidra export-all --dir tools

echo ""
echo "[8/8] Verifying Python Typer CLI, Mypy & Pytest Suite..."
python3 -m py_compile tools/ghidra_am34_memory_map.py tools/ghidra_importar_simbolos_panadiga.py tools/ghidra_conectar_debugger.py
python3 -m mypy tools/panadiga_cli.py
python3 -m pytest tools/tests tools/rag_engine/tests

echo ""
echo "======================================================================"
echo "[SUCCESS] ALL 8 COMMIT QUALITY GATES PASSED!"
echo "======================================================================"
