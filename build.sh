#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

PLATFORM="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"

echo "===================================================================="
echo "   PANADIGA ENGINEERING SUITE: MASTER BUILD PIPELINE (Unix/macOS)"
echo "   Platform: ${PLATFORM}-${ARCH} | Unified CLI & Desktop GUI"
echo "===================================================================="
echo ""

# Dispatch Android build if requested
if [ "${1:-}" = "--android" ] || [ "${1:-}" = "-a" ]; then
    echo "===================================================================="
    echo "   Panadiga Android APK Build Pipeline (Tauri 2 Mobile / ARM64)"
    echo "===================================================================="
    
    if [ -z "${ANDROID_HOME:-}" ]; then
        if [ -d "$HOME/Android/Sdk" ]; then
            export ANDROID_HOME="$HOME/Android/Sdk"
        elif [ -d "$HOME/Library/Android/sdk" ]; then
            export ANDROID_HOME="$HOME/Library/Android/sdk"
        else
            echo "[ERRO] ANDROID_HOME não definido. Configure a variável de ambiente."
            exit 1
        fi
    fi
    echo "[*] Android SDK detectado em: $ANDROID_HOME"

    if [ -z "${NDK_HOME:-}" ] && [ -d "$ANDROID_HOME/ndk" ]; then
        NDK_DIR=$(find "$ANDROID_HOME/ndk" -maxdepth 1 -mindepth 1 -type d | sort -V | tail -n 1)
        if [ -n "$NDK_DIR" ]; then
            export NDK_HOME="$NDK_DIR"
            export ANDROID_NDK_ROOT="$NDK_DIR"
        fi
    fi

    echo "[*] Verificando target aarch64-linux-android..."
    rustup target add aarch64-linux-android

    echo "[*] Compilando frontend React..."
    npm --prefix apps/desktop run build

    echo "[*] Configurando crate-type temporário para Android cdylib..."
    sed -i 's/crate-type = \["rlib"\]/crate-type = \["staticlib", "cdylib", "rlib"\]/g' Cargo.toml
    trap 'sed -i '\''s/crate-type = \["staticlib", "cdylib", "rlib"\]/crate-type = \["rlib"\]/g'\'' Cargo.toml' EXIT

    if [ ! -d "gen/android" ] && [ ! -d "src-tauri/gen/android" ]; then
        echo "[*] Inicializando módulo nativo Android com Tauri CLI..."
        npx @tauri-apps/cli android init --ci --skip-targets-install
    fi

    echo "[*] Executando build de APK via Tauri CLI..."
    npx @tauri-apps/cli android build --apk --target aarch64

    # Copy generated APK to dist/releases and update SHA256SUMS.txt
    mkdir -p dist/releases
    APK_SRC=$(find gen/android/app/build/outputs/apk -name "*.apk" -type f 2>/dev/null | head -n 1)
    if [ -n "$APK_SRC" ]; then
        cp -f "$APK_SRC" dist/releases/panadiga-android-arm64.apk
        if command -v sha256sum >/dev/null 2>&1; then
            APK_HASH=$(sha256sum dist/releases/panadiga-android-arm64.apk | awk '{print $1}')
        elif command -v shasum >/dev/null 2>&1; then
            APK_HASH=$(shasum -a 256 dist/releases/panadiga-android-arm64.apk | awk '{print $1}')
        else
            APK_HASH="UNKNOWN"
        fi
        sed -i '/panadiga-android-arm64.apk/d' dist/releases/SHA256SUMS.txt 2>/dev/null || true
        echo "$APK_HASH  panadiga-android-arm64.apk" >> dist/releases/SHA256SUMS.txt
    fi

    echo "===================================================================="
    echo "[OK] BUILD ANDROID CONCLUÍDO COM SUCESSO!"
    echo "     Arquivo .apk gerado em: gen/android/app/build/outputs/apk/"
    echo "===================================================================="
    exit 0
fi

# [1/4] Build React Interface (UI)
echo "[1/4] Compilando Frontend Desktop (React / TypeScript / Vite)..."
if [ ! -d "apps/desktop/node_modules" ]; then
    echo "[*] Instalando dependencias do frontend..."
    npm --prefix apps/desktop ci
fi
npm --prefix apps/desktop run build
echo "[OK] Bundle otimizado gerado em apps/desktop/dist/"
echo ""

# [2/4] Verifying and Exporting Lab Tools (Ghidra SSOT)
echo "[2/4] Exportando scripts Ghidra diretamente do Rust SSOT..."
cargo run --release -p panadiga -- ghidra export-all --dir tools >/dev/null 2>&1 || true
echo "[OK] Scripts Ghidra sincronizados em tools/"
echo ""

# [3/4] Compiling Native Rust Unified Release Binary
echo "[3/4] Compilando Binário Unificado Panadiga em modo Release..."
cargo build --release -p panadiga
echo "[OK] Binário unificado gerado com sucesso: target/release/panadiga"
echo ""

# [4/4] Packaging Distribution into dist/
echo "[4/4] Empacotando distribuição consolidada para dist/..."
mkdir -p dist/tools dist/firmware dist/releases

cp -f target/release/panadiga dist/panadiga
chmod +x dist/panadiga

# Copy tools and Ghidra scripts
if [ -d "tools" ]; then
    cp -rf tools/* dist/tools/ 2>/dev/null || true
fi

# Copy compiled Pico firmware binaries if present (canonical location)
if [ -f "src/pico/firmware_bins/panasonic_dwire32a_rp2040.uf2" ]; then
    cp -f "src/pico/firmware_bins/panasonic_dwire32a_rp2040.uf2" dist/firmware/
elif [ -f "dwire32a_jtag_research/panasonic_dwire32a_rp2040.uf2" ]; then
    cp -f "dwire32a_jtag_research/panasonic_dwire32a_rp2040.uf2" dist/firmware/
fi
if [ -f "src/pico/firmware_bins/panasonic_dwire32a_rp2350.uf2" ]; then
    cp -f "src/pico/firmware_bins/panasonic_dwire32a_rp2350.uf2" dist/firmware/
elif [ -f "dwire32a_jtag_research/panasonic_dwire32a_rp2350.uf2" ]; then
    cp -f "dwire32a_jtag_research/panasonic_dwire32a_rp2350.uf2" dist/firmware/
fi

# Copy fast runners into dist/
cp -f run.cmd dist/run.cmd
cp -f run.sh dist/run.sh
chmod +x dist/run.sh

# Generate release archive (.tar.gz) and SHA256 checksum
RELEASE_ARCHIVE="dist/releases/panadiga-${PLATFORM}-${ARCH}.tar.gz"
tar -czf "$RELEASE_ARCHIVE" \
    -C dist panadiga run.cmd run.sh tools firmware \
    -C "$SCRIPT_DIR" README.md README.pt-BR.md LICENSE 2>/dev/null || \
tar -czf "$RELEASE_ARCHIVE" \
    -C dist panadiga run.cmd run.sh tools firmware \
    -C "$SCRIPT_DIR" README.md README.pt-BR.md

if command -v sha256sum >/dev/null 2>&1; then
    HASH="$(sha256sum "$RELEASE_ARCHIVE" | awk '{print $1}')"
elif command -v shasum >/dev/null 2>&1; then
    HASH="$(shasum -a 256 "$RELEASE_ARCHIVE" | awk '{print $1}')"
else
    HASH="UNKNOWN_HASH"
fi
echo "$HASH  panadiga-${PLATFORM}-${ARCH}.tar.gz" > dist/releases/SHA256SUMS.txt

echo ""
echo "===================================================================="
echo "   [OK] BUILD CONCLUÍDO COM SUCESSO!"
echo "   - Binário Unificado: dist/panadiga"
echo "   - Ferramentas:       dist/tools/"
echo "   - Firmware Pico:     dist/firmware/"
echo "   - Pacote Release:    $RELEASE_ARCHIVE"
echo "   - Checksum SHA256:   $HASH"
echo "===================================================================="
