#!/usr/bin/env bash
# ==============================================================================
# Panasonic DMR-EH55 DIGA Engineering Suite - Unified Build Orchestrator
# Multiplatform: Linux (x86_64 / aarch64) & macOS (Darwin Intel / Apple Silicon)
# ==============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Flags
SKIP_TESTS=false
CLEAN=false
PACKAGE=false

for arg in "$@"; do
    case "$arg" in
        --skip-tests) SKIP_TESTS=true ;;
        --clean) CLEAN=true ;;
        --package) PACKAGE=true ;;
        --help|-h)
            echo "Uso: ./build_all.sh [opções]"
            echo ""
            echo "Opções:"
            echo "  --skip-tests  Ignora a execução da bateria de testes antes de compilar"
            echo "  --clean       Remove artefatos de compilações anteriores antes de iniciar"
            echo "  --package     Gera pacote comprimido (.tar.gz) com somas SHA-256"
            echo "  --help, -h    Exibe este menu de ajuda"
            exit 0
            ;;
        *)
            echo "[AVISO] Parâmetro desconhecido: $arg"
            ;;
    esac
done

# Detect Platform and Architecture
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
    Linux*)
        PLATFORM="linux"
        CPU_CORES="$(nproc 2>/dev/null || getconf _NPROCESSORS_ONLN 2>/dev/null || echo 4)"
        ;;
    Darwin*)
        PLATFORM="macos"
        CPU_CORES="$(sysctl -n hw.ncpu 2>/dev/null || echo 4)"
        ;;
    *)
        PLATFORM="unix-generic"
        CPU_CORES=4
        ;;
esac

export CARGO_BUILD_JOBS="${CPU_CORES}"
export CARGO_INCREMENTAL="1"

# Compiler Cache (sccache) support
if command -v sccache >/dev/null 2>&1; then
    export RUSTC_WRAPPER="sccache"
    echo "[+] sccache detectado e habilitado para cache de compilação!"
fi

echo "===================================================================="
echo "   PANASONIC DMR-EH55 DIGA ENGINEERING SUITE - BUILD UNIFICADO"
echo "   Plataforma: $OS $ARCH ($CPU_CORES Cores | UTF-8)"
echo "===================================================================="
echo ""

# 1. Environment Verification
echo "[1/6] Verificando ferramentas do ambiente (Rust & Node.js)..."
if ! command -v cargo >/dev/null 2>&1; then
    echo "[ERRO] Cargo/Rust não encontrado no PATH! Instale via https://rustup.rs"
    exit 1
fi
echo "  -> Cargo: $(cargo --version)"

if ! command -v node >/dev/null 2>&1 || ! command -v npm >/dev/null 2>&1; then
    echo "[ERRO] Node.js ou NPM não encontrado no PATH! Instale Node.js v18+"
    exit 1
fi
echo "  -> Node.js: $(node --version) | NPM: $(npm --version)"

# Optional Clean
if [ "$CLEAN" = true ]; then
    echo ""
    echo "[*] Limpando artefatos anteriores..."
    cargo clean
    rm -rf apps/desktop/dist
    rm -rf dist_bin
fi

# 2. Rust Workspace Tests
if [ "$SKIP_TESTS" = false ]; then
    echo ""
    echo "[2/6] Executando bateria de testes Rust (cargo test -p panadiga)..."
    cargo test -p panadiga
    echo "  -> 100% dos testes unitários e de integração passaram com sucesso!"
else
    echo ""
    echo "[2/6] Testes ignorados (--skip-tests ativado)."
fi

# 3. Rust CLI Release Compilation
echo ""
echo "[3/6] Compilando CLI panadiga em modo release..."
cargo build --release -p panadiga

CLI_SRC="target/release/panadiga"
if [ ! -f "$CLI_SRC" ]; then
    echo "[ERRO] Binário CLI não encontrado em $CLI_SRC!"
    exit 1
fi
CLI_SIZE="$(du -h "$CLI_SRC" | cut -f1)"
echo "  -> Binário CLI gerado: $CLI_SRC ($CLI_SIZE)"

# 4. Desktop Interface Build (TypeScript + React + Vite)
echo ""
echo "[4/6] Compilando interface Desktop (apps/desktop)..."
(
    cd apps/desktop
    if [ ! -d "node_modules" ]; then
        echo "  -> Instalando dependências npm..."
        npm install
    fi
    echo "  -> Executando build da UI (tsc && vite build)..."
    npm run build
)

if [ ! -f "apps/desktop/dist/index.html" ]; then
    echo "[ERRO] Bundle estático da interface desktop não encontrado em apps/desktop/dist/index.html!"
    exit 1
fi
echo "  -> Interface Desktop compilada com sucesso (apps/desktop/dist/)!"

# 5. Native Desktop Application Compilation (Tauri Rust)
echo ""
echo "[5/6] Compilando Aplicação Desktop Nativa (Tauri Rust)..."
cargo build --release -p panasonic-diga-desktop

DESKTOP_SRC="target/release/panasonic-diga-desktop"
if [ ! -f "$DESKTOP_SRC" ]; then
    echo "[ERRO] Binário Desktop não encontrado em $DESKTOP_SRC!"
    exit 1
fi
DESKTOP_SIZE="$(du -h "$DESKTOP_SRC" | cut -f1)"
echo "  -> Binário Desktop gerado: $DESKTOP_SRC ($DESKTOP_SIZE)"

# 6. Packaging to dist_bin/
echo ""
echo "[6/6] Empacotando distribuição para dist_bin/..."
mkdir -p dist_bin
mkdir -p dist_bin/tools

cp -f "$CLI_SRC" dist_bin/panadiga-cli
cp -f "$DESKTOP_SRC" dist_bin/panadiga-desktop
chmod +x dist_bin/panadiga-cli dist_bin/panadiga-desktop

# Copy external lab tools
if [ -d "tools" ]; then
    cp -rf tools/* dist_bin/tools/ 2>/dev/null || true
fi

# Generate Unix launcher script
cat << 'EOF' > dist_bin/Executar_Panadiga.sh
#!/usr/bin/env bash
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
exec "$SCRIPT_DIR/panadiga-desktop" "$@"
EOF
chmod +x dist_bin/Executar_Panadiga.sh

echo "  -> Pacote consolidado gerado em dist_bin/!"

# Generate Release Archive (.tar.gz) with SHA-256 Checksum
if [ "$PACKAGE" = true ] || [ "$CLEAN" = false ]; then
    echo ""
    echo "[*] Gerando Pacote de Release (.tar.gz) e Checksums SHA256..."
    mkdir -p dist_bin/releases
    RELEASE_ARCHIVE="dist_bin/releases/panadiga-${PLATFORM}-${ARCH}.tar.gz"
    
    tar -czf "$RELEASE_ARCHIVE" \
        -C dist_bin panadiga-cli panadiga-desktop Executar_Panadiga.sh tools \
        -C "$SCRIPT_DIR" README.md LICENSE 2>/dev/null || \
    tar -czf "$RELEASE_ARCHIVE" \
        -C dist_bin panadiga-cli panadiga-desktop Executar_Panadiga.sh tools \
        -C "$SCRIPT_DIR" README.md

    # Compute SHA-256
    if command -v sha256sum >/dev/null 2>&1; then
        HASH="$(sha256sum "$RELEASE_ARCHIVE" | awk '{print $1}')"
    elif command -v shasum >/dev/null 2>&1; then
        HASH="$(shasum -a 256 "$RELEASE_ARCHIVE" | awk '{print $1}')"
    else
        HASH="CHECKSUM_TOOL_NOT_AVAILABLE"
    fi

    CHECKSUM_FILE="dist_bin/releases/SHA256SUMS.txt"
    echo "$HASH  panadiga-${PLATFORM}-${ARCH}.tar.gz" > "$CHECKSUM_FILE"

    echo "  -> Pacote Release: $RELEASE_ARCHIVE"
    echo "  -> Checksum SHA256: $HASH"
fi

echo ""
echo "===================================================================="
echo "   PIPELINE MULTIPLATAFORMA CONCLUÍDO COM 100% DE SUCESSO!"
echo "===================================================================="
echo "  [OK] Panadiga Desktop : dist_bin/panadiga-desktop ($DESKTOP_SIZE)"
echo "  [OK] Panadiga CLI     : dist_bin/panadiga-cli ($CLI_SIZE)"
echo "  [OK] Ferramentas      : dist_bin/tools/ (GDB, OpenOCD, Ghidra)"
echo "  [OK] Inicializador    : dist_bin/Executar_Panadiga.sh"
echo "===================================================================="
echo ""
exit 0
