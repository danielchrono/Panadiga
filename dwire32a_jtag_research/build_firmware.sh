#!/usr/bin/env bash
# ==============================================================================
# Panasonic DMR-EH55 DIGA Engineering Suite
# Subsystem: DWire32A / OCD Interface Engine - Cross-Platform Firmware Builder
# Supported Targets: Raspberry Pi Pico (RP2040) / Pico 2 (RP2350 ARM & RISC-V)
# Operating Systems: Linux, macOS, BSD, Windows WSL
# ==============================================================================

set -euo pipefail

TARGET="${1:-all}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "${SCRIPT_DIR}"

echo "===================================================================="
echo "   PANASONIC DWIRE32A / OCD - MULTI-PLATFORM FIRMWARE BUILDER"
echo "   Target Boards : RP2040 (pico) and RP2350 (pico2)"
echo "   Selection     : ${TARGET}"
echo "===================================================================="

# 1. Locate Pico SDK
if [ -z "${PICO_SDK_PATH:-}" ]; then
    CANDIDATES=(
        "${SCRIPT_DIR}/../pico-sdk"
        "${HOME}/pico-sdk"
        "${HOME}/.pico-sdk/sdk/2.0.0"
        "${HOME}/.pico-sdk/sdk/1.5.1"
        "/usr/share/pico-sdk"
        "/opt/pico-sdk"
    )
    for c in "${CANDIDATES[@]}"; do
        if [ -d "${c}" ]; then
            export PICO_SDK_PATH="${c}"
            break
        fi
    done
fi

if [ -n "${PICO_SDK_PATH:-}" ]; then
    echo "[OK] Using Raspberry Pi Pico SDK at: ${PICO_SDK_PATH}"
else
    echo "[WARNING] PICO_SDK_PATH not set. CMake will attempt automatic fetch if configured."
fi

# 2. Check Prerequisites
command -v cmake >/dev/null 2>&1 || {
    echo "[ERROR] 'cmake' is not installed or not in PATH." >&2
    exit 1
}

command -v arm-none-eabi-gcc >/dev/null 2>&1 || {
    echo "[WARNING] 'arm-none-eabi-gcc' toolchain not detected in PATH." >&2
}

# 3. Helper Build Function
build_board() {
    local board="$1"
    local build_dir="build_${board}"
    local arch_tag="rp2040"
    if [ "${board}" = "pico2" ]; then
        arch_tag="rp2350"
    fi
    local out_name="panasonic_dwire32a_${arch_tag}.uf2"

    echo ""
    echo ">>> [1/2] Configuring CMake for Board: ${board} in ${build_dir}..."
    cmake -B "${build_dir}" -DCMAKE_BUILD_TYPE=Release -DPICO_BOARD="${board}"

    echo ">>> [2/2] Compiling firmware target for ${board}..."
    cmake --build "${build_dir}" --config Release -j"$(nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 4)"

    local src_uf2="${build_dir}/panasonic_dwire32a.uf2"
    if [ -f "${src_uf2}" ]; then
        cp -f "${src_uf2}" "${SCRIPT_DIR}/${out_name}"
        echo "[SUCCESS] Generated ${out_name}"
        if command -v sha256sum >/dev/null 2>&1; then
            sha256sum "${SCRIPT_DIR}/${out_name}"
        elif command -v shasum >/dev/null 2>&1; then
            shasum -a 256 "${SCRIPT_DIR}/${out_name}"
        fi
    else
        echo "[WARNING] UF2 binary not found at ${src_uf2}"
    fi
}

# 4. Execute Selected Build(s)
case "${TARGET}" in
    pico|rp2040)
        build_board "pico"
        ;;
    pico2|rp2350)
        build_board "pico2"
        ;;
    all)
        build_board "pico"
        build_board "pico2"
        if command -v python3 >/dev/null 2>&1 && [ -f "${SCRIPT_DIR}/merge_unified_uf2.py" ]; then
            python3 "${SCRIPT_DIR}/merge_unified_uf2.py" \
                --rp2040 "${SCRIPT_DIR}/panasonic_dwire32a_rp2040.uf2" \
                --rp2350 "${SCRIPT_DIR}/panasonic_dwire32a_rp2350.uf2" \
                --out "${SCRIPT_DIR}/panasonic_dwire32a_unified.uf2"
        fi
        ;;
    *)
        echo "[ERROR] Unknown board target '${TARGET}'. Use 'pico', 'pico2', or 'all'." >&2
        exit 1
        ;;
esac

echo ""
echo "===================================================================="
echo "   MULTI-PLATFORM BUILD COMPLETE!"
echo "===================================================================="
