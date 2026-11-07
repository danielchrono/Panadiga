#!/usr/bin/env bash
# ==============================================================================
# Panadiga Engineering Suite - Fast Multiplatform Launcher & Runner
# Multiplatform: Linux (x86_64 / aarch64) & macOS (Darwin Intel / Apple Silicon)
# ==============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

DESKTOP_BIN="$SCRIPT_DIR/dist_bin/panadiga-desktop"
CLI_BIN="$SCRIPT_DIR/dist_bin/panadiga-cli"

# 1. Help Menu
if [ "${1:-}" == "--help" ] || [ "${1:-}" == "-h" ]; then
    echo "===================================================================="
    echo "Panadiga Suite - Inicializador Rápido (Linux & macOS)"
    echo "===================================================================="
    echo "Uso:"
    echo "  ./run.sh                 Inicia a interface Desktop nativa instantaneamente"
    echo "  ./run.sh --dev           Inicia em modo de desenvolvimento (Hot-Reload)"
    echo "  ./run.sh --build         Força compilação completa e inicia o app"
    echo "  ./run.sh --cli [args]    Executa a CLI nativa com argumentos"
    echo "  ./run.sh rag query ...   Atalho direto para consulta ao motor RAG Zero-Error"
    echo "  ./run.sh nvram clean ... Atalho direto para cirurgia de NVRAM (U99)"
    echo "  ./run.sh hdd inspect ... Atalho direto para inspeção forense de disco"
    echo "  ./run.sh ghidra ...      Atalho direto para gerador de scripts Ghidra"
    echo "  ./run.sh pico ...        Atalho direto para utilitários do RP2040 DWire32A"
    echo "===================================================================="
    exit 0
fi

# 2. Live Development Mode (Vite + Tauri Dev)
if [ "${1:-}" == "--dev" ]; then
    echo "[INFO] Iniciando ambiente de desenvolvimento Desktop (Vite + Tauri Dev)..."
    (cd apps/desktop && npm run tauri dev)
    exit 0
fi

# 3. Force Build & Run
if [ "${1:-}" == "--build" ]; then
    echo "[INFO] Forçando compilação completa com paralelismo acelerado..."
    ./build_all.sh
    if [ -f "$DESKTOP_BIN" ]; then
        echo "[INFO] Iniciando Panadiga Desktop..."
        "$DESKTOP_BIN" &
    fi
    exit 0
fi

# 4. Direct CLI execution or Subcommand Shortcuts
CLI_SUBCOMMANDS="nvram hdd frm headers scan manuals pico ghidra rag"
FIRST_ARG="${1:-}"

if [ "$FIRST_ARG" == "--cli" ]; then
    shift
    if [ -f "$CLI_BIN" ]; then
        exec "$CLI_BIN" "$@"
    else
        exec cargo run -- "$@"
    fi
fi

for subcmd in $CLI_SUBCOMMANDS; do
    if [ "$FIRST_ARG" == "$subcmd" ]; then
        if [ -f "$CLI_BIN" ]; then
            exec "$CLI_BIN" "$@"
        else
            exec cargo run -- "$@"
        fi
    fi
done

# 5. Default: Launch Desktop GUI
if [ -f "$DESKTOP_BIN" ]; then
    echo "[INFO] Iniciando Panadiga Desktop instantaneamente..."
    OS="$(uname -s)"
    if [ "$OS" == "Darwin" ]; then
        # On macOS, check if .app bundle exists, otherwise launch binary
        if [ -d "$SCRIPT_DIR/dist_bin/panadiga-desktop.app" ]; then
            open "$SCRIPT_DIR/dist_bin/panadiga-desktop.app"
        else
            "$DESKTOP_BIN" &
        fi
    else
        "$DESKTOP_BIN" &
    fi
    exit 0
fi

# If binary not found, auto-build and run
echo "[AVISO] Binário Desktop não encontrado em dist_bin/. Compilando agora..."
./build_all.sh
if [ -f "$DESKTOP_BIN" ]; then
    echo "[INFO] Iniciando Panadiga Desktop..."
    "$DESKTOP_BIN" &
    exit 0
else
    echo "[ERRO] Falha ao localizar executável após o build!"
    exit 1
fi
