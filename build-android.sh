#!/usr/bin/env bash
set -euo pipefail

echo "==============================================================================="
echo " Panasonic DMR-EH55 DIGA Suite - Android APK Automated Build System"
echo " Tauri 2.0 Mobile / Rust cargo-ndk / AArch64 (ARM64)"
echo "==============================================================================="

# 1. Validate Android SDK
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

# 2. Validate Android NDK
if [ -z "${NDK_HOME:-}" ]; then
    if [ -d "$ANDROID_HOME/ndk" ]; then
        # Pick latest installed NDK
        NDK_DIR=$(find "$ANDROID_HOME/ndk" -maxdepth 1 -mindepth 1 -type d | sort -V | tail -n 1)
        if [ -n "$NDK_DIR" ]; then
            export NDK_HOME="$NDK_DIR"
            export ANDROID_NDK_ROOT="$NDK_DIR"
        fi
    fi
fi

if [ -n "${NDK_HOME:-}" ]; then
    echo "[*] Android NDK detectado em: $NDK_HOME"
fi

# 3. Ensure Rust Target is Installed
echo "[*] Verificando target aarch64-linux-android..."
rustup target add aarch64-linux-android

# 4. Build React / Vite Frontend
echo "[*] Compilando frontend React (apps/desktop)..."
npm --prefix apps/desktop run build

# 5. Temporarily configure crate-type for Android shared library (.so)
echo "[*] Configurando crate-type para Android cdylib..."
sed -i 's/crate-type = \["rlib"\]/crate-type = \["staticlib", "cdylib", "rlib"\]/g' Cargo.toml
trap 'sed -i '\''s/crate-type = \["staticlib", "cdylib", "rlib"\]/crate-type = \["rlib"\]/g'\'' Cargo.toml' EXIT

# 6. Initialize Android if not initialized
if [ ! -d "gen/android" ] && [ ! -d "src-tauri/gen/android" ]; then
    echo "[*] Inicializando módulo nativo Android com Tauri CLI..."
    npx @tauri-apps/cli android init --ci --skip-targets-install
fi

# 7. Build Android APK
echo "[*] Executando build de APK via Tauri CLI..."
npx @tauri-apps/cli android build --apk --target aarch64-linux-android

echo "==============================================================================="
echo " BUILD ANDROID CONCLUÍDO COM SUCESSO!"
echo " O arquivo .apk foi gerado em: gen/android/app/build/outputs/apk/"
echo "==============================================================================="
