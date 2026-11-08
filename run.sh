#!/usr/bin/env bash
# ==============================================================================
# Panadiga Engineering Suite - Fast Multiplatform Launcher & Runner
# Multiplatform: Linux (x86_64 / aarch64) & macOS (Darwin Intel / Apple Silicon)
# ==============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

DIST_BIN="$SCRIPT_DIR/dist/panadiga"
TARGET_BIN="$SCRIPT_DIR/target/release/panadiga"

# 1. Help Menu
if [ "${1:-}" = "--help" ] || [ "${1:-}" = "-h" ]; then
    echo "===================================================================="
    echo "Panadiga Suite - Inicializador Rápido (Linux & macOS)"
    echo "===================================================================="
    echo "Uso:"
    echo "  ./run.sh                 Inicia a interface Desktop nativa instantaneamente"
    echo "  ./run.sh --dev           Inicia em modo de desenvolvimento (Hot-Reload)"
    echo "  ./run.sh --build         Força compilação completa e inicia o app"
    echo "  ./run.sh --validate      Executa a suíte de 9 Quality Gates"
    echo "  ./run.sh --lint          Alias para --validate"
    echo "  ./run.sh --cli [args]    Executa a CLI nativa com argumentos"
    echo "  ./run.sh nvram clean ... Atalho direto para cirurgia de NVRAM (U99)"
    echo "  ./run.sh hdd inspect ... Atalho direto para inspeção forense de disco"
    echo "  ./run.sh ghidra ...      Atalho direto para gerador de scripts Ghidra"
    echo "  ./run.sh rag query ...   Atalho direto para consulta ao motor RAG Zero-Error"
    echo "  ./run.sh pico ...        Atalho direto para utilitários do RP2040 DWire32A"
    echo "===================================================================="
    exit 0
fi

# 2. Validation / Quality Gates
if [ "${1:-}" = "--validate" ] || [ "${1:-}" = "--lint" ]; then
    echo "[INFO] Executando suíte completa de 9 Quality Gates..."
    ./validate_all.sh
    exit $?
fi

# 3. Live Development Mode (Vite + Tauri Dev)
if [ "${1:-}" = "--dev" ]; then
    echo "[INFO] Iniciando ambiente de desenvolvimento Desktop (Vite + Tauri Dev)..."
    (cd apps/desktop && npm run dev)
    exit 0
fi

# 4. Force Build & Run
if [ "${1:-}" = "--build" ]; then
    echo "[INFO] Forçando compilação completa..."
    ./build.sh
    if [ -f "$DIST_BIN" ]; then
        "$DIST_BIN" &
    fi
    exit 0
fi

# 5. CLI Execution
if [ "${1:-}" = "--cli" ]; then
    shift
    if [ -f "$DIST_BIN" ]; then
        exec "$DIST_BIN" "$@"
    elif [ -f "$TARGET_BIN" ]; then
        exec "$TARGET_BIN" "$@"
    else
        cargo run -p panadiga -- "$@"
    fi
    exit $?
fi

# 6. Direct CLI subcommands
if [[ "${1:-}" =~ ^(nvram|hdd|frm|headers|scan|manuals|pico|ghidra|rag)$ ]]; then
    if [ -f "$DIST_BIN" ]; then
        exec "$DIST_BIN" "$@"
    elif [ -f "$TARGET_BIN" ]; then
        exec "$TARGET_BIN" "$@"
    else
        cargo run -p panadiga -- "$@"
    fi
    exit $?
fi

# 7. Default: Run Desktop GUI immediately
if [ -f "$DIST_BIN" ]; then
    echo "[INFO] Iniciando Panadiga Desktop diretamente de dist/..."
    "$DIST_BIN" &
    exit 0
fi

if [ -f "$TARGET_BIN" ]; then
    echo "[INFO] Iniciando Panadiga Desktop diretamente de target/release/..."
    "$TARGET_BIN" &
    exit 0
fi

echo "[AVISO] Executável unificado não encontrado."
echo "[INFO] Compilando ambiente nativo..."
./build.sh
if [ -f "$DIST_BIN" ]; then
    "$DIST_BIN" &
    exit 0
fi
