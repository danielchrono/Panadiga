<#
.SYNOPSIS
    Panasonic DMR-EH55 DIGA Engineering Suite - Unified Root Build & Distribution Orchestrator.
.DESCRIPTION
    Builds the complete engineering ecosystem:
    1. Runs full workspace test suite (cargo test --workspace).
    2. Compiles high-performance Rust CLI binary (target/release/panadiga.exe).
    3. Builds the modern TypeScript + React Desktop interface (apps/desktop/dist).
    4. Compiles the native Tauri Desktop application (target/release/panasonic-diga-desktop.exe).
    5. Packages all binaries and external tools into dist_bin\ ready for deployment.
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
$RustDesktopBinary = Join-Path $RootPath "target\release\panasonic-diga-desktop.exe"
$DesktopDir = Join-Path $RootPath "apps\desktop"
$DesktopDistHtml = Join-Path $DesktopDir "dist\index.html"
$DistBinDir = Join-Path $RootPath "dist_bin"

Write-Host "====================================================================" -ForegroundColor Cyan
Write-Host "   PANASONIC DMR-EH55 DIGA ENGINEERING SUITE - UNIFIED BUILD" -ForegroundColor Cyan
Write-Host "   Target Platform: Windows x86_64 (CLI + Desktop Binary Distribution)" -ForegroundColor Cyan
Write-Host "====================================================================" -ForegroundColor Cyan

# 1. Environment Verification
Write-Host "`n[1/5] Verificando ferramentas do ambiente (Rust & Node.js)..." -ForegroundColor Yellow

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
    if (Test-Path $DistBinDir) {
        Remove-Item -Recurse -Force $DistBinDir
    }
}

# 2. Rust Workspace Test Suite
if (-not $SkipTests) {
    Write-Host "`n[2/5] Executando bateria de testes do workspace (cargo test --workspace)..." -ForegroundColor Yellow
    cargo test --workspace
    if ($LASTEXITCODE -ne 0) {
        Write-Error "A bateria de testes do Cargo falhou! Abortando build."
        exit $LASTEXITCODE
    }
    Write-Host "  -> 100% dos testes unitários e de integração passaram com sucesso!" -ForegroundColor Green
} else {
    Write-Host "`n[2/5] Testes ignorados (--SkipTests ativado)." -ForegroundColor DarkGray
}

# 3. Rust CLI Release Compilation
Write-Host "`n[3/5] Compilando CLI panadiga em modo release (target/release/panadiga.exe)..." -ForegroundColor Yellow
cargo build --release -p panadiga
if ($LASTEXITCODE -ne 0) {
    Write-Error "Falha na compilação da CLI Rust! Abortando build."
    exit $LASTEXITCODE
}

if (-not (Test-Path $RustCliBinary)) {
    Write-Error "Binário final não encontrado em $RustCliBinary!"
    exit 1
}

$cliSize = (Get-Item $RustCliBinary).Length / 1MB
Write-Host ("  -> Binário CLI gerado: {0} ({1:N2} MB)" -f $RustCliBinary, $cliSize) -ForegroundColor Green

# 4. Desktop Interface Build (TypeScript + React + Vite)
Write-Host "`n[4/5] Compilando interface Desktop (apps/desktop)..." -ForegroundColor Yellow
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

    Write-Host "  -> Interface Desktop compilada (dist/index.html gerado)!" -ForegroundColor Green
} catch {
    Write-Error "Falha na compilação do Desktop: $_"
    Pop-Location
    exit 1
} finally {
    Pop-Location
}

# 5. Native Desktop Application Compilation (Tauri Rust)
Write-Host "`n[5/5] Compilando Aplicação Desktop Nativa (Tauri Rust com assets embutidos)..." -ForegroundColor Yellow
cargo build --release -p panasonic-diga-desktop
if ($LASTEXITCODE -ne 0) {
    Write-Error "Falha na compilação do aplicativo Desktop Tauri! Abortando build."
    exit $LASTEXITCODE
}

if (-not (Test-Path $RustDesktopBinary)) {
    Write-Error "Binário Desktop não encontrado em $RustDesktopBinary!"
    exit 1
}

$desktopSize = (Get-Item $RustDesktopBinary).Length / 1MB
Write-Host ("  -> Binário Desktop gerado: {0} ({1:N2} MB)" -f $RustDesktopBinary, $desktopSize) -ForegroundColor Green

# 6. Packaging to dist_bin\
Write-Host "`n[*] Empacotando distribuição consolidada para dist_bin\..." -ForegroundColor Yellow
if (-not (Test-Path $DistBinDir)) {
    New-Item -ItemType Directory -Path $DistBinDir | Out-Null
}

Copy-Item -Path $RustCliBinary -Destination (Join-Path $DistBinDir "panadiga-cli.exe") -Force
Copy-Item -Path $RustDesktopBinary -Destination (Join-Path $DistBinDir "panadiga-desktop.exe") -Force

$toolsDist = Join-Path $DistBinDir "tools"
if (-not (Test-Path $toolsDist)) {
    New-Item -ItemType Directory -Path $toolsDist | Out-Null
}
Copy-Item -Path (Join-Path $RootPath "tools\*") -Destination $toolsDist -Recurse -Force

Set-Content -Path (Join-Path $DistBinDir "Executar_Panadiga.bat") -Value "@echo off`r`nstart `"`" `"%~dp0panadiga-desktop.exe`""

Write-Host "  -> Pacote consolidado gerado com sucesso em dist_bin\!" -ForegroundColor Green

# Final Summary
Write-Host "`n====================================================================" -ForegroundColor Cyan
Write-Host "   PIPELINE UNIFICADO CONCLUÍDO COM 100% DE SUCESSO!" -ForegroundColor Green
Write-Host "====================================================================" -ForegroundColor Cyan
Write-Host ("  [OK] Panadiga Desktop : {0} ({1:N2} MB)" -f (Join-Path $DistBinDir 'panadiga-desktop.exe'), $desktopSize) -ForegroundColor White
Write-Host ("  [OK] Panadiga CLI     : {0} ({1:N2} MB)" -f (Join-Path $DistBinDir 'panadiga-cli.exe'), $cliSize) -ForegroundColor White
Write-Host "  [OK] Ferramentas      : $(Join-Path $DistBinDir 'tools\') (GDB, OpenOCD, Ghidra, HxD)" -ForegroundColor White
Write-Host "  [OK] Inicializador    : $(Join-Path $DistBinDir 'Executar_Panadiga.bat')" -ForegroundColor White
Write-Host "====================================================================`n" -ForegroundColor Cyan

exit 0
