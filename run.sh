#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

DESKTOP_BIN="$SCRIPT_DIR/dist_bin/panadiga-desktop"
CLI_BIN="$SCRIPT_DIR/dist_bin/panadiga-cli"

if [ "${1:-}" == "--dev" ]; then
    echo "[INFO] Starting desktop dev environment..."
    (cd apps/desktop && npm run tauri dev)
    exit 0
fi

if [ "${1:-}" == "--cli" ]; then
    shift
    if [ -f "$CLI_BIN" ]; then
        "$CLI_BIN" "$@"
    else
        cargo run -- "$@"
    fi
    exit 0
fi

if [ -f "$DESKTOP_BIN" ]; then
    echo "[INFO] Launching Panadiga Desktop..."
    "$DESKTOP_BIN" &
    exit 0
fi

echo "[INFO] Prebuilt binary not found. Running cargo run..."
cargo run -p panasonic-diga-desktop
