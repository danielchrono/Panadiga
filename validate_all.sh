#!/usr/bin/env bash
set -euo pipefail

echo "======================================================================"
echo "Panadiga Universal Code Quality & Commit Protection Gate"
echo "======================================================================"

echo ""
echo "[1/6] Validating Rust Workspace..."
cargo check --workspace --all-targets
cargo test --workspace

echo ""
echo "[2/6] Validating TypeScript Types..."
(cd apps/desktop && npm run typecheck)

echo ""
echo "[3/6] Executing Desktop Frontend Unit Tests (Vitest)...
(cd apps/desktop && npm test)

echo ""
echo "[4/6] Running Copy/Paste Detector (jscpd)..."
npx --prefix apps/desktop jscpd --config .jscpd.json

echo ""
echo "[5/6] Running UI/UX State Linter (uxlint)..."
(cd apps/desktop && npm run uxlint)

echo ""
echo "[6/6] Validating Ghidra Native Script Generator..."
cargo run -p panadiga -- ghidra export-all --dir tools

echo ""
echo "======================================================================"
echo "[SUCCESS] ALL COMMIT QUALITY GATES PASSED!"
echo "======================================================================"
