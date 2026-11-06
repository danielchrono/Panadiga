<#
.SYNOPSIS
    Panasonic DMR-EH55 DIGA Engineering Suite - Unified Root Build Orchestrator.
.DESCRIPTION
    Builds the complete engineering ecosystem:
    1. Runs full workspace test suite (cargo test).
    2. Compiles high-performance Rust CLI binary (target/release/panadiga.exe).
    3. Builds the modern TypeScript + React Desktop interface (apps/desktop/dist).
    Enforces zero-tolerance quality gates, strong typing, and verifies all target artifacts.
#>

[CmdletBinding()]
param (
    [switch]$SkipTests,
    [switch]$Clean
)

$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"

$RootPath = $PSScriptRoot
$RustCliBinary = Join-Path $RootPath "target\release\panadiga.exe"
$DesktopDir = Join-Path $RootPath "apps\desktop"
$DesktopDistHtml = Join-Path $DesktopDir "dist\index.html"

Write-Host "====================================================================" -ForegroundColor Cyan
Write-Host "   PANASONIC DMR-EH55 DIGA ENGINEERING SUITE - UNIFIED BUILD" -ForegroundColor Cyan
Write-Host "   Target Platform: Windows x86_64 (CLI + Desktop Applications)" -ForegroundColor Cyan
Write-Host "====================================================================" -ForegroundColor Cyan

# 1. Environment Verification
Write-Host "`n[1/4] Verificando ferramentas do ambiente (Rust & Node.js)..." -ForegroundColor Yellow

try {
    $cargoVer = cargo --version
    Write-Host "  -> Cargo: $cargoVer" -ForegroundColor Green
} catch {
    Write-Error "Cargo/Rust não encontrado no PATH! Instale o toolchain Rust."
    exit 1
}

try {
    $nodeVer = node --version
    $npmVer = npm --version
    Write-Host "  -> Node.js: $nodeVer | NPM: $npmVer" -ForegroundColor Green
} catch {
    Write-Error "Node.js ou NPM não encontrado no PATH! Instale o Node.js v18+."
    exit 1
}

# Optional Clean
if ($Clean) {
    Write-Host "`n[*] Limpando artefatos anteriores..." -ForegroundColor Yellow
    cargo clean
    if (Test-Path (Join-Path $DesktopDir "dist")) {
        Remove-Item -Recurse -Force (Join-Path $DesktopDir "dist")
    }
}

# 2. Rust Workspace Test Suite
if (-not $SkipTests) {
    Write-Host "`n[2/4] Executando bateria de testes do workspace (cargo test)..." -ForegroundColor Yellow
    cargo test
    if ($LASTEXITCODE -ne 0) {
        Write-Error "A bateria de testes do Cargo falhou! Abortando build."
        exit $LASTEXITCODE
    }
    Write-Host "  -> 100% dos testes unitários e de integração passaram com sucesso!" -ForegroundColor Green
} else {
    Write-Host "`n[2/4] Testes ignorados (--SkipTests ativado)." -ForegroundColor DarkGray
}

# 3. Rust CLI Release Compilation
Write-Host "`n[3/4] Compilando CLI panadiga em modo release (target/release/panadiga.exe)..." -ForegroundColor Yellow
cargo build --release
if ($LASTEXITCODE -ne 0) {
    Write-Error "Falha na compilação da CLI Rust! Abortando build."
    exit $LASTEXITCODE
}

if (-not (Test-Path $RustCliBinary)) {
    Write-Error "Binário final não encontrado em $RustCliBinary!"
    exit 1
}

$cliSize = (Get-Item $RustCliBinary).Length / 1MB
Write-Host ("  -> Binário CLI gerado com sucesso: {0} ({1:N2} MB)" -f $RustCliBinary, $cliSize) -ForegroundColor Green

# 4. Desktop Interface Build (TypeScript + React + Vite)
Write-Host "`n[4/4] Compilando interface Desktop (apps/desktop)..." -ForegroundColor Yellow
Push-Location $DesktopDir

try {
    if (-not (Test-Path "node_modules")) {
        Write-Host "  -> Instalando dependências npm..." -ForegroundColor Gray
        npm install
        if ($LASTEXITCODE -ne 0) {
            throw "npm install falhou com código $LASTEXITCODE"
        }
    }

    Write-Host "  -> Executando npm run build (tsc && vite build)..." -ForegroundColor Gray
    npm run build
    if ($LASTEXITCODE -ne 0) {
        throw "npm run build falhou com código $LASTEXITCODE"
    }

    if (-not (Test-Path $DesktopDistHtml)) {
        throw "Bundle estático da interface desktop não encontrado em $DesktopDistHtml"
    }

    Write-Host "  -> Interface Desktop compilada com sucesso (dist/index.html gerado)!" -ForegroundColor Green
} catch {
    Write-Error "Falha na compilação do Desktop: $_"
    Pop-Location
    exit 1
} finally {
    Pop-Location
}

# Final Summary
Write-Host "`n====================================================================" -ForegroundColor Cyan
Write-Host "   PIPELINE UNIFICADO CONCLUÍDO COM 100% DE SUCESSO!" -ForegroundColor Green
Write-Host "====================================================================" -ForegroundColor Cyan
Write-Host "  [OK] CLI Panadiga: $RustCliBinary" -ForegroundColor White
Write-Host "  [OK] Desktop GUI : $DesktopDistHtml" -ForegroundColor White
Write-Host "====================================================================`n" -ForegroundColor Cyan

exit 0
